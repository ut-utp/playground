use super::{BitCountType, Wire};

use core::ops::{Index, IndexMut};

// Warning! Indexes by byte (not bit). This isn't very useful.
impl<const B: BitCountType, const S: usize> Index<usize> for Wire<{B}, {S}> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.repr[idx]
    }
}

impl<const B: BitCountType, const S: usize> IndexMut<usize> for Wire<{B}, {S}> {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.repr[idx]
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::w;

    #[test]
    fn index_bytes() {
        assert_eq!(w!(1#0)[0], 0u8);
        assert_eq!(w!(8*8)[7], 0u8);
        assert_eq!(w!((8*7)#core::u64::MAX)[7], 255u8);
    }
}
