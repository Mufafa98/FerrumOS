use spin::Mutex;
pub struct MutU8Ptr {
    data: Mutex<*mut u8>,
}
impl MutU8Ptr {
    pub fn new(data: *mut u8) -> Self {
        MutU8Ptr {
            data: Mutex::new(data),
        }
    }

    pub fn add(&self, offset: usize) -> *mut u8 {
        let data = self.data.lock();
        unsafe { data.add(offset) }
    }
}
unsafe impl Sync for MutU8Ptr {}
unsafe impl Send for MutU8Ptr {}
