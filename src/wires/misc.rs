#![feature(const_generics)]

// use typenum::{Sum, Unsigned, U7};
use core::marker::PhantomData;
use core::convert::TryInto;

type BitCountType = u32;

const_assert!(label; std::mem::size_of::<usize>() >= std::mem::size_of::<BitCountType>());

// const fn num_bytes(b: BitCountType) -> usize {
//     // let bits: usize = ;
//     (b as usize + 7) / 8
// }

// // Construct a `Bits` using anything other than this function and we can't save you.
// pub fn new<const B: BitCountType>() -> Bits<{B}, {num_bytes(B)}> {
//     Bits {
//         repr: [0u8; num_bytes(B)]
//     }
// }

// pub struct Bits<const B: BitCountType, const S: usize> {
//     repr: [u8; S]
// }

// impl<const B: BitCountType,const S: usize> Bits<{B}, {S}> {
//     // This should work but ICEs:
// /*    const fn new<I: Into<BitCountType>>(input: I) -> Result<Bits<{B}>, IntoBitsError> {
//         Some(Bits {
//             repr: std::mem::transmute::<[u8; std::mem::size_of::<BitCountType>()], [u8; num_bytes(B)]>(input.into().to_le_bytes())/*.try_into().expect("")*/,
//         })
//     }
// */
//     const fn get_bit<const N: usize>(self) -> u8 {
//         // Since we can't impose const numeric bounds this can panic!
//         self.repr[N / 8] >> (N % 8) % 1
//     }
// }

// const fn num_bytes(b: BitCountType) -> usize {
//     // let bits: usize = ;
//     (b as usize + 7) / 8
// }

// // Construct a `Bits` using anything other than this function and we can't save you.
// pub fn new<const B: BitCountType>() -> Bits<{B}, {num_bytes(B)}> {
//     Bits {
//         repr: [0u8; num_bytes(B)]
//     }
// }

// pub struct Bits<const B: BitCountType> {
//     repr: [u8; (B as usize + 7) / 8]
// }

// impl<const B: BitCountType> Bits<{B}> {
//     fn new() -> Self {
//         Bits {
//             repr: [0u8; (B as usize + 7) / 8]
//         }
//     }
// }

// impl<const B: BitCountType,const S: usize> Bits<{B}, {S}> {
//     // This should work but ICEs:
//     const fn new<I: Into<BitCountType>>(input: I) -> Result<Bits<{B}>, IntoBitsError> {
//         Some(Bits {
//             repr: std::mem::transmute::<[u8; std::mem::size_of::<BitCountType>()], [u8; num_bytes(B)]>(input.into().to_le_bytes())/*.try_into().expect("")*/,
//         })
//     }

//     const fn get_bit<const N: usize>(self) -> u8 {
//         // Since we can't impose const numeric bounds this can panic!
//         self.repr[N / 8] >> (N % 8) % 1
//     }
// }


// // Attempt 3:
// struct Bits<const B: BitCountType, const S: usize> {
//     repr: [u8; S]
// }

// impl<const B: BitCountType, const S: usize> Bits<{B}, {S}> {
//     fn new() -> Self {
//         Bits {
//             repr: [0u8; S]
//         }
//     }
// }


// use typenum::{Sum, Unsigned, U7};

// const fn num_bits(inp: usize) -> usize {
//     8
// }

// struct Bits<T: typenum::marker_traits::Unsigned, const B: usize>
// {
//     repr: [u8; /*std::mem::size_of::<[u8; B::to_usize()]>()*/ B ],
//     _rat: PhantomData<T>
// }

// impl<T: typenum::marker_traits::Unsigned, const B: usize> Bits<T, {B}> {
//     fn new() -> Self {
//         Bits {
//             repr: [0u8; T::to_usize()],
//             _rat: PhantomData
//         }
//     }
// }


// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn sanity() {
//         Bits::<{8}>::new();
//     }
// }
