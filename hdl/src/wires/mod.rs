//! Here we define a `Wire` type that's generic over the number of bits the wire
//! contains. The actual `Wire` type is defined here, but the macros meant to be
//! used to actually create and manipulate wires are in the `wire-macros` crate.
//!
//! The `Wire` type is essentially just a packed representation of the bits that
//! a wire contains (`[bool; num_wires]` seems like it would be perfect here but
//! unfortunately this can't be packed: each element must be at least byte
//! aligned -- depending on the architecture -- so that each element has a
//! unique address).

mod conversions;
mod fmt;

use core::mem::{size_of, MaybeUninit};

use crate::util::ConstU8Arr;
use conversions::IntoBits;
use core::convert::TryInto;

/// The type used to count the number of bits a wire contains.
///
/// It doesn't _really_ matter what unsigned integer type is chosen here since,
/// in practice, it's extremely unlikely we'll have wires enough bits to exceed
/// even a `u8` (255 bits).
///
/// All the same, even though we'll likely never reach this limit, I'd rather
/// not have it vary per platform which is why we're not just using `usize`
/// here. Update: we weren't but thanks to const generics limitations we now
/// are! See the docs on `util::ConstU8Arr` (crate public) for more details.
pub type BitCountType = usize;
const BIT_COUNT_MAX: BitCountType = core::usize::MAX;

// TODO: Switch back once const generics are better!
// pub type BitCountType = u32;
// const BIT_COUNT_MAX: BitCountType = core::u32::MAX;

// Again, it's unlikely that we'll ever run into this bound, but if we're ever
// dealing with very large wires it's possible that BitCountType permits a
// number of bits larger than that we can actually store on the machine (this
// is highly unlikely but possible, for example with BitCountType = u16 and an
// eight bit machine).
//
// So, just to be thorough we should check that:
//   `usize::max >= (BitCountType::max / 8)`
//
// If it isn't we'll just throw a compile time error; there isn't really a way
// to recover from your target machine being physically unable to represent
// things.
//
// This first check makes sure we can safely cast a thing of type `BitCountType`
// as a `usize`:
const_assert!(sanity_check; size_of::<usize>() >= size_of::<BitCountType>());

// And this second, now redundant, assert performs the check we wanted in the
// first place:
const_assert!(redundant_check; core::usize::MAX >= BIT_COUNT_MAX as usize / 8);

// Hypothetically, this check is stricter than it needs to be; `usize` can be
// 3 bits smaller (since we're dividing by 8) than `BitCountType` and the first
// of the above checks does not allow this (it errs on the side of being not
// permissive enough). Absent a use case for this, we'll just call this good
// enough and move on.

/// The mapping between number of bits and how many bytes we'll need to
/// represent said number of bits is simple: divide by 8 and take the ceiling.
///
///  bits | bytes
///    0  |   0
///    1  |   1
///    2  |   1
///   ... |  ...
///    7  |   1
///    8  |   1
///    9  |   2
///
/// Equivalently, add 7 and divide by 8 (truncating or flooring).
pub const fn num_bytes(bits: BitCountType) -> usize {
    //! To be thorough, we'll try to use checked operations even though the only
    //! real danger is the add operation potentially overflowing.
    // Hopefully these will be stripped out in most cases, but this needs to be
    // tested (TODO).
    //
    // Also it's worth noting that a side-effect of the `const_assert!` above is
    // that `From<BitCountType>` will always be implemented for `usize` since
    // conversions to larger unsized integer types are infallible and
    // implemented with `From`/`Into` and the assert above guarantees
    // `BitCountType` will be smaller or of the same size.
    //
    // In theory, that is. In practice, the `From` impls don't seem to vary with
    // the size of `usize`. Since we've already checked that `usize` is big
    // enough though, this is safe:

    // This is what we want to be able to do:
    // ```
    // (bits as usize)
    //     .checked_add(7)
    //     .unwrap()
    //     .checked_div(8)
    //     .unwrap()
    // ```
    // But we can't since checked operations are not yet const functions. As
    // mentioned the add is the only potentially problematic operation but only
    // within extremely unlikely scenarios (very very large Wires).
    ((bits as usize) + 7) / 8
}

