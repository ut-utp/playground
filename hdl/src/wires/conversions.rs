//! In lieu of numeric bounds on const generics (still waiting on official const
//! generics; see: https://github.com/rust-lang/rfcs/issues/1621) we present
//! this workaround:

use core::ops::IndexMut;
use core::slice::SliceIndex;
use core::ops::Range;
// use core::slice::SliceIndex;
use core::ops::Index;
use super::{num_bytes, BitCountType, Wire};
use repeat_macros::repeat_with_n;

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

// macro_rules! repeated_impl_inner {
//     {$max:expr, $impl_ty:ty, $t:ident $($r:ident)*} => {
//         impl $impl_ty for Wire<{$max}, {num_bytes($max)}> {}

//         repeated_impl_inner!{($max - 1), $impl_ty, $($r)*}
//     };
//     {$max:expr, $impl_ty:ty,} => {
//         impl $impl_ty for Wire<{$max}, {num_bytes($max)}> {}
//     }
// }

// // TODO: finish this vvvvvvv
// macro_rules! repeated_impl {
//     ($max:expr, $impl_ty:ty) => {
//         repeated_impl_inner! {$max, $impl_ty, repeat!{$max, T}}
//     };
// }

// repeated_impl_inner!{128, FitsInU128, T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T}
// repeated_impl_inner!{64, FitsInU64, T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T}
// repeated_impl_inner!{32, FitsInU32, T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T}
// repeated_impl_inner!{16, FitsInU16, T T T T T T T T T T T T T T T T}
// repeated_impl_inner!{8, FitsInU8, T T T T T T T T}

// repeat_with_n!(128, N, impl FitsInU128 for Wire<{N as BitCountType}, {num_bytes(N as BitCountType)}> { });

macro_rules! single_impl {
    ($num:expr, $impl_ty:ty) => {
        impl $impl_ty for Wire<{ $num as BitCountType }, { num_bytes($num as BitCountType) }> {}
    };
}

// macro_rules! impl_size (
//     ($num:expr, $impl_ty:ty) => { repeat_with_n! {$num, N, const A: usize, 8; }; };
// );

// impl_size!(128, FitsInU128);

// macro_rules! impl_sizes (
//     ($num:expr, $($r:expr),*) => {
//         repeat_with_n!($num, N, single_impl!(N, paste::expr! { < > }););

//         impl_sizes!($($r),*);
//     };
//     ($num:expr) => {
//         repeat_with_n!($num, N, single_impl!(N, paste::expr! { < > }););
//     }
// );

// macro_rules! name {
//     ($num:literal) => { repeat_with_n!($num, N, single_impl!(N, FitsInU128);); };
// }

// name!(128);

// impl_sizes!(128, 64, 32, 8);

repeat_with_n!(128, N, single_impl! {N, FitsInU128});
repeat_with_n!(64, N, single_impl! {N, FitsInU64});
repeat_with_n!(32, N, single_impl! {N, FitsInU32});
repeat_with_n!(16, N, single_impl! {N, FitsInU16});
repeat_with_n!(8, N, single_impl! {N, FitsInU8});

// repeat_with_n!(128, N, single_impl!(N, FitsInU128));
// This is dumb and requires nightly since it uses `#![feature(proc_macro_hygiene)]`.
// fn ____() {
//     repeat_with_n!(128, N, { single_impl!(N, FitsInU128); });
//     repeat_with_n!(64,  N, { single_impl!(N, FitsInU64); });
//     repeat_with_n!(32,  N, { single_impl!(N, FitsInU32); });
//     repeat_with_n!(16,  N, { single_impl!(N, FitsInU16); });
//     repeat_with_n!(8,   N, { single_impl!(N, FitsInU8); });
// }
// fn foo() {
//     repeat_with_n!(128, n, println!("{:?}", n));
//     repeat_with_n!(10, Y, {
//             println!("{}", format!("{}", format!("{}{}{}{}", Y, Y, Y, Y)));
//         });
// }

// fn get_wire_bytes()

// macro_rules! impl_for_size {
//     ($type:ty, $marker_trait:path, $enum_variant:path, $nom:expr) => {
//         #[doc = "Wires with 0 to 8 bits (0 to 1 bytes) can be represented by a `"]
//         #[doc = $nom]
//         #[doc = "`."]
//         impl<const B: BitCountType, const S: usize> From<Wire<{ B }, { S }>> for $type
//         where
//             Wire<{ B }, { S }>: $marker_trait,
//         {
//             fn from(wire: Wire<{ B }, { S }>) -> Self {
//                 // This causes an ICE:
//                 // let _bytes = wire.get_bytes::<{core::mem::size_of::<Self>()}>();

