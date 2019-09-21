#![recursion_limit="256"]
#![allow(incomplete_features)]
#![feature(const_generics/*, maybe_uninit_slice*/)]
#![feature(const_fn)]

#![no_std]

#![deny(bad_style,
       const_err,
       dead_code,
       improper_ctypes,
       legacy_directory_ownership,
       non_shorthand_field_patterns,
       no_mangle_generic_items,
       overflowing_literals,
       path_statements ,
       patterns_in_fns_without_body,
       plugin_as_library,
       private_in_public,
       safe_extern_statics,
       unconditional_recursion,
       unions_with_drop_fields,
       unused,
       unused_allocation,
       unused_lifetimes,
       unused_comparisons,
       unused_parens,
       while_true)]

#![deny(missing_debug_implementations,
       missing_docs,
       unsafe_code,
       trivial_casts,
       trivial_numeric_casts,
       unused_extern_crates,
       unused_import_braces,
       unused_qualifications,
       unused_results)]

#[macro_use]
extern crate static_assertions;

// Some old notes:

// The macro way:
// generic s-ext and z-ext functions
//
// everything in a proc macro block (`design! {}`)
// within the block, functions annotated with #[block("tag-name")] get decorated with
// a thing that goes and records the actual function's inputs and outputs in a struct
// within the struct in the design macro
//
// prefix the names of the function's args with the tag name
//
// do this kind of destructure on the usage side:
// ```
//     let Rat {
//        foo, bar, baz,
//    } = a;
// ```
// so you get a compile error if/when new flags are added
//
// The function decorating/thing that generates the struct and lets ppl access it
// should be behind a feature gate.

// The other way:

// This:
// ```
// let a: [Bit; 4] = b!(0b1000);
// let b: [Bit; 4] = b!(0b0100);
//
// let c: [Bit; 4] = b!(0b1111);
// let d: [Bit; 8] = b!(0b11001000);
//
// let e: Xor<Imm<Output = [Bit; 4]>, Imm<Output = [Bit; 4]>, Output = [Bit; 4]> = (a ^ b);
//
// let f: Or<Add<Zext<Xor<Imm<Output = [Bit; 4]>, Imm<Output = [Bit; 4]>, Output = [Bit; 4]>, Output = [Bit; 8]>, Zext<Imm<Output = [Bit; 4]>, Output = [Bit; 8]>>, Imm<Output = [Bit; 8]> = (zext::<8>(a ^ b) + zext::<8>(c)) | d;

mod util;
mod ops;
mod wires;

// use core::cell::RefCell;
// use core::cell::Cell;
// use core::ops::{Add as AddOp, Sub as SubOp, Mul as MulOp, Div as DivOp, BitAnd, BitOr, BitXor};


// macro_rules! b {
//     ($bits:literal b $rest:tt) => { $rest };
// }


// struct Add<L: Op, R: Op, Output = <L as Op>::Output>
// where
//     L: Op<Output = <R as Op>::Output>
// {
//     lhs: L,
//     rhs: R,
//     _rs: PhantomData<Output>
// }


// #[derive(Debug)]
// struct RegisterInput<R: Registers, T: Display, F: Fn(&R) -> T> {
//     func: F,
//     _output: PhantomData<T>,
//     _input: PhantomData<R>,
// }

// impl<R: Ctx, T: Display, F: Fn(&R) ->T> RegisterInput<R, T, F> {
//     fn new(func: F) -> OpWrapper<R, Self> {
//         RegisterInput { func, _input: PhantomData, _output: PhantomData }.into()
//     }
// }


// impl<S: Ctx, T: Display, F: Fn(&S) ->T> Op<S> for RegisterInput<S, T, F>
// {
//     type Output = T;
//     const OPERATION: OpType = OpType::RegisterInput;

//     fn execute(self, state: &S) -> Self::Output {
//         (self.func)(state)
//     }

//     fn execute_with_metadata(self, state: &S) -> (String, Self::Output) {
//         let val = self.execute(state);
//         (format!("<{}>", val), val)
//     }
// }

// trait Registers {
//     fn read_reg(&self, num: u8) -> u16;

//     fn write_reg(&self, num: u8, bytes: u16);
// }

// trait Memory {
//     fn read_mem(&self, addr: u16) -> u8;

//     fn write_mem(&self, addr: u16, byte: u8);
// }

// trait Ctx: Registers + Memory {}
// impl<S: Registers + Memory> Ctx for S {}

// struct State {
//     general: Cell<[u16; 8]>,
//     mem: RefCell<[u8; std::u16::MAX as usize]>
// }

// impl State {
//     fn new() -> Self {
//         State {
//             general: Cell::new([0u16; 8]),
//             mem: RefCell::new([0u8; std::u16::MAX as usize])
//         }
//     }
// }

// impl Registers for State {
//     fn read_reg(&self, num: u8) -> u16 {
//         self.general.get()[num as usize]
//     }

//     fn write_reg(&self, num: u8, bytes: u16) {
//         let mut regs = self.general.get();
//         regs[num as usize] = bytes;
//         self.general.set(regs);
//     }
// }

// impl Memory for State {
//     fn read_mem(&self, addr: u16) -> u8 {
//         self.mem.borrow()[addr as usize]
//     }

//     fn write_mem(&self, addr: u16, byte: u8) {
//         self.mem.borrow_mut()[addr as usize] = byte;
//     }
// }


// fn main() {
//     // let a: Imm<_> = 89.into();
//     // let b: Imm<_> = 90.into();

//     let state = State::new();

//     let r0 = RegisterInput::<State, _, _>::new(|r| r.read_reg(0));

//     <State as Registers>::write_reg(&state, 0, 3);

//     let a: OpWrapper<_, _> = 89.into();
//     let b = 90.into();

//     let c = 678.into();

//     // let w: OpWrapper<_> = a.into();
//     let c = a + b + c + r0;

//     let (m, v) = c.execute_with_metadata(&state);

//     println!("{} = {}", m, v);
// }


// TODO: split wires into it's own hdl-wires crate (probably)