/// For a given bit (0-indexed), returns the byte index (0-indexed) and the bit
/// number (0-indexed) within said byte.
///
/// The byte index should be in [0, (bit / 8)]
/// The bit number should be in [0, (bit % 8)]
///
///  bit | byte idx, bit idx
///   0  |     0        0
///   1  |     0        1
///   7  |     0        7
///   8  |     1        0
///   9  |     1        1
///  16  |     2        0
const fn byte_and_offset(bit: BitCountType) -> (usize, usize) {
    // As in `num_bytes`, the `as` is safe thanks to the const_assert.

    // This is what we want to be able to do:
    // ```
    // (
    //     (bit as usize).checked_div(8).unwrap(),
    //     (bit as usize).checked_rem(8).unwrap(),
    // )
    // ```
    // But since checked operations aren't const functions yet, we'll just drop
    // the checking (both division and the remainder operation are infallible
    // on unsigned integers).

    ((bit as usize) / 8, (bit as usize) % 8)
}

#[derive(Copy, Clone)]
pub struct Wire<const B: BitCountType, const S: usize> {
    repr: [u8; S],
    // repr: [u8; (B as usize + 7) / 8],
}

// Doesn't work:
// type WireAlias<const B: BitCountType> = Wire<{B}, {num_bytes(B)}>;

// Also doesn't work:
// pub fn new<const B: BitCountType>() -> Wire<{B}, {_}> {
//
// }

// This too:
// pub fn help<const B: BitCountType, const S: usize>(uno: BitCountType, dos: usize) -> Wire<{B}, {S}> {
//     Wire::<{ uno }, { dos }>::new()
// }

impl<const B: BitCountType, const S: usize> Wire<{ B }, { S }> {
    #[inline]
    pub fn new() -> Self {
        // Make sure our number of bytes matches our number of bits:
        debug_assert!(S == num_bytes(B));

        Wire {
            #![allow(unsafe_code)]
            // 0s for our u8 arrays are a valid initialized state and not UB.
            repr: unsafe { MaybeUninit::zeroed().assume_init() },
        }
    }

    // Unfortunately this function doesn't appear to work (ICEs whenever it's actually
    // invoked like `Wire::get_bytes(&self)`). There's a workaround (calling new and
    // then set at the call site) but we'll still leave this function in in-case this
    // works once const generics become more stable.
    #[inline]
    #[allow(unused)]
    fn new_with_val<C: IntoBits>(val: C) -> Self {
        let mut wire = Self::new();
        wire.set(val);

        wire
    }

    /// Set the value of a wire.
    ///
    /// Currently, only unsigned types are accepted.
    ///
    /// For a wire with B bits, we can use values occupying [0, B] bits. If a
    /// value occupying more bits than the wire has is provided this will panic
    /// but _only in debug mode_. If a value has fewer bits than a wire, the
    /// remaining bits will be set to 0.
    #[inline]
    pub fn set<C: IntoBits>(&mut self, val: C) -> &mut Self {
        // Check that the value we're trying to represent fits in the number of
        // bits we've got:
        // N.B. checking the number of leading zeros is a valid way to know how
        // many bits `val` requires since we only accept unsigned types.
        debug_assert!(
            B >= ((<usize as TryInto<BitCountType>>::try_into(C::BYTES).unwrap() * 8)
                - (<u32 as TryInto<BitCountType>>::try_into(val.num_leading_zeros()).unwrap()))
        );
        // debug_assert!(B >= ((C::BYTES * 8) - (val.num_leading_zeros().try_into().unwrap())));
        // debug_assert!(B >= ((C::BYTES * 8) - (val.num_leading_zeros().try_into().unwrap())));
        // debug_assert!(S >= C::BYTES);

        // We have some cases:
        //   - S == C::BYTES: nice and easy; just copy S bytes into `repr`
        //   - S  < C::BYTES, but B >= (C::BYTES * 8) - val.num_leading_zeros():
        //                    This ends up being the same as the first case; we
        //                    just copy the first S bytes over since we've made
        //                    sure that the rest of the bytes are all zero.
        //   - S  > C::BYTES: copy 0..C::BYTES into `repr` and zero C::BYTES..S

        // To make life simpler, we'll just always copy Z bytes and zero Z..S
        // where Z = S in the first and second case and Z = C::BYTES in the
        // third case.

        // This doesn't currently compile (and Ord::min isn't yet a const fn).
        // const Z: usize = { if C::BYTES >= S { S } else { C::BYTES } };

        let z: usize = C::BYTES.min(S);

        self.repr[0..z].copy_from_slice(&val.le_bytes()[0..z]);
        (z..S).for_each(|i| self.repr[i] = 0);

        self
    }

