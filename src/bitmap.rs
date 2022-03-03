use std::{alloc::Layout, fmt::Write};

use somok::Somok;

use crate::{mem::Mem, round_up, CELL_SIZE};
pub struct GcBitmap {
    start: *mut usize,
    size: usize,
    heap_start: usize,
    heap_end: usize,
}

impl std::fmt::Debug for GcBitmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = String::with_capacity(self.size * 64);
        for i in 0..self.size {
            let bitmap = unsafe { self.start.add(i).read() };
            writeln!(map, "\t{bitmap:064b}")?;
        }
        writeln!(f, "GcBitmap {{\nsize: {},\nmap: {}}}", self.size, map)
    }
}

impl GcBitmap {
    /// Creates a new `GcBitmap` for a given `Mem`
    pub fn new(heap: &Mem) -> Self {
        let size = round_up(heap.size() / CELL_SIZE, 64) / 64;
        let start = unsafe {
            std::alloc::alloc_zeroed(Layout::array::<usize>(size).expect("Invalid layout")) as _
        };
        Self {
            start,
            size,
            heap_start: heap.start() as usize,
            heap_end: heap.end() as usize,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }
    pub fn _index(&self, index: usize) -> Option<usize> {
        if index < self.size {
            unsafe { self.start.add(index).read().some() }
        } else {
            None
        }
    }
    pub unsafe fn index_unchecked(&self, index: usize) -> usize {
        self.start.add(index).read()
    }

    /// Sets bit corresponding to given pointer. Panics if pointer does not belong in [heap_start..heap_end)
    pub fn set<const SET: bool>(&mut self, obj: *const u8) {
        let addr = obj as usize;
        if !(addr >= self.heap_start && addr < self.heap_end) {
            panic!("This pointer does not belong to this bitmap")
        }
        let offset = addr - self.heap_start;
        let index = Self::offset_to_index(offset);
        let mask = Self::offset_to_mask(offset);
        let mut bitmap = unsafe { self.start.add(index).read() };
        if SET {
            bitmap |= mask;
        } else {
            bitmap &= !mask;
        }
        unsafe {
            self.start.add(index).write(bitmap);
        }
    }

    pub fn get(&self, obj: *const u8) -> bool {
        let addr = obj as usize;
        let offset = addr - self.heap_start;
        let index = Self::offset_to_index(offset);
        let mask = Self::offset_to_mask(offset);
        let bitmap = unsafe { self.start.add(index).read() };

        bitmap & mask != 0
    }

    pub fn set_all(&mut self) {
        for index in 0..self.size {
            unsafe {
                self.start.add(index).write(usize::MAX);
            }
        }
    }

    pub fn bit_index_to_offset(index: usize) -> usize {
        index * CELL_SIZE
    }
    pub fn offset_to_index(offset: usize) -> usize {
        (offset / CELL_SIZE) / 64
    }
    pub fn offset_bit_index(offset: usize) -> usize {
        (offset / CELL_SIZE) % 64
    }
    pub fn offset_to_mask(offset: usize) -> usize {
        1 << Self::offset_bit_index(offset)
    }
}