//                 // This (which _should_ be identical) does not?
//                 const SIZE: usize = core::mem::size_of::<$type>();
//                 let mut bytes = crate::util::ConstU8Arr::<{SIZE}>::new();

//                 bytes[(SIZE - S)..SIZE].copy_from_slice(&wire.repr);

//                 // There currently doesn't seem to be a way to get the typechecker to
//                 // understand that a const generic value is the same as another constant
//                 // (i.e. size_of::<Self>() == 1 for u8, or even 1 == 1) so we resort to
//                 // using unsafe:
//                 // #[allow(unsafe_code)]
//                 // Self::from_le_bytes(unsafe { core::mem::transmute(bytes) })
//                 Self::from_le_bytes(*bytes)

//                 // let other_bytes = [0u8; core::mem::size_of::<Self>()];
//                 // Self::from_le_bytes(other_bytes)
//             }
//         }

//         // impl From<$type> for IntoBits {
//         //     fn from(num: $type) -> Self {
//         //         $enum_variant(num)
//         //     }
//         // }
//     };

//     ($type:ty, $marker_trait:path, $enum_variant:path) => {
//         impl_for_size!($type, $marker_trait, $enum_variant, stringify!($type));
//     }
// }

macro_rules! impl_for_size {
    ($type:ty, $marker_trait:path, $nom:expr) => {
        #[doc = "Wires with 0 to 8 bits (0 to 1 bytes) can be represented by a `"]
        #[doc = $nom]
        #[doc = "`."]
        impl<const B: BitCountType, const S: usize> From<Wire<{ B }, { S }>> for $type
        where
            Wire<{ B }, { S }>: $marker_trait,
        {
            fn from(wire: Wire<{ B }, { S }>) -> Self {
                // This causes an ICE:
                // let _bytes = wire.get_bytes::<{core::mem::size_of::<Self>()}>();

                // This (which _should_ be identical) does not?
                const SIZE: usize = core::mem::size_of::<$type>();
                let mut bytes = crate::util::ConstU8Arr::<{SIZE}>::new();

                bytes[(SIZE - S)..SIZE].copy_from_slice(&wire.repr);

                // There currently doesn't seem to be a way to get the typechecker to
                // understand that a const generic value is the same as another constant
                // (i.e. size_of::<Self>() == 1 for u8, or even 1 == 1) so we resort to
                // using unsafe:
                // #[allow(unsafe_code)]
                // Self::from_le_bytes(unsafe { core::mem::transmute(bytes) })
                Self::from_le_bytes(*bytes)

                // let other_bytes = [0u8; core::mem::size_of::<Self>()];
                // Self::from_le_bytes(other_bytes)
            }
        }

        impl IntoBits for $type {
            const BYTES: usize = core::mem::size_of::<Self>();
            // type ByteArr = [u8; core::mem::size_of::<Self>()];

            // fn to_le_bytes(&self) -> [u8; Self::BYTES as usize] {
            // fn to_le_bytes(&self) -> Self::ByteArr {
            //     self.to_le_bytes()
            // }

            #[inline]
            fn le_bytes(&self) -> Box<[u8]> {
                Box::new(self.to_le_bytes())
            }

            #[inline]
            fn num_leading_zeros(&self) -> u32 {
                self.leading_zeros()
            }
        }
    };

    ($type:ty, $marker_trait:path) => {
        impl_for_size!($type, $marker_trait, stringify!($type));
    }
}


// pub enum IntoBits {
//     U8(u8),
//     U16(u16),
//     U32(u32),
//     U64(u64),
//     U128(u128),
// }

// pub trait NumBytes {
//     const BYTES: usize;
// }

// impl Index<usize> for [u8; 1] {

// }

pub trait IntoBits/*: NumBytes where Self: NumBytes*/ {
    const BYTES: usize;
    // type ByteArr: SliceIndex<usize, Output = u8>;
    // type ByteArr: Index<usize>;
    // type ByteArr where Range<usize>: SliceIndex<Self::ByteArr, Output = [u8]>;
    // type ByteArr: Index<Range<usize>, Output = [u8]>/* where Range<usize>: SliceIndex<Self::ByteArr, Output=[u8]>*/;
    // type ByteArr: AsRef<[u8]>;

    // type ByteArr where Range<usize>: SliceIndex<Self::ByteArr, Output=[u8]>;
    // type ByteArr: Index<usize, Output = [u8]>;