    /// Where S is the number of bytes we have, and U is the number of bytes
    /// we need, the following makes a slice of the last S bytes of U:
    ///   `(U - S)..S`
    ///
    /// Some examples:
    ///   S  |  U  | `(U - S)..U`
    /// --------------------------
    ///   0  |  1  |     1..1
    ///   1  |  1  |     0..1
    ///   0  |  4  |     4..4
    ///   1  |  4  |     3..4
    ///   2  |  4  |     2..4
    ///   3  |  4  |     1..4
    ///   4  |  4  |     0..4
    ///   0  |  8  |     8..8
    ///   8  |  8  |     0..8
    ///
    // Unfortunately this function doesn't appear to work (ICEs whenever it's actually
    // invoked). We found a workaround that we're using for now, but we'll still leave
    // this function in in-case this works once const generics become more stable.
    #[allow(unused)]
    fn get_bytes<const U: usize>(&self) -> [u8; U] {
        // let mut bytes = [0u8; core::mem::size_of::<T>()];
        let mut bytes = ConstU8Arr::<{ U }>::new();

        bytes[(U - S)..U].copy_from_slice(&self.repr);
        *bytes
    }

    // fn new_with_inference() -> Self
}

#[doc(hidden)]
#[macro_export(crate)]
macro_rules! new_wire {
    ($bits:expr) => {
        Wire::<{ $bits }, { $crate::wires::num_bytes($bits) }>::new()
    };
}

// For some reason this fails to compile:
// Update: it's because calls to `Wire::new_with_val` fail to compile (just like
// `Wire::get_bytes`). The below works so we'll use it.
// macro_rules! new_wire_with_val {
//     ($bits:expr, $val:expr) => {Wire::<{ $bits }, { num_bytes($bits) }>::new_with_val($val)};
// }

