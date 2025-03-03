//  Directive(attribut) donnée à rust pour Désactiver la Bibliothèque Standard d'inpout et D'output
#![no_std]

// Structure qui contient les information sur la panic du kernel
use core::panic::PanicInfo;  // depuis la crate "core", on importe PanicInfo
use core::alloc::{GlobalAlloc, Layout}; // importation de Global et Layout depuis core::Alloc (pour la taille et l'alignement des allocations)
use core::ptr; // ça c'est pour manipuler les pointeurs a bas niveau

// Fonction qui va ignorer les informations de panic du kernel
fn panic(_info: &PanicInfo) -> !{

    loop{}
}


// structure pour l'allocateur slab
pub struct SlabAllocator {

    // mémoire utilisé pour l'allocation
    memory_start: usize,  // adresse de debut de la zone mémoire
    memory_size: usize,   //taille de la zone mémoire à gerer

    // pointeur vers la prochaine zone mémoire libre
    next_free: usize,

}

impl SlabAllocator{
    // constructeur pour initialisé l'allocateur
    pub const fn new(start: usize, size: usize) -> SlabAllocator {
        SlabAllocator{
            memory_start: start,
            memory_size: size,
            next_free: start,
        }
    }
}

// Imprémentation du trait GlobalAlloc
