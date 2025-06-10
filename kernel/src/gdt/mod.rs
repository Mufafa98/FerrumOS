//! This module contains the implementation of the Global Descriptor Table (GDT) for x86_64 architecture.
//!
//! The GDT is responsible for defining memory segments and their access permissions.
//! It also includes the Task State Segment (TSS) which holds information about task switching.
use lazy_static::lazy_static;
use x86_64::{
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};
/// Initialize the GDT with the code and TSS segments
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, DS, SS};
    use x86_64::instructions::tables::load_tss;
    // Initialize the GDT
    GDT.0.load();
    unsafe {
        // Reload the code segment register
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        SS::set_reg(GDT.1.data_selector);
        // Load the Task State Segment (TSS)
        load_tss(GDT.1.tss_selector);
    }
}
/// The index of the double fault stack in the Interrupt Stack Table (IST)
///
/// The IST is an array of stack pointers that is used during hardware interrupts
/// to prevent stack overflows
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    /// The Global Descriptor Table (GDT) with the Task State Segment (TSS)
    /// and the selectors for code and TSS segments
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        // Create a new GDT with the code and TSS segments
        let mut gdt = GlobalDescriptorTable::new();
        // Add the code and TSS segments to the GDT
        // The code segment is used for executing code
        // The TSS segment is used for task switching
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let data_selector = gdt.append(Descriptor::kernel_data_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        // TO DO: Remove the following line after checking the code is working
        // let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        // let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                tss_selector,
            },
        )
    };
}
/// Struct used to store the selectors for the code and TSS segments
/// in the GDT
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}
// TO DO : Stack overflow error
// const fn stack_initializer() -> VirtAddr {
// // Set the stack size to 5 pages (5 * 4096 bytes)
// const STACK_SIZE: usize = 4096 * 5;
// // Create a static stack with the specified size
// static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
// #[allow(static_mut_refs)]
// // Get the start and end addresses of the stack
// //
// // The stack is marked as unsafe because it is a static mutable reference
// // and the address of the stack is taken
// let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
// let stack_end = stack_start
//     + STACK_SIZE
//         .try_into()
//         .expect("[Allocator]: Failed to fit usize into u64 in TSS initialization(GDT)");
// stack_end
// }

lazy_static! {
    /// The Task State Segment (TSS) used for task switching
    /// and storing the stack for double faults
    static ref TSS: TaskStateSegment = {
        // Create a new TSS with a stack for double faults
        let mut tss = TaskStateSegment::new();
        // Set the stack for double faults
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] =
            {    // Set the stack size to 5 pages (5 * 4096 bytes)
            const STACK_SIZE: usize = 4096 * 5;
            // Create a static stack with the specified size
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            #[allow(static_mut_refs)]
            // Get the start and end addresses of the stack
            //
            // The stack is marked as unsafe because it is a static mutable reference
            // and the address of the stack is taken
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start
                + STACK_SIZE
                    .try_into()
                    .expect("[Allocator]: Failed to fit usize into u64 in TSS initialization(GDT)");
            stack_end
        };
        tss
    };
}
