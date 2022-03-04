use crate::CELL_SIZE;

#[repr(C)]
pub struct ObjectHeader {
    run_size: u32,
    size: u32,
    ty: usize,
}

#[used]
static ASSERT_HEADER_SIZE: [u8; CELL_SIZE] = [0; std::mem::size_of::<ObjectHeader>()];

impl ObjectHeader {
    pub fn new(ty: usize, size: u32, run_size: u32) -> Self {
        Self { run_size, size, ty }
    }
    pub fn max_size(&self) -> usize {
        self.run_size as usize * CELL_SIZE
    }
    pub fn size(&self) -> usize {
        self.size as usize
    }
    pub fn ty(&self) -> usize {
        self.ty
    }
    pub fn set_size(&mut self, size: u32) {
        assert!(size as usize <= self.max_size());
        self.size = size
    }
}
