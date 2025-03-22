#![no_std]  
///! Pas d'utilisation de la bibliothèque standard



/// import des modules nécéssaires
/// utiliser code car on est en bas niveau et qu'on a pas std
use core::alloc::{GlobalAlloc, Layout}; /// trait obligatoire pour utilisé l'allocateur , layout c'est l'alignement (la ou tu peux stocker) et la taille du bloc mémoire (combien de mémoire tu veux)
use core::ptr;  /// ça c'est pour manipuler des pointeurs bas niveau, cool pour chercher des adresses en ram
use core::mem;  /// pour accéder a la ram et à c'est infos
use core::cell::UnsafeCell; /// ça c'est pour muer les données internes de l'allocateur, unsafeCelle rend mutable les données static, &self

/// J'ai choisi d'utiliser 4 tailles de slabs différentes pour ne pas gaspiller la ram.
/// tu veux alouer 20 octés bin voila un slab de 32 comme ça tu es tranquille
/// Définition de la taille d'un slab (slab = bloc mémoires en français)
const SLAB_SIZES: [usize; 4] = [16, 32, 64, 128]; // Différentes tailles de slabs

/// Cette structure représente un bloc de mémoire libre.
/// Quand il est libre, on peut utiliser son espace pour stocker un pointeur
/// vers le prochain bloc libre, et ça forme ainsi une liste chaînée.
/// Voila, Un bloc de mémoire dans notre allocateur
#[repr(C)]
struct SlabBlock {
    next: *mut SlabBlock, // Pointeur vers le prochain bloc libre
}


/// Notre allocateur de type Slab
pub struct SlabAllocator {
    /// Utilisation d'UnsafeCell pour permettre une mutabilité intérieure
    /// tout en respectant les règles de Rust concernant les références mutables
    inner: UnsafeCell<SlabAllocatorInner>,
}

/// Structure interne contenant les données de l'allocateur
struct SlabAllocatorInner {
    /// La mémoire que notre allocateur va gérer, debut et fin (les limites)
    /// usize = valeurs d'adresse mémoire, en gros un pointeur non signé
    memory_start: usize,  
    memory_end: usize,
    
    /// Liste\tableu de blocs libres pour chaque taille de slab, SlabBlock = Block mémoire & slabsize sa taille
    free_lists: [*mut SlabBlock; SLAB_SIZES.len()],
    
    /// Nombre de slabs alloués pour chaque taille
    allocated_counts: [usize; SLAB_SIZES.len()],
}

impl SlabAllocator {
    /// Crée un nouvel allocateur
    /// 
    /// # Safety
    /// 
    /// La région de mémoire fournie doit être valide et non utilisée par d'autres parties du programme.
    /// L'appelant doit garantir que la mémoire spécifiée par `start` et `size`
    /// est valide, libre, et ne sera pas utilisée ailleurs dans le programme. 
    /// une constante car elle va etre evaluer a la compilation pour les test et comme c'est du no_std donc il y a des initialisation a la compilation
    pub const fn new(start: usize, size: usize) -> Self {
        SlabAllocator {
            // Inner = novuelle instance embalé sans unsafecell
            inner: UnsafeCell::new(SlabAllocatorInner {
                // Démarrage de l'attribution de taille mémoire
                memory_start: start,
                memory_end: start + size,
                // Plage mémoire défini
                free_lists: [ptr::null_mut(); SLAB_SIZES.len()],
                allocated_counts: [0; SLAB_SIZES.len()],
            }),
        }
    }
    
    /// Initialise les listes de blocs libres
    /// 
    /// # Safety
    /// 
    /// Cette fonction est unsafe car elle modifie la mémoire interne
    /// unsafe va modifier l'interieur de UnsafeCell
    /// &self lis les données de la structure sans les modifiers, en gros de la lecture seule mais avec unsafeCell tu passes en lecture écriture donc tu peux modifier
    pub unsafe fn init(&self) {
        // self.inner donne un pointeur brut mutable sur &mut SlabAllocatorInner, d'ailleurs il est deviner tout seul par rust (en gros le mot est grisé dans mon vscode)
        let inner = &mut *self.inner.get();
        // on parcourt les index et les tailles des blocs (16, 32, 64, 128)
        for (i, &_size) in SLAB_SIZES.iter().enumerate() {
            // ça c'est pour vidé la liste de bloc libres
            inner.free_lists[i] = ptr::null_mut();
            // ça c'est pour remettre a 0 le compteur de bloc alloué pour la taille choisi
            inner.allocated_counts[i] = 0;
        }
    }
    