    // fn to_le_bytes(self) -> [u8; <Self as NumBytes>::BYTES];
    // fn to_le_bytes(self) -> Self::ByteArr;
    // fn to_le_bytes(self) -> crate::util::ConstU8Arr<{<Self as NumBytes>::BYTES}>;
    // fn to_le_bytes(&self) -> Self::ByteArr;
    // fn le_bytes(&self) -> &[u8];
    fn le_bytes(&self) -> Box<[u8]>;
    // fn as_slice(&self, s: usize) -> &[u8] {
    //     // &self.to_le_bytes()[0..s]
    //     (0..s).get(&self.to_le_bytes()).unwrap()
    // }
    fn num_leading_zeros(&self) -> u32;
}

// impl IntoBits for u8 {
//     const BYTES: usize = core::mem::size_of::<Self>();
//     // type ByteArr = [u8; 1];

//     fn le_bytes(&self) -> Box<[u8]> {
//         &self.to_le_bytes()
//     }

//     fn leading_zeros(&self) -> u32 {
//         self.leading_zeros()
//     }
// }

// impl_for_size!(u8, FitsInU8, IntoBits::U8);
// impl_for_size!(u16, FitsInU16, IntoBits::U16);
// impl_for_size!(u32, FitsInU32, IntoBits::U32);
// impl_for_size!(u64, FitsInU64, IntoBits::U64);
// impl_for_size!(u128, FitsInU128, IntoBits::U128);

impl_for_size!(u8, FitsInU8);
impl_for_size!(u16, FitsInU16);
impl_for_size!(u32, FitsInU32);
impl_for_size!(u64, FitsInU64);
impl_for_size!(u128, FitsInU128);

// /// Wires with 0 to 8 bits (0 to 1 bytes) can be represented by a u8.
// impl<const B: BitCountType, const S: usize> From<Wire<{ B }, { S }>> for u8
// where
//     Wire<{ B }, { S }>: FitsInU8,
// {
//     fn from(wire: Wire<{ B }, { S }>) -> Self {
//         let bytes = wire.get_bytes::<{core::mem::size_of::<Self>()}>();

//         // There currently doesn't seem to be a way to get the typechecker to
//         // understand that a const generic value is the same as another constant
//         // (i.e. size_of::<Self>() == 1 for u8, or even 1 == 1) so we resort to
//         // using unsafe:
//         #[allow(unsafe_code)]
//         Self::from_le_bytes(unsafe { core::mem::transmute(bytes) })

//         // bytes[]
//         // for i in 0..S {
//         //     bytes[i] = self.repr
//         // }
//         // bytes[0]
//     }
// }

// /// Wires with 0 to 16 bits (0 to 2 bytes) can be represented by a u16.
// impl<const B: BitCountType, const S: usize> From<Wire<{ B }, { S }>> for u16
// where
//     Wire<{ B }, { S }>: FitsInU16,
// {
//     fn from(wire: Wire<{ B }, { S }>) -> Self {
//         // This works if we're guaranteed to have more bytes than we need; it
//         // breaks when we have fewer.
//         let (bytes, _) = wire.repr.split_at(core::mem::size_of::<Self>());
//         Self::from_le_bytes(bytes.try_into().unwrap())
//     }
// }

// /// Wires with 0 to 32 bits (0 to 4 bytes) can be represented by a u32.
// impl<const B: BitCountType, const S: usize> From<Wire<{ B }, { S }>> for u32
// where
//     Wire<{ B }, { S }>: FitsInU32,
// {
//     fn from(wire: Wire<{ B }, { S }>) -> Self {
//         let bytes = wire.get_bytes::<{core::mem::size_of::<Self>()}>();
//         Self::from_le_bytes(bytes.try_into().unwrap())
//     }
// }

// /// Wires with 0 to 64 bits (0 to 8 bytes) can be represented by a u64.
// impl<const B: BitCountType, const S: usize> From<Wire<{ B }, { S }>> for u64
// where
//     Wire<{ B }, { S }>: FitsInU64,
// {
//     fn from(wire: Wire<{ B }, { S }>) -> Self {
//         let (bytes, _) = wire.repr.split_at(core::mem::size_of::<Self>());
//         Self::from_le_bytes(bytes.try_into().unwrap())
//     }
// }

// /// Wires with 0 to 128 bits (0 to 16 bytes) can be represented by a u128.
// impl<const B: BitCountType, const S: usize> From<Wire<{ B }, { S }>> for u128
// where
//     Wire<{ B }, { S }>: FitsInU128,
// {
//     fn from(wire: Wire<{ B }, { S }>) -> Self {
//         let (bytes, _) = wire.repr.split_at(core::mem::size_of::<Self>());
//         Self::from_le_bytes(bytes.try_into().unwrap())
//     }
// }

// impl FitsInU128 for Wire<{1}, {num_bytes(2)}> {

// }
