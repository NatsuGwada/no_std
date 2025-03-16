#![no_std]
#![no_main]

extern crate slab_allocator;

use core::panic::PanicInfo;  // information sur les kernel panik
use slab_allocator::init_global_allocator;  //initialise le global allocator pour utilisé mon slab


#[repr(align(4096))]
struct MemoryRegion([u8; 1024 * 1024]); // 1MB

static mut MEMORY: MemoryRegion = MemoryRegion([0; 1024 * 1024]);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialiser l'allocateur avec une région de mémoire hypothétique
    unsafe {
        let addr = &mut MEMORY.0 as *mut u8 as usize;
        init_global_allocator(addr, 1024 * 1024);
    }
    
    // Juste une boucle pour que le programme ne se termine pas
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
