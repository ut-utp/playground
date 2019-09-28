use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut, Index};

/// A stack backed `u8` array.
///
/// The main reason this exists is to work around a temporary limitation of the
/// const generics implementation (described below).
///
/// It's currently possible to declare an array in a struct whose length is
/// determined by a `usize` const generic parameter:
/// ```rust
/// # #![feature(const_generics)]
/// struct Foo<const SIZE: usize> { arr: [u8; SIZE] }
/// ```
///
/// However, actually constructing an instance of this type is tricky. If you
/// try to do it as you'd normally do, you get errors about using generic
/// parameters in an array length (even though array lengths are a place where
/// const expressions can be used):
/// ```rust,compile_fail
/// # #![feature(const_generics)]
/// # struct Foo<const SIZE: usize> { arr: [u8; SIZE] }
/// impl<const SIZE: usize> Foo<{SIZE}> {
///     fn new() -> Self {
///         Self {
///             arr: [u8; SIZE]
///         }
///     }
/// }
/// ```
///
/// So, what to do? This is a [known limitation](https://bit.ly/2nE8fHL). In
/// some cases `Default::default()` may save you. And in others, you may have to
/// resort to `unsafe`.
///
/// We ended up having to use `MaybeUninit::zeroed()` (it's safe for arrays of
/// `u8`s!). Rather than duplicate the unsafe code everywhere, `ConstU8Arr` was
/// created and that's how we got here.
///
///
/// There's a long list of const generic things that don't yet work as you'd
/// expect, but here's a particularly frustrating one: you can't use `as` casts
/// in places where const expressions are expected without running into ICEs.
/// ```rust,compile_fail
/// # #![feature(const_generics)]
/// struct Foo<const SIZE: u32> { arr: [u8; SIZE as usize] }
///
/// // -> error: internal compiler error: src/librustc/ty/subst.rs:597: const
/// //           parameter `SIZE/#0` (Const { ty: u32, val: Param(SIZE/#0) }/0)
/// //           out of range when substituting substs=[]
/// ```
/// This seems to happen anywhere you can specify const generic parameter values
/// except in actual functions? As in, this totally works:
/// ```rust
/// # #![feature(const_generics)]
/// # use core::mem::MaybeUninit;
/// struct Foo<const SIZE: usize> { arr: [u8; SIZE] }
///
/// const Y: u32 = 42;
/// let x: Foo<{Y as usize}> = Foo {
///   arr: unsafe { MaybeUninit::zeroed().assume_init() }
/// };
/// ```
///
/// This isn't a show stopper, but it's definitely an annoyance. Originally
/// [`wires::BitCountType`](../wires/type.BitCountType.html) was `u32` but
/// because the cast example above doesn't work (for formatting wires, we need
/// one character per bit in the output but if we store the number of bits (B)
/// as a `u32` we can't have an array of B characters since array lengths have
/// to be usize and we can't cast!!), `wires::BitCountType` is now `usize`
/// (TODO: switch back when const generics are more complete!).
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

// impl<const SIZE: usize> Index<usize> for ConstU8Arr<{SIZE}> {
//     type Output = u8;

//     fn index(&self, idx: usize) -> &u8 {
//         &self.arr[idx]
//     }
// }

impl<const SIZE: usize> AsRef<[u8]> for ConstU8Arr<{SIZE}> {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}
