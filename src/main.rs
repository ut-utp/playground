// fn main() {
//     println!("Hello, world!");
// }

type Bit = bool;

struct Rat {
    foo: u8,
    bar: u16,
    baz: u32,
    // doo: u64
}

fn foo(a: Rat) -> (u8, u16, u32) {
    let Rat {
        foo, bar, baz,
    } = a;

    (foo, bar, baz)
}

fn a_block(a: [Bit; 4], b: [Bit; 4]) -> [Bit; 12] {
    [a[0], a[1], a[2], a[3], a[0], a[1], a[2], a[3], a[0], a[1], a[2], a[3]]
}

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


use core::cell::Cell;
use core::fmt::Display;
use core::marker::PhantomData;
use core::ops::{Add as AddOp, Sub as SubOp, Mul as MulOp, Div as DivOp, BitAnd, BitOr, BitXor};

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

enum OpType {
    Immediate,
    Input,
    Addition,
}

trait Op {
    type Output;
    const OPERATION: OpType;

    fn execute<S: FullContext>(self, state: &S) -> Self::Output;
    fn execute_with_metadata<S: FullContext>(self, state: &S) -> (String, Self::Output);

    // fn execute_with_graph(self) -> // TODO
}

// struct Add<L: Op, R: Op, Output = <L as Op>::Output>
// where
//     L: Op<Output = <R as Op>::Output>
// {
//     lhs: L,
//     rhs: R,
//     _rs: PhantomData<Output>
// }

#[derive(Copy, Clone, Debug)]
struct Add<L: Op, R: Op>
where
    L: Op<Output = <R as Op>::Output>,
    L::Output: AddOp<<R as Op>::Output, Output = <R as Op>::Output>
{
    lhs: L,
    rhs: R,
}

impl<L: Op, R: Op> Op for Add<L, R>
where
    L: Op<Output = <R as Op>::Output>,
    L::Output: AddOp<<R as Op>::Output, Output = <R as Op>::Output>
{
    type Output = <L as Op>::Output;
    const OPERATION: OpType = OpType::Addition;

    fn execute<S: FullContext>(self, state: &S) -> Self::Output {
        let lhs = self.lhs.execute(state);
        let rhs = self.rhs.execute(state);

        lhs + rhs
    }

    fn execute_with_metadata<S: FullContext>(self, state: &S) -> (String, Self::Output) {
        let (ml, lhs) = self.lhs.execute_with_metadata(state);
        let (mr, rhs) = self.rhs.execute_with_metadata(state);

        (format!("({} + {})", ml, mr), lhs + rhs)
    }
}

#[derive(Debug)]
struct RegisterInput<T: Display, I: Registers, F: Fn(&I) -> T> {
    func: F,
    _output: PhantomData<T>,
    _input: PhantomData<I>,
}

impl<T: Display, I: FullContext, F: Fn(&I) ->T> RegisterInput<T, I, F> {
    fn new(func: F) -> OpWrapper<Self> {
        RegisterInput { func, _input: PhantomData, _output: PhantomData }.into()
    }
}


impl<T: Display, I: FullContext, F: Fn(&I) ->T> Op for RegisterInput<T, I, F>
{
    type Output = T;
    const OPERATION: OpType = OpType::Input;

    fn execute<S: FullContext>(self, state: &S) -> Self::Output {
        (self.func)(state)
    }

    fn execute_with_metadata<S: FullContext>(self, state: &S) -> (String, Self::Output) {
        let val = self.execute(state);
        (format!("<{}>", val), val)
    }
}

#[derive(Copy, Clone, Debug)]
struct Imm<T: Display> {
    imm: T
}

impl<T: Display> From<T> for OpWrapper<Imm<T>> {
    fn from(imm: T) -> Self {
        Imm { imm }.into()
    }
}

impl<T: Display> Op for Imm<T> {
    type Output = T;
    const OPERATION: OpType = OpType::Immediate;

    fn execute<S: FullContext>(self, _state: &S) -> Self::Output {
        self.imm
    }

    fn execute_with_metadata<S: FullContext>(self, _state: &S) -> (String, Self::Output) {
        (format!("{}", self.imm), self.imm)
    }
}

#[derive(Copy, Clone, Debug)]
struct OpWrapper<T: Op>(T);

impl<T: Op> OpWrapper<T> {
    fn unwrap(self) -> T {
        self.0
    }
}

impl<T: Op> Op for OpWrapper<T> {
    type Output = <T as Op>::Output;
    const OPERATION: OpType = <T as Op>::OPERATION;

    fn execute<S: FullContext>(self, state: &S) -> Self::Output {
        self.0.execute(state)
    }

    fn execute_with_metadata<S: FullContext>(self, state: &S) -> (String, Self::Output) {
        self.0.execute_with_metadata(state)
    }
}

impl<O: Op> From<O> for OpWrapper<O> {
    fn from(op: O) -> Self {
        OpWrapper(op)
    }
}

#[derive(Copy, Clone, Debug)]
struct LabeledOp<T: Op>(T, &'static str); // TODO!

impl<L: Op, R: Op> AddOp<OpWrapper<R>> for OpWrapper<L>
where
    L: Op<Output = <R as Op>::Output>,
    L::Output: AddOp<<R as Op>::Output, Output = <R as Op>::Output>
{
    type Output = OpWrapper<Add<L, R>>;

    fn add(self, rhs: OpWrapper<R>) -> Self::Output {
        Add {
            lhs: self.unwrap(),
            rhs: rhs.unwrap(),
        }.into()
    }
}

trait Registers {
    fn read(&self, num: u8) -> u16;

    fn write(&self, num: u8, byte: u16);
}

trait Memory {
    fn read(self, addr: u16) -> u8;

    fn write(&self, addr: u16, byte: u8);
}

trait FullContext: Registers + Memory {}
impl<S: Registers + Memory> FullContext for S {}

struct State {
    general: Cell<[u16; 8]>,
    mem: [u8; std::u16::MAX as usize]
}

impl State {
    fn new() -> Self {
        State {
            general: Cell::new([0u16; 8]),
            mem: [0u8; std::u16::MAX as usize]
        }
    }
}

impl Registers for State {

}

impl Memory for State {

}


fn main() {
    // let a: Imm<_> = 89.into();
    // let b: Imm<_> = 90.into();

    let state = State::new();

    let r0 = RegisterInput::new(|r| r.read(0));

    <State as Registers>::write(&state, 0, 3);

    let a: OpWrapper<_> = 89.into();
    let b = 90.into();

    let c = 678.into();

    // let w: OpWrapper<_> = a.into();
    let c = a + b + c + r0;

    let (m, v) = c.execute_with_metadata(&state);

    println!("{} = {}", m, v);
}