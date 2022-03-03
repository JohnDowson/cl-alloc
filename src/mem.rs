use std::ptr::null_mut;

use crate::{round_up, PAGE_SIZE};

#[derive(Debug)]
pub struct Mem {
    start: *mut libc::c_void,
    size: usize,
}

impl Mem {
    pub fn new(size: usize) -> Self {
        let size = round_up(size, PAGE_SIZE);
        let mem = unsafe {
            libc::mmap(
                null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            )
        };
        unsafe { libc::madvise(mem, size, libc::MADV_SEQUENTIAL) };
        if mem == libc::MAP_FAILED {
            panic!("mmap failed");
        }
        unsafe {
            std::ptr::write_bytes(mem, 0, size);
        }
        Self { start: mem, size }
    }

    pub fn start(&self) -> *mut u8 {
        self.start as _
    }
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn end(&self) -> *mut u8 {
        unsafe { (self.start as *mut u8).add(self.size) }
    }

    pub fn decommit(&self, page: *mut u8, size: usize) {
        unsafe {
            libc::madvise(page as *mut _, size as _, libc::MADV_DONTNEED);
        }
    }
    pub fn commit(&self, page: *mut u8, size: usize) {
        unsafe {
            libc::madvise(
                page as *mut _,
                size as _,
                libc::MADV_WILLNEED | libc::MADV_SEQUENTIAL,
            );
        }
    }
}

impl Drop for Mem {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.start, self.size);
        }
    }
}
