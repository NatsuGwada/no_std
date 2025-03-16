#![no_std]  // pas d'utilisation de la bibliothèque 

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use core::mem;
use core::cell::UnsafeCell;

// J'ai choisi d'utiliser 4 tailles de slabs différentes pour gérer efficacement
// différentes tailles d'allocations. C'est un compromis entre flexibilité et complexité.
// Définition de la taille d'un slab 
const SLAB_SIZES: [usize; 4] = [16, 32, 64, 128]; // Différentes tailles de slabs

// Cette structure représente un bloc de mémoire libre.
// Quand il est libre, on peut utiliser son espace pour stocker un pointeur
// vers le prochain bloc libre, formant ainsi une liste chaînée.
/// Un bloc de mémoire dans notre allocateur
#[repr(C)]
struct SlabBlock {
    next: *mut SlabBlock, // Pointeur vers le prochain bloc libre
}

/// Notre allocateur de type Slab
pub struct SlabAllocator {
    // Utilisation d'UnsafeCell pour permettre une mutabilité intérieure
    // tout en respectant les règles de Rust concernant les références mutables
    inner: UnsafeCell<SlabAllocatorInner>,
}

/// Structure interne contenant les données de l'allocateur
struct SlabAllocatorInner {
    // La mémoire que notre allocateur va gérer
    memory_start: usize,
    memory_end: usize,
    
    // Liste de blocs libres pour chaque taille de slab
    free_lists: [*mut SlabBlock; SLAB_SIZES.len()],
    
    // Nombre de slabs alloués pour chaque taille
    allocated_counts: [usize; SLAB_SIZES.len()],
}

impl SlabAllocator {
    /// Crée un nouvel allocateur
    /// 
    /// # Safety
    /// 
    /// La région de mémoire fournie doit être valide et non utilisée par d'autres 
    /// parties du programme.
    pub const fn new(start: usize, size: usize) -> Self {
        SlabAllocator {
            inner: UnsafeCell::new(SlabAllocatorInner {
                memory_start: start,
                memory_end: start + size,
                free_lists: [ptr::null_mut(); SLAB_SIZES.len()],
                allocated_counts: [0; SLAB_SIZES.len()],
            }),
        }
    }
    
    /// Initialise les listes de blocs libres
    pub unsafe fn init(&self) {
        let inner = &mut *self.inner.get();
        for (i, &_size) in SLAB_SIZES.iter().enumerate() {
            // Ne pas initialiser maintenant, on le fera à la demande
            inner.free_lists[i] = ptr::null_mut();
            inner.allocated_counts[i] = 0;
        }
    }
    
    /// Trouve l'index de la taille de slab appropriée
    unsafe fn find_slab_index(&self, layout: &Layout) -> Option<usize> {
        let inner = &*self.inner.get();
        let required_size = layout.size().max(mem::size_of::<SlabBlock>());
        
        for (i, &size) in SLAB_SIZES.iter().enumerate() {
            if size >= required_size && layout.align() <= size {
                return Some(i);
            }
        }
        
        None // Aucune taille de slab appropriée
    }
    
    /// Cette fonction est unsafe car elle manipule directement des pointeurs
    /// et accède à la mémoire sans vérification.
    ///
    /// # Safety
    ///
    /// Le code appelant doit garantir que:
    /// - L'entrée `slab_index` est valide et dans les limites de `SLAB_SIZES`
    /// - La région mémoire spécifiée par `memory_start` et `memory_end` est valide
    /// - Cette fonction ne doit pas être appelée concurremment avec d'autres opérations sur l'allocateur
    unsafe fn allocate_more_slabs(&self, slab_index: usize) -> Result<(), ()> {
        let inner = &mut *self.inner.get();
        let slab_size = SLAB_SIZES[slab_index];
        
        // Calculer combien de slabs on peut créer
        let slabs_to_create = 10; // Créons 10 slabs à la fois
        let required_memory = slab_size * slabs_to_create;
        
        // Trouver une zone libre dans notre mémoire
        // (Implémentation très basique pour l'exemple)
        let current_alloc = inner.memory_start + inner.allocated_counts.iter().enumerate()
            .map(|(i, &count)| count * SLAB_SIZES[i])
            .sum::<usize>();
            
        if current_alloc + required_memory > inner.memory_end {
            return Err(());  // Plus assez de mémoire
        }
        
        // Créer les nouveaux slabs et les ajouter à la liste libre
        for i in 0..slabs_to_create {
            let block_addr = current_alloc + i * slab_size;
            let block_ptr = block_addr as *mut SlabBlock;
            
            // Ajouter à la liste libre
            (*block_ptr).next = inner.free_lists[slab_index];
            inner.free_lists[slab_index] = block_ptr;
        }
        
        inner.allocated_counts[slab_index] += slabs_to_create;
        Ok(())
    }
}

// Implémentation thread-safe de GlobalAlloc
unsafe impl GlobalAlloc for SlabAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Utiliser UnsafeCell.get() pour accéder de manière sûre aux données mutables
        let inner = &mut *self.inner.get();
        
        // Trouver la taille de slab appropriée
        if let Some(slab_index) = self.find_slab_index(&layout) {
            // Vérifier s'il y a un bloc libre disponible
            if inner.free_lists[slab_index].is_null() {
                // Allouer plus de slabs si nécessaire
                if self.allocate_more_slabs(slab_index).is_err() {
                    return ptr::null_mut();
                }
            }
            
            // Prendre un bloc de la liste libre
            let block = inner.free_lists[slab_index];
            inner.free_lists[slab_index] = (*block).next;
            
            return block as *mut u8;
        }
        
        // Aucune taille de slab appropriée
        ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Utiliser UnsafeCell.get() pour accéder de manière sûre aux données mutables
        let inner = &mut *self.inner.get();
        
        // Trouver la taille de slab appropriée
        if let Some(slab_index) = self.find_slab_index(&layout) {
            // Ajouter le bloc à la liste libre
            let block = ptr as *mut SlabBlock;
            (*block).next = inner.free_lists[slab_index];
            inner.free_lists[slab_index] = block;
        }
    }
}

