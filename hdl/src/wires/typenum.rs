
use typenum::{Sum, Unsigned, U7};

const fn num_bits(inp: usize) -> usize {
    8
}

struct Bits<B: typenum::marker_traits::Unsigned>
{
    repr: [u8; std::mem::size_of::<[u8; B::to_usize()]>() ]
}
