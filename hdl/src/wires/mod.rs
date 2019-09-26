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

use core::mem::{size_of, MaybeUninit};

use conversions::IntoBits;
use core::convert::TryInto;
use crate::util::ConstU8Arr;

/// The type used to count the number of bits a wire contains.
///
/// It doesn't _really_ matter what unsigned integer type is chosen here since,
/// in practice, it's extremely unlikely we'll have wires enough bits to exceed
/// even a `u8` (255 bits).
///
/// All the same, even though we'll likely never reach this limit, I'd rather
/// not have it vary per platform which is why we're not just using `usize`
/// here.
pub type BitCountType = u32;
const BIT_COUNT_MAX: BitCountType = core::u32::MAX;

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
/// Equivalently, add 7 and divide by 8 (truncating or flooring):
const fn num_bytes(bits: BitCountType) -> usize {
    //! To be thorough, we'll use checked operations even though the only real
    //! danger is the add potentially overflowing.
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

impl<const B: BitCountType, const S: usize> Wire<{B}, {S}> {
    #[inline]
    pub fn new() -> Self {
        Wire {
            // 0s for our u8 arrays are a valid initialized state and not UB.
            #![allow(unsafe_code)]
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

    #[inline]
    pub fn set<C: IntoBits>(&mut self, val: C) -> &Self {
        // Make sure our number of bytes matches our number of bits:
        debug_assert!(S == num_bytes(B));

        // Check that the value we're trying to represent fits in the number of
        // bits we've got:
        debug_assert!(B >=
            ((<usize as TryInto<BitCountType>>::try_into(C::BYTES).unwrap() * 8)
             - (val.num_leading_zeros()))
        );
        // debug_assert!(B >= ((C::BYTES * 8) - (val.num_leading_zeros().try_into().unwrap())));
        // debug_assert!(B >= ((C::BYTES * 8) - (val.num_leading_zeros().try_into().unwrap())));
        // debug_assert!(S >= C::BYTES);

        let bytes = val.le_bytes();
        self.repr.copy_from_slice(&bytes[0..S]);

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
    fn get_bytes</*T: core::marker::Sized, */const U: usize>(&self) -> [u8; U] {
        // let mut bytes = [0u8; core::mem::size_of::<T>()];
        let mut bytes = ConstU8Arr::<{U}>::new();

        bytes[(U-S)..U].copy_from_slice(&self.repr);
        *bytes
    }

    // fn new_with_inference() -> Self
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! new_wire {
        ($bits:expr) => {Wire::<{ $bits }, { num_bytes($bits) }>::new()};
    }

    // For some reason this fails to compile:
    // Update: it's because calls to `Wire::new_with_val` fail to compile (just like
    // `Wire::get_bytes`). The below works so we'll use it.
    // macro_rules! new_wire_with_val {
    //     ($bits:expr, $val:expr) => {Wire::<{ $bits }, { num_bytes($bits) }>::new_with_val($val)};
    // }

    macro_rules! new_wire_with_val {
        ($bits:expr, $val:expr) => {new_wire!($bits).set($val)};
    }

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
        new_wire_with_val!(0, 0usize);
        new_wire_with_val!(1, 1usize);
        new_wire_with_val!(2, 2usize);
        new_wire_with_val!(2, 3usize);
        new_wire_with_val!(3, 4usize);
        new_wire_with_val!(4, 15usize);
        new_wire_with_val!(32, 4_294_967_295usize);
        new_wire_with_val!(33, 4_294_967_296usize);
    }

    #[test]
    #[should_panic]
    fn not_enough_bits_1() {
        new_wire_with_val!(0, 1usize);
    }

    #[test]
    #[should_panic]
    fn not_enough_bits_2() {
        new_wire_with_val!(4, 16usize);
    }

    // #[test]
    // fn new_alias() {
    //     WireAlias::<{1}>::new();
    // }
}
