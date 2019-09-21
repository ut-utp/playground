use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

pub(crate) struct ConstU8Arr<const SIZE: usize> {
    arr: [u8; SIZE]
}

impl<const SIZE: usize> ConstU8Arr<{SIZE}> {
    pub fn new() -> Self {
        Self {
            // 0s for u8 arrays are a valid initialized state and not UB.
            #![allow(unsafe_code)]
            arr: unsafe { MaybeUninit::zeroed().assume_init() },
        }
    }
}

impl<const SIZE: usize> Deref for ConstU8Arr<{SIZE}> {
    type Target = [u8; SIZE];

    fn deref(&self) -> &[u8; SIZE] {
        &self.arr
    }
}

impl<const SIZE: usize> DerefMut for ConstU8Arr<{SIZE}> {
    fn deref_mut(&mut self) -> &mut [u8; SIZE] {
        &mut self.arr
    }
}
