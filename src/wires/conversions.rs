//! In lieu of numeric bounds on const generics (still waiting on official const
//! generics; see: https://github.com/rust-lang/rfcs/issues/1621) we present
//! this workaround:

use super::{Wire, BitCountType, num_bytes};
use core::convert::TryInto;

/// Wires with 128 bits or fewer.
pub trait FitsInU128 {}

/// Wires with 64 bits or fewer.
pub trait FitsInU64: FitsInU128 {}

/// Wires with 32 bits or fewer.
pub trait FitsInU32: FitsInU64 {}

/// Wires with 16 bits or fewer.
pub trait FitsInU16: FitsInU32 {}

/// Wires with 8 bits or fewer.
pub trait FitsInU8: FitsInU16 {}

macro_rules! repeated_impl_inner {
    {$max:expr, $impl_ty:ty, $t:ident $($r:ident)*} => {
        impl $impl_ty for Wire<{$max}, {num_bytes($max)}> {}

        repeated_impl_inner!{($max - 1), $impl_ty, $($r)*}
    };
    {$max:expr, $impl_ty:ty,} => {
        impl $impl_ty for Wire<{$max}, {num_bytes($max)}> {}
    }
}

// TODO: finish this vvvvvvv
macro_rules! repeated_impl  {
    ($max:expr, $impl_ty:ty) => {
        repeated_impl_inner!{$max, $impl_ty, repeat!{$max, T}}
    };
}

repeated_impl_inner!{128, FitsInU128, T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T}
repeated_impl_inner!{64, FitsInU64, T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T}
repeated_impl_inner!{32, FitsInU32, T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T}
repeated_impl_inner!{16, FitsInU16, T T T T T T T T T T T T T T T T}
repeated_impl_inner!{8, FitsInU8, T T T T T T T T}

impl<const B: BitCountType, const S: usize> From<Wire<{B}, {S}>> for u8
where Wire<{B}, {S}>: FitsInU8
{
    fn from(wire: Wire<{B}, {S}>) -> Self {
        wire.repr[0]
    }
}

impl<const B: BitCountType, const S: usize> From<Wire<{B}, {S}>> for u16
where Wire<{B}, {S}>: FitsInU16
{
    fn from(wire: Wire<{B}, {S}>) -> Self {
        let (bytes, _) = wire.repr.split_at(std::mem::size_of::<Self>());
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

// impl FitsInU128 for Wire<{1}, {num_bytes(2)}> {

// }
