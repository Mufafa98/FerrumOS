// TO DO:
// improve fragmentation by keeping the list sorted and merge
// adjiacent blocks of free memory
//
// maybe implement the list as a tree? for faster access to the
// right memory

//! Linked list allocator implementation

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};
/// A node in the linked list
struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    /// Create a new ListNode
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }
    /// Get the start address of the node
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
    /// Get the end address of the node
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}
/// The linked list allocator
pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    /// Create a new empty LinkedListAllocator
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }
    /// Initialize the allocator with the given heap bounds
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the given heap bounds are
    /// valid and that the memory in the heap bounds is unused.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size)
    }
    /// Add a free region to the list
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the given address and size
    /// form a valid region of memory that is not used.
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // ensure that the address is aligned
        assert_eq!(
            align_up(addr, align_up(addr, mem::align_of::<ListNode>())),
            addr
        );
        // ensure that a ListNode fits into the region
        assert!(size >= mem::size_of::<ListNode>());
        // create a new ListNode
        let mut node = ListNode::new(size);
        // place the new node at the start of the free region
        // and update the list's head
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr)
    }
    /// Find a free region that can hold a block of the given size and alignment
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        // Store the current node in a mutable reference
        let mut current = &mut self.head;
        // Iterate over the list
        while let Some(ref mut region) = current.next {
            // Try to allocate from the region
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                // If the region can hold the block, return it
                // and remove it from the list by updating the current node's next pointer
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                // If the region cannot hold the block, continue with the next region
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }
    /// Try to allocate from a region
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        // Calculate the aligned start address of the allocation
        let alloc_start = align_up(region.start_addr(), align);
        // Calculate the end address of the allocation and check if it is within the region
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;
        // Check if the allocation fits into the region
        if alloc_end > region.end_addr() {
            return Err(());
        }
        // Calculate the size of the excess memory and check if it fits a ListNode
        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            return Err(());
        }

        Ok(alloc_start)
    }
    /// Get the size and alignment of the given layout
    fn size_align(layout: Layout) -> (usize, usize) {
        // Calculate the layout size and align it to the alignment of ListNode
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        // Calculate the size of the layout and ensure that it is at least the size of ListNode
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    /// Allocate memory
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get the size and alignment of the layout
        let (size, align) = LinkedListAllocator::size_align(layout);
        // Get a mutable reference to the allocator
        let mut allocator = self.lock();
        // Try to find a free region that can hold the block
        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            // If a region was found, calculate the end address of the allocation
            // and add the excess size
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            // If there is excess size, add a new free region
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            // If no region was found, return null pointer
            ptr::null_mut()
        }
    }
    /// Deallocate memory
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Get the size and alignment of the layout
        let (size, _) = LinkedListAllocator::size_align(layout);
        // Get a mutable reference to the allocator and add a new free region
        self.lock().add_free_region(ptr as usize, size)
    }
}