// Définir un allocateur global si la feature 'alloc' est activée
#[cfg(feature = "alloc")]
pub struct GlobalSlabAllocator {
    inner: UnsafeCell<SlabAllocator>,
    initialized: core::sync::atomic::AtomicBool,
}

#[cfg(feature = "alloc")]
unsafe impl Sync for GlobalSlabAllocator {}

#[cfg(feature = "alloc")]
impl GlobalSlabAllocator {
    pub const fn new() -> Self {
        GlobalSlabAllocator {
            inner: UnsafeCell::new(SlabAllocator::new(0, 0)),
            initialized: core::sync::atomic::AtomicBool::new(false),
        }
    }
    
    pub fn init(&self, start: usize, size: usize) {
        use core::sync::atomic::Ordering;
        
        if !self.initialized.load(Ordering::Acquire) {
            unsafe {
                let allocator = &mut *self.inner.get();
                *allocator = SlabAllocator::new(start, size);
                allocator.init();
                self.initialized.store(true, Ordering::Release);
            }
        }
    }
}

#[cfg(feature = "alloc")]
unsafe impl GlobalAlloc for GlobalSlabAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        use core::sync::atomic::Ordering;
        
        if !self.initialized.load(Ordering::Acquire) {
            return ptr::null_mut();
        }
        
        (*self.inner.get()).alloc(layout)
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        use core::sync::atomic::Ordering;
        
        if self.initialized.load(Ordering::Acquire) {
            (*self.inner.get()).dealloc(ptr, layout);
        }
    }
}

// Définir un allocateur global si la feature 'alloc' est activée
#[cfg(feature = "alloc")]
#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalSlabAllocator = GlobalSlabAllocator::new();

// Initialisation de l'allocateur global avec une région de mémoire
#[cfg(feature = "alloc")]
pub fn init_global_allocator(start: usize, size: usize) {
    GLOBAL_ALLOCATOR.init(start, size);
}

/// les tests
#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::Layout;

    // Une Structure pour tester l'allocation
    struct TestStruct {
        a: u32,
        b: u64,
        c: [u8; 32],
    }

    #[test]
    fn test_alloc_dealloc() {
        // Créer un espace mémoire pour notre allocateur (simulé pour les tests)
        static mut TEST_MEMORY: [u8; 1024 * 1024] = [0; 1024 * 1024];
        
        unsafe {
            let memory_ptr = TEST_MEMORY.as_ptr() as usize;
            let allocator = SlabAllocator::new(memory_ptr, TEST_MEMORY.len());
            allocator.init();

            // Allouer de la mémoire pour un u32
            let layout = Layout::new::<u32>();
            let ptr = allocator.alloc(layout);
            assert!(!ptr.is_null());

            // Ecrire dans la mémoire allouée
            *(ptr as *mut u32) = 42;
            assert_eq!(*(ptr as *mut u32), 42);

            // Désallouer/Déalloc
            allocator.dealloc(ptr, layout);
        }
    }

    #[test]
    fn test_different_sizes() {
        // Créer un espace mémoire pour notre allocateur (simulé pour les tests)
        static mut TEST_MEMORY: [u8; 1024 * 1024] = [0; 1024 * 1024];
        
        unsafe {
            let memory_ptr = TEST_MEMORY.as_ptr() as usize;
            let allocator = SlabAllocator::new(memory_ptr, TEST_MEMORY.len());
            allocator.init();

            // Tester différentes tailles d'allocation
            let layouts = [
                Layout::new::<u8>(),
                Layout::new::<u32>(),
                Layout::new::<u64>(),
                Layout::new::<[u8; 32]>(),
                Layout::new::<TestStruct>(),
            ];

            // Utilisons un tableau pour stocker les pointeurs alloués
            let mut ptrs = [(ptr::null_mut(), Layout::from_size_align(0, 1).unwrap()); 5];
            
            // Allouer
            for (i, &layout) in layouts.iter().enumerate() {
                let ptr = allocator.alloc(layout);
                assert!(!ptr.is_null());
                ptrs[i] = (ptr, layout);
            }

            // Désallouer
            for (ptr, layout) in ptrs.iter() {
                if !ptr.is_null() {
                    allocator.dealloc(*ptr, *layout);
                }
            }
        }
    }

    #[test]
    fn test_reuse_blocks() {
        // Créer un espace mémoire pour notre allocateur (simulé pour les tests)
        static mut TEST_MEMORY: [u8; 1024 * 1024] = [0; 1024 * 1024];
        
        unsafe {
            let memory_ptr = TEST_MEMORY.as_ptr() as usize;
            let allocator = SlabAllocator::new(memory_ptr, TEST_MEMORY.len());
            allocator.init();

            let layout = Layout::new::<u32>();

            // première allocation
            let ptr1 = allocator.alloc(layout);
            assert!(!ptr1.is_null());

            // Désallocation
            allocator.dealloc(ptr1, layout);

            // Deuxieme allocation - ça doit réutilisé le meme bloc
            let ptr2 = allocator.alloc(layout);
            assert!(!ptr2.is_null());

            // Les pointeurs doivent être identiques(réutilisation)
            assert_eq!(ptr1, ptr2);

            // Nettoyer
            allocator.dealloc(ptr2, layout);
        }
    }
}