#[doc(hidden)]
#[macro_export(crate)]
macro_rules! new_wire_with_val {
    ($bits:expr, $val:expr) => {
        {
            let mut w = $crate::new_wire!($bits);
            w.set($val);

            w
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        new_wire!(0);
        new_wire!(1);
    }

    // #[test]
    // fn new_with_val() {
    //     new_wire!(1).set(1usize);
    // }

    #[test]
    fn new_with_val() {
        new_wire_with_val!(0, 0u8);
        new_wire_with_val!(1, 1u8);
        new_wire_with_val!(1, 1u16);
        new_wire_with_val!(1, 1u32);
        new_wire_with_val!(1, 1u64);
        new_wire_with_val!(1, 1u128);
        new_wire_with_val!(1, 1usize);
        new_wire_with_val!(2, 2usize);
        new_wire_with_val!(2, 3usize);
        new_wire_with_val!(3, 4usize);
        new_wire_with_val!(4, 15usize);
        new_wire_with_val!(32, 4_294_967_295usize);
        new_wire_with_val!(33, 4_294_967_296usize);
    }

    #[test]
    fn max_val_testing() {
        new_wire_with_val!(1, 1usize);

        macro_rules! max_test {
            ($type:ty, $max:expr) => {
                new_wire_with_val!((8 * size_of::<$type>()) as BitCountType, $max);
            };
        }

        max_test!(u8, core::u8::MAX);
        max_test!(u16, core::u16::MAX);
        max_test!(u32, core::u32::MAX);
        max_test!(u64, core::u64::MAX);
        max_test!(u128, core::u128::MAX);
        max_test!(usize, core::usize::MAX);
    }

    #[test]
    fn big() {
        new_wire_with_val!(256, 0u8);
        new_wire_with_val!(256, 0u16);
        new_wire_with_val!(256, 0u32);
        new_wire_with_val!(256, 0u64);
        new_wire_with_val!(256, 0u128);
        new_wire_with_val!(256, 0usize);
        new_wire_with_val!(256, core::u8::MAX);
        new_wire_with_val!(256, core::u16::MAX);
        new_wire_with_val!(256, core::u32::MAX);
        new_wire_with_val!(256, core::u64::MAX);
        new_wire_with_val!(256, core::u128::MAX);
        new_wire_with_val!(256, core::usize::MAX);
    }

    #[test]
    fn bigger() {
        new_wire_with_val!(8_192, core::usize::MAX); // 1 KiB on the stack
    }

    #[test]
    fn large() {
        new_wire_with_val!(524_288, core::usize::MAX); // 64 KiB on the stack
    }

    #[test]
    fn larger() {
        new_wire_with_val!(2_097_152, core::usize::MAX); // 256 KiB on the stack
    }

    #[test]
    // This will almost certainly fail since we'd be trying to put a ~512 MB
    // array on the stack ((2 ^ 32) bits / 8 bits to a byte -> 512 MB) so we'll
    // ignore this test:
    #[ignore]
    fn huge() {
        new_wire_with_val!(core::u32::MAX as BitCountType, 0usize);
        new_wire_with_val!(core::u32::MAX as BitCountType, 1usize);
        new_wire_with_val!(core::u32::MAX as BitCountType, core::u8::MAX);
        new_wire_with_val!(core::u32::MAX as BitCountType, core::u16::MAX);
        new_wire_with_val!(core::u32::MAX as BitCountType, core::u32::MAX);
        new_wire_with_val!(core::u32::MAX as BitCountType, core::u64::MAX);
        new_wire_with_val!(core::u32::MAX as BitCountType, core::u128::MAX);
        new_wire_with_val!(core::u32::MAX as BitCountType, core::usize::MAX);
    }

    #[test]
    #[should_panic]
    fn val_with_too_many_bits_1() {
        new_wire_with_val!(0, 1usize);
    }

    #[test]
    #[should_panic]
    fn val_with_too_many_bits_2() {
        new_wire_with_val!(4, 16usize);
    }

    #[test]
    fn val_with_fewer_bits() {
        new_wire_with_val!(16, 16u8);
    }

    #[test]
    fn fewer_bytes_but_enough_bits() {
        // A wire with 42 bits should have 6 bytes.
        assert_eq!(size_of::<Wire::<{ 42 }, { num_bytes(42) }>>(), 6);

        // This tests that we're actually checking the number of bits in our
        // bounds checks and not the number of bytes of the given type. A `u64`
        // will have 8 bytes which would fail a byte size check (6 bytes for a
        // 42 bit wire as above), but checking the number of bits for the value
        // should reveal that the value can indeed (just barely) fit in 42 bits.
        new_wire_with_val!(42, 4398046511103u64); // 2 ^ 42 - 1
    }

    #[test]
    #[should_panic]
    fn fewer_bytes_and_too_many_bits() {
        // Same test as above but right above the boundary (should panic).
        new_wire_with_val!(42, 4398046511103u64 + 1u64); // 2 ^ 42
    }
}
