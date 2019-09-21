use core::convert::TryInto;

// type Bytes<const B: usize> = (B + 7) / 8;

type BitCountType = u32;

// struct Bytes<const B: BitCountType>;
// impl<const B: BitCountType> Bytes<{B}> {
//     const fn num_bytes() -> usize {
//         let bits: usize = B.try_into().unwrap();
//         (bits + 7) / 8
//     }
// }

// const fn num_bytes(b: BitCountType) -> usize {
//     let bits: usize = b.try_into().unwrap();
//     (bits + 7) / 8
// }

pub struct Bits<const B: BitCountType> {
    // repr: [u8; {Bytes<{B}>::num_bytes()}]
    // repr: [u8; num_bytes(B)]
    repr: [u8; (B as usize + 7) / 8]
}

// We don't yet have numeric bounds on const generics (still waiting on official const
// generics) so this kind of thing is not yet legal:
// See: https://github.com/rust-lang/rfcs/issues/1621
// impl<const Current: usize, const Desired: usize> TryFrom<Bits<{Current}>> for Bits<{Desired}>
// where Desired > Current { }

// Note: `const fn const_assert<const N: usize, const B: usize>(n: N, b: B)` doesn't work
// This also does not work:
// const fn const_assert(n: usize, b:usize) {
//     const_assert!(n < b);
// }

enum IntoBitsError {
    ValueTooBigForBits,
    ValueTooSmallForBits
}

impl<const B: BitCountType> Bits<{B}> {
    // This should work but ICEs:
/*    const fn new<I: Into<BitCountType>>(input: I) -> Result<Bits<{B}>, IntoBitsError> {
        Some(Bits {
            repr: std::mem::transmute::<[u8; std::mem::size_of::<BitCountType>()], [u8; num_bytes(B)]>(input.into().to_le_bytes())/*.try_into().expect("")*/,
        })
    }
*/
    const fn get_bit<const N: usize>(self) -> u8 {
        // Since we can't impose const numeric bounds this can panic!
        self.repr[N / 8] >> (N % 8) % 1
    }

    const fn get_sign_bit(self) -> Bits<{1}> {
        Bits {
            repr: [ self.get_bit::<{B - 1}>();(B as usize + 7) / 8 ]
        }
    }
}
