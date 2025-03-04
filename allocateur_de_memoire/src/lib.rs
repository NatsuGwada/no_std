#![no_std]
#![cfg_attr(feature = "alloc, featre(allocator_api))]   // activation de l'api d'allocateur si alloc est definie

use core::alloc::{GlobalAlloc, Layout};   // implémentation du trait
use core::ptr;   // manipulation de pointeur
use code::mem;   // la mémoire

// un slab = une unité de mémoire pré-allouée
// Définition de la taille d'un slab
const SLAB_SIZES: [usize; 4] = [16, 32, 64, 128]; // différentes tailles de slab en octets
const MAX_SLABS: usize = 100; // nombre maximum de slabs par taille

/// un bloc de mémoire dans l'allocateur
#[repr(C)]
struct SlabBlock {
  next: *mut SlabBlock, // Pointeur vers le prochain bloc libre
}

/// Notre allocateur de type Slab
pub struct SlabAllocator {
  // la mémoire que l'allocateur va gérer
  memory_start: usize,
  memory_end: usize,

  // Liste de blocs libres pour chaque taille de slab
  free_lists [*mut SlabBlock; SLAB_SIZES.len()]

  // nombre de slabs allouées poru chaque tailles
  allocated_counts: [usize; SLAB_SIZES.len()],
}

impl SlabAllocator {
  /// nouveau allocateur
  pub const fn new(start: usize, size: usize) -> Self {
    SlabAllocator {
      memory_start: start,
      memory_end: start + size,
      free_lists: [ptr::null_mut(); SLAB_SIZES.len()],
      allocated_counts: [0; SLAB_SIZES.len()],
      
    }
  }

  fn find_slab_index(&self, layout: &Layout) -> Option<usize> {
    let required_size = layout.size().max(mem::size_of::<SlabBlock>());

    for (i, &size) in SLAB_SIZES.iter().enumerate()  {
      if size >= required_size && layout.align() <= size {
        return Some(i);
      }
    }
    None // aucune taille de slab Appropriée
    
  }

  /// alloue un nouveau bloc de mémoire pour une taille spécifique
  unsafe fn allocate_more_slab(&mut self, slab_index: usize) -> Result<(), ()> {
    let slab_size = SLAB_SIZES[slab_index];
  

    // Calculer combien de slabs on peut créer
    let slab_to_create = 10; // Créons 10 slabs à la fois
    let required_mémory = slab_size * slabs_to_create;
  
    // Trouver une zone libre dans la mémoire
    let current_alloc = self.memory_start + self.allocated_counts.iter().enumerate()
      .map(|(i, &count) | count * SLAB_SIZES[i])
      .sum::<usize>()
  
    if current_alloc + required_mémory < self.memory_end {
      return Err(());  // plus de mémoire disponible
      
    }
  
    // crée les nouveau slabs et ajouter les à la liste libre
  
    for i in 0..slabs_to_create {
      let block_addr = current_alloc + i * slab_size;
      let block_ptr = block_addr as *mut SlabBlock
    
      // Ajouter à la liste libre
      (*block_ptr).next = self.free_lists[slab_index];
      self.free_lists[slab_index] = block_ptr;
      
    }
  
  
      self.allocated_counts[slab_index] += slabs_to_create;
      Ok(())
  }

}
