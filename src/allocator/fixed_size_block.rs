//! A fixed size block allocator.
use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{
    mem,
    ptr::{self, NonNull},
};
/// A node in the fixed size block allocator
struct ListNode {
    next: Option<&'static mut ListNode>,
}
/// Supported block sizes
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];
/// The fixed size block allocator
pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap, // TO DO : modify this to something made by you
}

impl FixedSizeBlockAllocator {
    /// Create a new empty FixedSizeBlockAllocator
    pub const fn new() -> Self {
        // This is needed because we can't use Copy trait on Option<&'static mut ListNode>
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }
    /// Initialize the allocator with the given heap bounds
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }
    /// Allocate a block of the given size using the fallback allocator
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}
/// Returns the index of the block size that fits the given layout
fn list_index(layout: &Layout) -> Option<usize> {
    // Get the size of the layout or the alignment, whichever is larger
    let required_block_size = layout.size().max(layout.align());
    // Find the index of the smallest block size that fits the given required block size
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    /// Allocate memory
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get a mutable reference to the allocator
        let mut allocator = self.lock();
        // Get the index of the block size that fits the given layout
        match list_index(&layout) {
            Some(index) => {
                // If the layout fits a block size, get a block from the list
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        // If the list is not empty, take the first block from the list
                        // and update the list head with the next block in the list
                        allocator.list_heads[index] = node.next.take();
                        // Return the block's address
                        node as *mut ListNode as *mut u8
                    }
                    None => {
                        // If the list is empty, allocate a new block
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;
                        // Create a layout for the block and allocate
                        // memory using the fallback allocator
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            None => {
                // If the layout doesn't fit any block size, use the fallback allocator
                allocator.fallback_alloc(layout)
            }
        }
    }
    /// Deallocate memory
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Get a mutable reference to the allocator
        let mut allocator = self.lock();
        // Get the index of the block size that fits the given layout
        match list_index(&layout) {
            Some(index) => {
                // If the layout fits a block size, deallocate the block

                // Create a new ListNode
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                // Check that the size and alignment of the new node are correct
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                // Write the new node to the block and update the list head
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            None => {
                // If the layout doesn't fit any block size, deallocate using the fallback allocator
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}
