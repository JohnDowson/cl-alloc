use crate::{bitmap::GcBitmap, mem::Mem, CELL_SIZE, PAGE_SIZE};
use somok::Somok;
use std::ptr::NonNull;

pub(crate) struct Page {
    pub(crate) mark: GcBitmap,
    pub(crate) block: GcBitmap,
    mem: Mem,
}

impl std::fmt::Debug for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Page")
            .field("mem", &self.mem)
            .field("mark", &self.mark)
            .field("block", &self.block)
            .finish()
    }
}

impl Page {
    pub(crate) fn new() -> Self {
        let mem = Mem::new(PAGE_SIZE);
        let mut mark = GcBitmap::new(&mem);
        mark.set_all();
        let block = GcBitmap::new(&mem);
        Self { mark, block, mem }
    }

    pub(crate) fn pointer_belongs(&self, ptr: *const u8) -> bool {
        ptr as usize >= self.mem.start() as usize && ptr as usize <= self.mem.end() as usize
    }

    pub(crate) fn is_empty(&self) -> bool {
        for index in 0..self.block.size() {
            let block = unsafe { self.block.index_unchecked(index) };
            let mark = unsafe { self.mark.index_unchecked(index) };
            if block != 0 || mark != usize::MAX {
                return false;
            }
        }
        true
    }

    pub(crate) fn mark(&mut self, ptr: *const u8) -> bool {
        if !self.pointer_belongs(ptr) {
            return false;
        }
        let is_header = self.block.get(ptr);

        if is_header {
            self.mark.set::<true>(ptr);
            true
        } else {
            self.mark(unsafe { ptr.sub(CELL_SIZE) })
        }
    }

    pub(crate) fn sweep(&mut self) {
        // block' = block & mark
        // mark' = block ^ mark
        let mut last_header = None;
        for index in (0..self.mem.size() / CELL_SIZE).step_by(CELL_SIZE) {
            let ptr = unsafe { self.mem.start().add(index) };
            let mark = self.mark.get(ptr);
            let block = self.block.get(ptr);
            match (block, mark) {
                // extent
                (false, false) => {
                    if let Some(false) = last_header {
                        self.mark.set::<true>(ptr);
                    } else if last_header.is_none() {
                        panic!("Encountered extent without a header")
                    }
                }
                // free
                (false, true) => (),
                // white
                (true, false) => {
                    self.block.set::<false>(ptr);
                    self.mark.set::<true>(ptr);
                    last_header = false.some()
                }
                // black
                (true, true) => {
                    self.mark.set::<false>(ptr);
                    last_header = true.some()
                }
            }
        }
    }

    pub(crate) fn find_free_run(&mut self, size: usize) -> Option<NonNull<u8>> {
        let mut run_start = 0;
        let mut run_length = 0;
        let mut bit_index = 0;
        for index in 0..self.mark.size() {
            if run_length == size {
                break;
            }
            let (mark, block) = unsafe {
                (
                    self.mark.index_unchecked(index),
                    self.block.index_unchecked(index),
                )
            };
            for offset in 0..64 {
                if run_length == size {
                    break;
                }
                let mark_bit = (mark >> offset) & 1;
                let block_bit = (block >> offset) & 1;
                if mark_bit == 1 && block_bit == 0 {
                    if run_length == 0 {
                        run_start = bit_index
                    }
                    run_length += 1;
                } else {
                    run_length = 0
                }
                bit_index += 1;
            }
        }
        if run_length != size {
            None
        } else {
            let offset = GcBitmap::bit_index_to_offset(run_start);
            let ptr = unsafe { self.mem.start().add(offset) };
            debug_assert!(self.pointer_belongs(ptr));

            self.block.set::<true>(ptr);
            self.mark.set::<false>(ptr);
            for i in (CELL_SIZE..size * CELL_SIZE).step_by(CELL_SIZE) {
                unsafe {
                    self.block.set::<false>(ptr.add(i));
                    self.mark.set::<false>(ptr.add(i));
                }
            }
            self.mem.commit(ptr, size * CELL_SIZE);

            NonNull::new(ptr)
        }
    }
}
