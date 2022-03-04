mod page;

use crate::PAGE_SIZE;

use self::page::Page;
use std::ptr::NonNull;

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

    pub fn mark(&mut self, ptr: *const u8) -> bool {
        for page in self.pages.iter_mut() {
            if page.mark(ptr) {
                return true;
            }
        }
        false
    }

    pub fn sweep(&mut self) {
        for page in self.pages.iter_mut() {
            page.sweep();
        }
        self.pages.retain(|p| !p.is_empty());
    }

    fn _page(&self, index: usize) -> &Page {
        &self.pages[index]
    }

    pub fn alloc_page(&mut self) {
        self.pages.push(Page::new())
    }

    pub fn alloc(&mut self, size: usize) -> Option<NonNull<u8>> {
        assert!(
            size <= PAGE_SIZE,
            "Can't allocate objects larger than PAGE_SIZE: {PAGE_SIZE}"
        );
        let maybe = self.pages.iter_mut().find_map(|p| p.find_free_run(size));
        if maybe.is_none() {
            self.alloc_page();
            self.pages.last_mut().unwrap().find_free_run(size)
        } else {
            maybe
        }
    }
}

impl Default for CLAlloc {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test() {
    let mut a = CLAlloc::new();
    let ptr = a.alloc(8).unwrap().as_ptr();
    let ptr2 = a.alloc(4).unwrap().as_ptr();

    assert_eq!(
        (a._page(0).block.get(ptr), a._page(0).mark.get(ptr)),
        (true, false),
        "Assert pointer is white"
    );
    a.mark(ptr);
    assert_eq!(
        (a._page(0).block.get(ptr), a._page(0).mark.get(ptr)),
        (true, true),
        "Assert pointer is black"
    );

    a.sweep();
    assert_eq!(
        (a._page(0).block.get(ptr), a._page(0).mark.get(ptr)),
        (true, false),
        "Assert pointer is white"
    );

    assert_eq!(
        (a._page(0).block.get(ptr2), a._page(0).mark.get(ptr2)),
        (false, true),
        "Assert pointer is free"
    );

    a.alloc(2);
    assert_eq!(
        (a._page(0).block.get(ptr2), a._page(0).mark.get(ptr2)),
        (true, false),
        "Assert pointer is white"
    );
}