    /// Trouve l'index de la taille de slab appropriée , en gros ça choisi quelle taile de bloc conviens a la demande d'allocation doonée 
    ///
    /// # Safety
    /// on reviens a l'exemple d'en haut 20 octer tu veux? ok tien voila 32 octer de dispo pour toi
    unsafe fn find_slab_index(&self, layout: &Layout) -> Option<usize> {
        // let inner = &*self.inner.get();
        // bon ça , ça calculle la taille minimal nécéssaire
        let required_size = layout.size().max(mem::size_of::<SlabBlock>());
        // La on parcours les 4 tailles possible, si compatibilité (sois les slab est assser grand ou pas), puis on retourn l'index si c'est bon sinon on renvoie None
        // l'index c'est la position de la taille dans le tableau [16, 32, 64, 128], 16 = index 0, 32 = index 1, ect..
        for (i, &size) in SLAB_SIZES.iter().enumerate() {
            if size >= required_size && layout.align() <= size {
                // renvoie de l'index ici
                return Some(i);
            }
        }
        
        None // Aucune taille de slab appropriée, normalement il y a que une taille trop grosse par rapport a 128 qui ne passe pas
    }
    
    /// Cette fonction est unsafe car elle manipule directement des pointeurs pour alouer plus de blocs libres pour un slab donnée
    /// et accède à la mémoire sans vérification.
    ///
    /// # Safety
    ///
    /// Le code appelant doit garantir que:
    /// - L'entrée `slab_index` est valide et dans les limites de `SLAB_SIZES`
    /// - La région mémoire spécifiée par `memory_start` et `memory_end` est valide
    /// - Cette fonction ne doit pas être appelée concurremment avec d'autres opérations sur l'allocateur
    unsafe fn allocate_more_slabs(&self, slab_index: usize) -> Result<(), ()> {

        // Récupération de la référenc emutable 
        let inner = &mut *self.inner.get();
        // récupération de la taille des blocs
        let slab_size = SLAB_SIZES[slab_index];
        
        // Calculer combien de slabs on peut créer
        let slabs_to_create = 10; // Créons 10 slabs à la fois
        let required_memory = slab_size * slabs_to_create;
        
        // Trouver une zone libre dans notre mémoire
        let current_alloc = inner.memory_start + inner.allocated_counts.iter().enumerate()
            .map(|(i, &count)| count * SLAB_SIZES[i])
            .sum::<usize>();
        
        // Vérifie si il reste asser de place
        if current_alloc + required_memory > inner.memory_end {
            return Err(());  // Plus assez de mémoire
        }
        
        // Créer les nouveaux slabs et les ajouter à la liste libre
        for i in 0..slabs_to_create {
            // Assurer que l'adresse du bloc est correctement alignée
            let block_addr = current_alloc + i * slab_size;
            // les allignement mémoires
            let alignment = mem::align_of::<SlabBlock>();
            let aligned_addr = (block_addr + alignment - 1) & !(alignment - 1);
            // Transformation de l'adresse aligné en "pointeur vers un bloc mémoire"
            let block_ptr = aligned_addr as *mut SlabBlock;
            
            // Ajouter à la liste libre  
            (*block_ptr).next = inner.free_lists[slab_index];
            inner.free_lists[slab_index] = block_ptr;
        }
        // Mise à jour du compteur
        inner.allocated_counts[slab_index] += slabs_to_create;
        Ok(())
    }
}

