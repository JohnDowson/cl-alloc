use std::ptr::NonNull;

use crate::{bitmap::GcBitmap, mem::Mem, CELL_SIZE, PAGE_SIZE};

#[derive(Debug)]
pub struct CLAlloc {
    pages: Vec<Page>,
}

impl CLAlloc {
    pub fn new() -> Self {
        Self {
            pages: Default::default(),
        }
    }

    pub fn mark(&mut self, ptr: *const u8) {
        for page in self.pages.iter_mut() {
            if page.mark(ptr) {
                return;
            }
        }
    }

    pub fn sweep(&mut self) {
        for page in self.pages.iter_mut() {
            page.sweep();
        }
        self.pages.retain(|p| !p.is_empty());
    }

    pub fn alloc_page(&mut self) {
        self.pages.push(Page::new())
    }

    pub fn alloc(&mut self, size: usize) -> Option<NonNull<u8>> {
        self.pages.iter_mut().find_map(|p| p.find_free_run(size))
    }
}

impl Default for CLAlloc {
    fn default() -> Self {
        Self::new()
    }
}

struct Page {
    mark: GcBitmap,
    block: GcBitmap,
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
    fn new() -> Self {
        let mem = Mem::new(PAGE_SIZE);
        let mut mark = GcBitmap::new(&mem);
        mark.set_all();
        let block = GcBitmap::new(&mem);
        Self { mark, block, mem }
    }

    fn pointer_belongs(&self, ptr: *const u8) -> bool {
        ptr as usize >= self.mem.start() as usize && ptr as usize <= self.mem.end() as usize
    }

    fn is_empty(&self) -> bool {
        for index in 0..self.block.size() {
            let block = unsafe { self.block.index_unchecked(index) };
            let mark = unsafe { self.mark.index_unchecked(index) };
            if block != 0 || mark != usize::MAX {
                return false;
            }
        }
        true
    }

    fn mark(&mut self, ptr: *const u8) -> bool {
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

    fn sweep(&mut self) {
        // block' = block & mark
        // mark' = block ^ mark
    }

    fn find_free_run(&mut self, size: usize) -> Option<NonNull<u8>> {
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
            for i in (16..size * 16).step_by(16) {
                unsafe {
                    self.block.set::<false>(ptr.add(i));
                    self.mark.set::<false>(ptr.add(i));
                }
            }
            self.mem.commit(ptr, size * 16);

            NonNull::new(ptr)
        }
    }
}

#[test]
fn test() {
    let mut a = CLAlloc::new();
    a.alloc_page();
    a.alloc(8);
    a.alloc(2);
    unsafe {
        let ptr = a.pages[0].mem.start().add(32);
        dbg! { a.mark(ptr) };
    }
    panic! {"{:?}", a};
}
