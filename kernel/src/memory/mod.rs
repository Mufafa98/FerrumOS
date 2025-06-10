//! Memory management module
use limine::memory_map::{Entry, EntryType};
use limine::request::MemoryMapRequest;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};
#[used]
#[link_section = ".requests"]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();
use lazy_static::lazy_static;
lazy_static! {
    static ref MEMORY_REGIONS: &'static [&'static Entry] =
        MEMORY_MAP_REQUEST.get_response().unwrap().entries();
}
/// Initialize a new OffsetPageTable
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}
/// Returns a mutable reference to the active level 4 table
/// # Safety
/// This function is unsafe because the caller must guarantee that the physical memory offset is correct.
/// Otherwise, undefined behavior can happen.
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    // read the current CR3 register which contains the address of the level 4 table
    let (level_4_table_frame, _) = Cr3::read();
    // get the physical address from the frame
    let phys = level_4_table_frame.start_address();
    // calculate the virtual address by adding the physical offset
    let virt = physical_memory_offset + phys.as_u64();
    // cast the virt address to a mutable pointer to a page table
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    // return a mutable reference to it
    &mut *page_table_ptr
}
/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    // memory_map: &'static [&'static Entry],
    next: usize,
}
impl BootInfoFrameAllocator {
    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = MEMORY_REGIONS.iter();
        // let temp = regions.clone();
        // for region in temp.into_iter() {
        //     let tp = match region.entry_type {
        //         EntryType::USABLE => "USABLE",
        //         EntryType::RESERVED => "RESERVED",
        //         EntryType::ACPI_RECLAIMABLE => "ACPI_RECLAIMABLE",
        //         EntryType::ACPI_NVS => "ACPI_NVS",
        //         EntryType::BAD_MEMORY => "BAD",
        //         EntryType::BOOTLOADER_RECLAIMABLE => "BOOTLOADER_RECLAIMABLE",
        //         EntryType::KERNEL_AND_MODULES => "KERNEL_AND_MODULES",
        //         EntryType::FRAMEBUFFER => "FRAMEBUFFER",
        //         _ => "UNKNOWN",
        //     };
        //     serial_println!(
        //         "region: {:x} entry type: {:?} len: {:?}",
        //         region.base,
        //         tp,
        //         region.length
        //     );
        // }

        let usable_regions = regions.filter(|r| r.entry_type == EntryType::USABLE);

        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.base..(r.base + r.length));
        // crate::serial_println!(
        //     "addr_ranges: {:?}",
        //     addr_ranges.map(|x| x.start + " " + x.end)
        // );
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // serial_println!(
        //     "frame_addresses: {:?}",
        //     frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
        // );
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
    /// Create a new FrameAllocator from the passed memory map.
    pub unsafe fn init() -> Self {
        BootInfoFrameAllocator {
            // memory_map: MEMORY_REGIONS.,
            next: 0,
        }
    }
}
/// Implement the FrameAllocator trait for BootInfoFrameAllocator
///
/// This implementation is unsafe because the caller must guarantee that the allocator is used correctly.
/// Otherwise, undefined behavior can happen.
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

// TO DO : This implementation is not quite optimal since it recreates the usable_frame allocator on every allocation. It would be better to directly store the iterator as a struct field instead. Then we wouldn’t need the nth method and could just call next on every allocation. The problem with this approach is that it’s not possible to store an impl Trait type in a struct field currently. It might work someday when named existential types are fully implemented. (https://github.com/rust-lang/rfcs/pull/2071)
/// Create an example mapping for the given page to frame 0xb8000.
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;
    // create a frame for the 0xb8000 VGA buffer
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    // set up flags for the mapping (writable and present)
    let flags = Flags::PRESENT | Flags::WRITABLE;
    // map the page to the frame
    let map_to_result = unsafe {
        // FIXME: this is not safe, we do it only for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}