/// Implémentation thread-safe de GlobalAlloc
/// # Safety
/// On dit ici que SlabAllocator implémente le trait GlobalAlloc, ce qui permet de l’utiliser comme allocateur global pour tout le programme.
/// ⚠️ unsafe car on manipule la mémoire manuellement via des pointeurs
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
            
            // Vérifier l'alignement
            let ptr = block as *mut u8;
            if (ptr as usize) % layout.align() != 0 {
                // Le pointeur n'est pas aligné correctement, on le désalloue
                // et on retourne une erreur
                (*block).next = inner.free_lists[slab_index];
                inner.free_lists[slab_index] = block;
                return ptr::null_mut();
            }
            // Sinon, on retourne le pointeur vers la mémoire prête à être utilisé
            return ptr;
        }
        
        // Aucune taille de slab appropriée
        ptr::null_mut()
    }
    /// Fonction appelée quand on veux désallouer uen zone mémoire
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Utiliser UnsafeCell.get() pour accéder de manière sûre aux données mutables
        let inner = &mut *self.inner.get();
        
        // Trouver la taille de slab appropriée
        if let Some(slab_index) = self.find_slab_index(&layout) {
            // Ajouter le bloc à la liste libre
            let block = ptr as *mut SlabBlock;
            // ajout du bloc en tête de liste
            (*block).next = inner.free_lists[slab_index];
            inner.free_lists[slab_index] = block;
        }
    }
}

// Définir un allocateur global si la feature 'alloc' est activée
#[cfg(feature = "alloc")]
pub struct GlobalSlabAllocator {
    // Encepsulation de l'allocateru standard en Unsafecell pour le rendre mutable
    inner: UnsafeCell<SlabAllocator>,
    // un booléan(truc qui dit oui ou non) qui vérrifie si l'allocateur a déja été initialisé
    initialized: core::sync::atomic::AtomicBool,
}

/// une structure est thread-safe car `UnsafeCell` n'est pas Sync par défaut
#[cfg(feature = "alloc")]
unsafe impl Sync for GlobalSlabAllocator {}

#[cfg(feature = "alloc")]
impl GlobalSlabAllocator {
    /// Fonction constante pour initialisé une instance vide à la compilation
    pub const fn new() -> Self {
        GlobalSlabAllocator {
            /// Valeurinitiale = alllocateur vide ( adresse 0, taill e 0)
            inner: UnsafeCell::new(SlabAllocator::new(0, 0)),
            initialized: core::sync::atomic::AtomicBool::new(false),
        }
    }
    
    /// initialisation de l'allocateur avec une plage mémoire donnée
    pub fn init(&self, start: usize, size: usize) {
        use core::sync::atomic::Ordering;
        
        /// si pas encore initialisé
        if !self.initialized.load(Ordering::Acquire) {
            unsafe {
                // récupère l'accès mutable à l'allocateur
                let allocator = &mut *self.inner.get();
                *allocator = SlabAllocator::new(start, size); // crée un nouvelle allocateur
                allocator.init(); // initialise les structures internes
                self.initialized.store(true, Ordering::Release); // signale que c'est pret
            }
        }
    }
}

#[cfg(feature = "alloc")]
unsafe impl GlobalAlloc for GlobalSlabAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        use core::sync::atomic::Ordering;
        // Si l'allocateur n'est pas initialisé , on fait rien
        if !self.initialized.load(Ordering::Acquire) {
            return ptr::null_mut();
        }
        // laisse l'allocation à l'allocateur interne
        (*self.inner.get()).alloc(layout)
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        use core::sync::atomic::Ordering;
        
        if self.initialized.load(Ordering::Acquire) {
            (*self.inner.get()).dealloc(ptr, layout);
        }
    }
}


// Initialisation de l'allocateur global avec une région de mémoire simulé
#[cfg(feature = "alloc")]
#[test_case] // utilisé dans des environnement kernel ou systeme embarqué pour dire a cargo test d'exécuté le test unitaire
fn init_global_alloc_for_tests() {
    static mut TEST_MEMORY: [u8; 1024 * 1024] = [0; 1024 * 1024];
    unsafe {
        let memory_ptr = TEST_MEMORY.as_ptr() as usize;
        init_global_allocator(memory_ptr, TEST_MEMORY.len());
    }
}

/// les tests
#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::Layout;

    // Une Structure pour tester l'allocation complexe
    struct TestStruct {
        a: u32,   // grosse structure
        b: u64,   // alignement différents, peut forcer l'alignement global à 8
        c: [u8; 32],   // tableau interne, ça augmente sa taille
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
