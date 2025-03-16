#![no_std]  // pas d'utilisation de la bibliothèque 
#![cfg_attr(feature = "alloc", feature(allocator_api))]  
#![feature(alloc_error_handler)]  // Rust nightly pour activer la fonctionnalité de #[alloc_error_handler],permet de définir une fonction personnalisée pour gérer les erreurs d'allocation de mémoire.
/// Rust Nightly est une version expérimentale de Rust qui contient les dernières fonctionnalités en développement avant qu'elles ne soient stabilisées dans les versions Rust Stable.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use core::mem;

// J'ai choisi d'utiliser 4 tailles de slabs différentes pour gérer efficacement
// différentes tailles d'allocations. C'est un compromis entre flexibilité et complexité.
// Définition de la taille d'un slab 
const SLAB_SIZES: [usize; 4] = [16, 32, 64, 128]; // Différentes tailles de slabs
const MAX_SLABS: usize = 100; // Nombre maximum de slabs par taille


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
            memory_start: start,
            memory_end: start + size,
            free_lists: [ptr::null_mut(); SLAB_SIZES.len()],
            allocated_counts: [0; SLAB_SIZES.len()],
        }
    }
    
    /// Initialise les listes de blocs libres
    pub unsafe fn init(&mut self) {
        for (i, &_size) in SLAB_SIZES.iter().enumerate() {
            // Ne pas initialiser maintenant, on le fera à la demande
            self.free_lists[i] = ptr::null_mut();
            self.allocated_counts[i] = 0;
        }
    }
    
    /// Trouve l'index de la taille de slab appropriée
    fn find_slab_index(&self, layout: &Layout) -> Option<usize> {
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

    unsafe fn allocate_more_slabs(&mut self, slab_index: usize) -> Result<(), ()> {
        let slab_size = SLAB_SIZES[slab_index];
        
        // Calculer combien de slabs on peut créer
        let slabs_to_create = 10; // Créons 10 slabs à la fois
        let required_memory = slab_size * slabs_to_create;
        
        // Trouver une zone libre dans notre mémoire
        // (Implémentation très basique pour l'exemple)
        let current_alloc = self.memory_start + self.allocated_counts.iter().enumerate()
            .map(|(i, &count)| count * SLAB_SIZES[i])
            .sum::<usize>();
            
        if current_alloc + required_memory > self.memory_end {
            return Err(());  // Plus assez de mémoire
        }
        
        // Créer les nouveaux slabs et les ajouter à la liste libre
        for i in 0..slabs_to_create {
            let block_addr = current_alloc + i * slab_size;
            let block_ptr = block_addr as *mut SlabBlock;
            
            // Ajouter à la liste libre
            (*block_ptr).next = self.free_lists[slab_index];
            self.free_lists[slab_index] = block_ptr;
        }
        
        self.allocated_counts[slab_index] += slabs_to_create;
        Ok(())
    }
}

unsafe impl GlobalAlloc for SlabAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Créer une version mutable de self
        let this = &mut *(self as *const _ as *mut SlabAllocator);
        
        // Trouver la taille de slab appropriée
        if let Some(slab_index) = this.find_slab_index(&layout) {
            // Vérifier s'il y a un bloc libre disponible
            if this.free_lists[slab_index].is_null() {
                // Allouer plus de slabs si nécessaire
                if this.allocate_more_slabs(slab_index).is_err() {
                    return ptr::null_mut();
                }
            }
            
            // Prendre un bloc de la liste libre
            let block = this.free_lists[slab_index];
            this.free_lists[slab_index] = (*block).next;
            
            return block as *mut u8;
        }
        
        // Aucune taille de slab appropriée
        ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Créer une version mutable de self
        let this = &mut *(self as *const _ as *mut SlabAllocator);
        
        // Trouver la taille de slab appropriée
        if let Some(slab_index) = this.find_slab_index(&layout) {
            // Ajouter le bloc à la liste libre
            let block = ptr as *mut SlabBlock;
            (*block).next = this.free_lists[slab_index];
            this.free_lists[slab_index] = block;
        }
    }
}

