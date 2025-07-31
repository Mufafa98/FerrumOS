// Reference https://github.com/danikhan632/ata_x86/blob/main/src/port.rs
// TODO Document this module
use core::arch::asm;
pub trait PortRead {
    unsafe fn read_from_port(port: u16) -> Self;
}
pub trait PortWrite {
    unsafe fn write_to_port(port: u16, value: Self);
}

impl PortRead for u8 {
    unsafe fn read_from_port(port: u16) -> Self {
        let value: u8;
        unsafe {
            asm!(
                "in al, dx",
                out("al") value,
                in("dx") port,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }
}
impl PortRead for u16 {
    unsafe fn read_from_port(port: u16) -> Self {
        let value: u16;
        unsafe {
            asm!(
                "in ax, dx",
                out("ax") value,
                in("dx") port,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }
}
impl PortRead for u32 {
    unsafe fn read_from_port(port: u16) -> Self {
        let value: u32;
        unsafe {
            asm!(
                "in eax, dx",
                out("eax") value,
                in("dx") port,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }
}

impl PortWrite for u8 {
    unsafe fn write_to_port(port: u16, value: u8) {
        unsafe {
            asm!(
                "out dx, al",
                in("dx") port,
                in("al") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}
impl PortWrite for u16 {
    unsafe fn write_to_port(port: u16, value: u16) {
        unsafe {
            asm!(
                "out dx, ax",
                in("dx") port,
                in("ax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}
impl PortWrite for u32 {
    unsafe fn write_to_port(port: u16, value: u32) {
        unsafe {
            asm!(
                "out dx, eax",
                in("dx") port,
                in("eax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}

mod sealed {
    pub trait Access {
        const DEBUG_NAME: &'static str;
    }
}

pub trait PortReadAccess: sealed::Access {}
pub trait PortWriteAccess: sealed::Access {}

pub struct ReadOnlyAccess(());
pub struct WriteOnlyAccess(());
pub struct ReadWriteAccess(());

impl sealed::Access for ReadOnlyAccess {
    const DEBUG_NAME: &'static str = "ReadOnly";
}
impl PortReadAccess for ReadOnlyAccess {}

impl sealed::Access for WriteOnlyAccess {
    const DEBUG_NAME: &'static str = "WriteOnly";
}
impl PortWriteAccess for WriteOnlyAccess {}

impl sealed::Access for ReadWriteAccess {
    const DEBUG_NAME: &'static str = "ReadWrite";
}
impl PortReadAccess for ReadWriteAccess {}
impl PortWriteAccess for ReadWriteAccess {}

pub struct GenericPort<T, A> {
    port: u16,
    phantom: core::marker::PhantomData<(T, A)>,
}

pub type PortReadOnly<T> = GenericPort<T, ReadOnlyAccess>;
pub type PortWriteOnly<T> = GenericPort<T, WriteOnlyAccess>;
pub type Port<T> = GenericPort<T, ReadWriteAccess>;

impl<T, A> GenericPort<T, A> {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            phantom: core::marker::PhantomData,
        }
    }
}
impl<T: PortRead, A: PortReadAccess> GenericPort<T, A> {
    pub unsafe fn read(&self) -> T {
        T::read_from_port(self.port)
    }
}
impl<T: PortWrite, A: PortWriteAccess> GenericPort<T, A> {
    pub unsafe fn write(&self, value: T) {
        T::write_to_port(self.port, value)
    }
}

impl<T, A: sealed::Access> core::fmt::Debug for GenericPort<T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("PortGeneric")
            .field("Port", &self.port)
            .field("Size", &core::mem::size_of::<T>())
            .field("Access", &format_args!("{}", A::DEBUG_NAME))
            .finish()
    }
}

impl<T, A> Clone for GenericPort<T, A> {
    fn clone(&self) -> Self {
        Self {
            port: self.port,
            phantom: core::marker::PhantomData,
        }
    }
}
impl<T, A> PartialEq for GenericPort<T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.port == other.port
    }
}
impl<T, A> Eq for GenericPort<T, A> {}
