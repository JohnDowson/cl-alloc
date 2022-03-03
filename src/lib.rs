mod alloc;
mod bitmap;
mod mem;
mod object;

pub use alloc::CLAlloc;
pub use object::ObjectHeader;

pub const CELL_SIZE: usize = 16;
pub const PAGE_SIZE: usize = 4096;

pub fn round_up(n: usize, m: usize) -> usize {
    if m == 0 {
        n
    } else {
        let rem = n % m;
        if rem == 0 {
            n
        } else {
            n + m - rem
        }
    }
}
