//  Directive(attribut) donnée à rust pour Désactiver la Bibliothèque Standard d'inpout et D'output
#![no_std]

// Structure qui contient les information sur la panic du kernel
use core::panic::PanicInfo;


#[panic_handler]   // Directive donné à  rust pour utilisé  la fonction lorsqu'un panic se produit
// Fonction qui va ignorer les informations de panic du kernel
fn panic(_info: &PanicInfo) -> !{

    if let Some(location) = _info.location() {
        let file = location.file();
        let line = location.line();

    }
    
    loop{}


}


