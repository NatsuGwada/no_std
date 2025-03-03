//  Directive(attribut) donnée à rust pour Désactiver la Bibliothèque Standard d'inpout et D'output
#![no_std]

// Structure qui contient les information sur la panic du kernel
use core::panic::PanicInfo;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

// Fonction qui va ignorer les informations de panic du kernel
fn panic(_info: &PanicInfo) -> !{

    loop{}
}


// structure pour l'allocateur slab
pub struct SlabAllocator {

    // mémoire utilisé pour l'allocation
    memory_start: usize,
    memory_size: usize,

    // pointeur vers la prochaine zone mémoire libre
    next_free: usize,

}

// 
