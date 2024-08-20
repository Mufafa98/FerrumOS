// TO DO : Fix this test case many_boxes_long_lived
//! Bump allocator implementation

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;
/// A simple allocator that allocates memory linearly
///
/// This allocator is very simple but inefficient. It allocates memory linearly by simply bumping
/// a pointer to the next free address. It only frees all memory at once when the `dealloc` function
/// is called. It is very fast for allocating memory, but it is inefficient in terms of memory usage.

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Create a new empty bump allocator
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }
    /// Initialize the bump allocator with the given heap bounds
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the given heap bounds are
    /// valid and that the memory in the heap bounds is unused.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    /// Allocate memory
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the memory is not used
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get a mutable reference to the bump allocator
        let mut bump = self.lock();
        // Get the allocation start address by aligning the next address
        let alloc_start = align_up(bump.next, layout.align());
        // Get the allocation end address by adding the size to the start address
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };
        // Check if the allocation end address is within the heap bounds
        // If it is, bump the next pointer to the end address and return the start address
        // Otherwise, return null pointer
        // Also, increment the number of allocations, this is used to free all memory at once
        // when dealloc is called and the number of allocations is zero
        if alloc_end > bump.heap_end {
            ptr::null_mut()
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }
    /// Deallocate memory
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Get a mutable reference to the bump allocator
        let mut bump = self.lock();
        // Decrement the number of allocations
        bump.allocations -= 1;
        // Reset the next pointer to the start of the heap
        // only if all allocations have been deallocated
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
