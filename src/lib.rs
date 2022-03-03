mod alloc;
mod bitmap;
mod mem;

pub use alloc::CLAlloc;

const CELL_SIZE: usize = 16;
const PAGE_SIZE: usize = 4096;

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