// Définir un allocateur global si la feature 'alloc' est activée
#[cfg(feature = "alloc")]
#[global_allocator]
static mut ALLOCATOR: SlabAllocator = SlabAllocator::new(0, 0);

// Initialisation de l'allocateur global avec une région de mémoire
#[cfg(feature = "alloc")]
pub fn init_global_allocator(start: usize, size: usize) {
    unsafe {
        ALLOCATOR = SlabAllocator::new(start, size);
        ALLOCATOR.init();
    }
}

// Fonction d'allocation de secours
#[alloc_error_handler]
fn alloc_error_handler(_layout: Layout) -> ! {
    loop {}  // Boucle infinie en cas d'erreur d'allocation
}

/// les tests

#[cfg(test)]
mod tests{
    use super::*;
    use core::alloc::Layout;
    use core::mem;


    // Une Structure pour tester l'allocation
    struct TestStruct {
        a: u32,
        b: u64,
        c: [u8; 32],
    }

    // Test Simple D'allocation et désallocation
    #[test]
    fn test_alloc_dealloc(){
        // Créer un espace mémoire pour notre allocateur (simulé pour les tests)
        let mem_size = 1024 * 1024; // 1MB
        let memory = Box::into_raw(Box::new([0u8; 1024 $ 1024])) as usize;

        let allocator = SlabAllocator::new(memory, mem_size);


        unsafe {
            // Initialiser l'allocateur
            let mut alloc = allocator;
            alloc.init();

            // Allouer de la mémoire pour un u32
            let layout = Layout::new::<u32>();
            let ptr = alloc.alloc(layout);
            assert!(!ptr.is_null());

            // Ecrire dans la mémoire allouée
            *(ptr as *mut u32) = 42;
            assert_eq!(*(ptr as *mut u32), 42);

            // Désallouer/Déalloc
            alloc.dealloc(ptr, layout);

            // Nettoyer la mémoire de test
            Box::from_raw(memory as *mut [u8; 1024 * 1024]);
        }

    } 


    // Test d'allocation pour plusieurs tailles
    #[test]
    fn test_different_sizes() {
        let mem_size = 1024 * 1024; // 1MB
        let memory = box::into_raw(Box::new([0u8; 1024 * 1024])) as usize;
        
        let allocator = SlabAllocator::new(memory, mem_size);


        unsafe {
            let mut alloc = allocator;
            alloc.init();



            // Tester différentes tailles d'allocation
            let layouts = [
                Layout::new::<u8>(),
                Layout::new::<u32>(),
                Layout::new::<u64>(),
                Layout::new::<[u8; 32]>(),
                Layout::new::<TestStruct>(),
            ];

            let mut ptrs = Vec::with_capacity(layouts.len());

            // Allouer
            for layout in &layouts {
                let ptr = alloc.alloc(*layout);
                assert!(!ptr.is_null());
                ptrs.push((ptr, *layout));
            }

            // Désallouer
            for (ptr, layout) in ptrs {
                alloc.dealloc(ptr, layout);

            }

            // Nettoyage de ram utilisé
            Box::from_raw(memory as *mut [u8; 1024 * 1024]);


        }
    }

    // Test de réutilisation des blocs
    #[test]
    fn test_reuse_blocks() {
        let mem_size = 1024 * 1024; // 1MB
        let memory = Box::into_raw(Box::new([0u8; 1024 * 1024])) as usize;

        let alloator = SlabAllocator::new(memory, mem_size);
        
        
        unsafe {
            let mut alloc = alloator;
            alloc.init();

            let layout = Layout::new::<u32>();

            // première allocation
            let ptr1 = alloc.alloc(layout);
            assert!(!ptr1.is_null());


            // Désallocation
            alloc.dealloc(ptr1, layout);


            // Deuxieme allocation - ça doit réutilisé le meme bloc
            let ptr2 = alloc.alloc(layout);
            assert!(!ptr2.is_null());


            // Les pointeurs doivent être identiques(réutilisation)
            assert_eq!(ptr1, ptr2);

            // Nettoyer
            alloc.dealloc(ptr2, layout);
            Box::from_raw(memory as *mut [u8; 1024 * 1024]);

        }
    }
}
