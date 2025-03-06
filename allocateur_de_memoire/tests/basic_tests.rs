#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate slab_allocator;

use core::panic::PanicInfo;
use core::alloc::{GlobalAlloc, Layout};

// Zone mémoire statique pour nos tests
static mut TEST_MEMORY: [u8; 4096] = [0; 4096];

#[test_case]
fn test_small_alloc() {
    let memory_start = unsafe { TEST_MEMORY.as_ptr() as usize };
    let mut allocator = unsafe {
        let mut alloc = slab_allocator::SlabAllocator::new(memory_start, 4096);
        alloc.init();
        alloc
    };

    // Allouer 16 octets
    let layout = Layout::from_size_align(16, 8).unwrap();
    let ptr = unsafe { allocator.alloc(layout) };
    
    assert!(!ptr.is_null(), "L'allocation a échoué");
    
    // Écrire dans la mémoire pour vérifier qu'elle est utilisable
    unsafe {
        for i in 0..16 {
            *ptr.add(i) = i as u8;
        }
        
        // Vérifier qu'on peut lire les valeurs écrites
        for i in 0..16 {
            assert_eq!(*ptr.add(i), i as u8);
        }
    }
    
    // Libérer la mémoire
    unsafe {
        allocator.dealloc(ptr, layout);
    }
}

#[test_case]
fn test_multiple_allocs() {
    // Même principe mais avec plusieurs allocations
    // ...
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
        println!("Test passed!");
    }
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Test failed: {}", info);
    loop {}
}
