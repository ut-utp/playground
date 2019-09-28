use core::ops::DerefMut;
use core::ops::Deref;
use core::ops::Range;
use core::ops::Index;
use crate::util::ConstU8Arr;
use core::slice::SliceIndex;

use super::{BitCountType, Wire};


/// An array of ASCII characters (`u8`s) containing the formatted representation
/// of a wire.
///
/// Arrays larger than 32 elements do not implement traits, so we can't have
/// impls of our `Fmt` trait return `[u8; B]`. Instead we need a glorified
/// newtype around our const generic array upon which we _can_ implement traits.
/// We already have one such type (`ConstU8Arr`) but it (intentionally) isn't
/// public, so we're going to wrap it (a glorified newtype) in another newtype
/// that _is_ public.
pub struct FmtArr<const B: BitCountType> {
    arr: ConstU8Arr<{B}>, // TODO: when const generics are better case this to
                          // to usize rather than trusting BitCountType = usize.
}

impl<const B: BitCountType> Deref for FmtArr<{B}> {
    type Target = [u8; B];

    fn deref(&self) -> &[u8; B] {
        self.arr.deref()
    }
}

impl<const B: BitCountType> DerefMut for FmtArr<{B}> {
    fn deref_mut(&mut self) -> &mut [u8; B] {
        self.arr.deref_mut()
    }
}

impl<const B: BitCountType> AsRef<[u8]> for FmtArr<{B}> {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}

impl<const B: BitCountType> FmtArr<{B}> {
    fn new() -> Self {
        Self {
            arr: ConstU8Arr::<{B}>::new()
        }
    }
}

impl<const B: BitCountType> FmtArr<{B}> {
    fn as_str(&self) -> &str {
        core::str::from_utf8(&**self).unwrap()

        // Or:
        // unsafe { core::str::from_utf8_unchecked(&**self) }
    }
}

impl<'a, const B: BitCountType> From<&'a FmtArr<{B}>> for &'a str {
    fn from(fmt: &FmtArr<{B}>) -> &str {
        fmt.as_str()
    }
}

// Does not work!!!
// pub trait FmtLen {
//     const LEN: usize;
// }

// pub trait FmtAlt: FmtLen {
//     fn fmt(&self) -> [u8; <Self as FmtLen>::LEN];
// }

pub trait Fmt/*<'a>*/ {
    // type Output: SliceIndex<usize, Output = u8>;
    // type Output where usize: SliceIndex<Self::Output, Output = u8>; // GATs!!
    // type Output: Index<Range<usize>, Output = u8>;
    // type Output: AsRef::<[u8]>/* + core::array::LengthAtMost32*/;
    // type Output: Index<usize, Output = u8>;
    // type Output: Into::<&'a[u8]>;
    // const type Len: usize; // Associated const generics!?

    // Same as the above thing (separate out the constant into another trait):
    // does not work!!
    // const LEN: usize;
    // fn fmt(&self) -> [u8; <Self as Fmt>::LEN];

    type Output: AsRef::<[u8]>;

    fn fmt(&self) -> Self::Output;
}

// const fn foo(u: BitCountType) -> usize {
//     u as usize
// }

impl</*'a, */const B: BitCountType, const S: usize> Fmt/*<'a>*/ for Wire<{ B }, { S }> {
    // type Output = [u8; S]; // LengthAtMost32 strikes again
    // type Output = &'a [u8];
    // type Output = ConstU8Arr<{B}>;
    type Output = FmtArr<{B}>;

    fn fmt(&self) -> Self::Output {
        let bits: FmtArr::<{B}> = FmtArr::<{B}>::new();

        bits
    }
}


// use core::fmt::Debug;

// Debug, Display, Binary, Octal, LowerHex, UpperHex
