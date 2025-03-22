// Importation de std uniquement pour les tests
#[cfg(test)]
extern crate std;
extern crate slab_allocator;

use slab_allocator::SlabAllocator;
use core::alloc::{GlobalAlloc, Layout};
use std::println;

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
        let ptr = GlobalAlloc::alloc(&allocator, layout);
        assert!(!ptr.is_null());
        
        // Vérifier l'alignement
        println!("Pointeur alloué: {:p}", ptr);
        assert_eq!((ptr as usize) % layout.align(), 0, "Le pointeur n'est pas aligné correctement");

        // Écrire dans la mémoire allouée
        *(ptr as *mut u32) = 42;
        assert_eq!(*(ptr as *mut u32), 42);

        // Désallouer/Déalloc
        GlobalAlloc::dealloc(&allocator, ptr, layout);
    }
}
