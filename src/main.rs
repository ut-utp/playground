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


use core::cell::RefCell;
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

trait Op<S: Ctx> {
    type Output;
    const OPERATION: OpType;

    fn execute(self, state: &S) -> Self::Output;
    fn execute_with_metadata(self, state: &S) -> (String, Self::Output);

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
struct Add<S: Ctx, L: Op<S>, R: Op<S>>
where
    L: Op<S, Output = <R as Op<S>>::Output>,
    L::Output: AddOp<<R as Op<S>>::Output, Output = <R as Op<S>>::Output>
{
    lhs: L,
    rhs: R,
    _state: PhantomData<S>,
}

impl<S: Ctx, L: Op<S>, R: Op<S>> Op<S> for Add<S, L, R>
where
    L: Op<S, Output = <R as Op<S>>::Output>,
    L::Output: AddOp<<R as Op<S>>::Output, Output = <R as Op<S>>::Output>
{
    type Output = <L as Op<S>>::Output;
    const OPERATION: OpType = OpType::Addition;

    fn execute(self, state: &S) -> Self::Output {
        let lhs = self.lhs.execute(state);
        let rhs = self.rhs.execute(state);

        lhs + rhs
    }

    fn execute_with_metadata(self, state: &S) -> (String, Self::Output) {
        let (ml, lhs) = self.lhs.execute_with_metadata(state);
        let (mr, rhs) = self.rhs.execute_with_metadata(state);

        (format!("({} + {})", ml, mr), lhs + rhs)
    }
}

#[derive(Debug)]
struct Input<R: Registers, T: Display, F: Fn(&R) -> T> {
    func: F,
    _output: PhantomData<T>,
    _input: PhantomData<R>,
}

impl<R: Ctx, T: Display, F: Fn(&R) ->T> Input<R, T, F> {
    fn new(func: F) -> OpWrapper<R, Self> {
        Input { func, _input: PhantomData, _output: PhantomData }.into()
    }
}


impl<S: Ctx, T: Display, F: Fn(&S) ->T> Op<S> for Input<S, T, F>
{
    type Output = T;
    const OPERATION: OpType = OpType::Input;

    fn execute(self, state: &S) -> Self::Output {
        (self.func)(state)
    }

    fn execute_with_metadata(self, state: &S) -> (String, Self::Output) {
        let val = self.execute(state);
        (format!("<{}>", val), val)
    }
}

#[derive(Copy, Clone, Debug)]
struct Imm<T: Display> {
    imm: T
}

impl<S: Ctx, T: Display> From<T> for OpWrapper<S, Imm<T>> {
    fn from(imm: T) -> Self {
        Imm { imm }.into()
    }
}

impl<S: Ctx, T: Display> Op<S> for Imm<T> {
    type Output = T;
    const OPERATION: OpType = OpType::Immediate;

    fn execute(self, _state: &S) -> Self::Output {
        self.imm
    }

    fn execute_with_metadata(self, _state: &S) -> (String, Self::Output) {
        (format!("{}", self.imm), self.imm)
    }
}

#[derive(Copy, Clone, Debug)]
struct OpWrapper<S: Ctx, T: Op<S>>(T, PhantomData<S>);

impl<S: Ctx, T: Op<S>> OpWrapper<S, T> {
    fn unwrap(self) -> T {
        self.0
    }
}

impl<S: Ctx, T: Op<S>> Op<S> for OpWrapper<S, T> {
    type Output = <T as Op<S>>::Output;
    const OPERATION: OpType = <T as Op<S>>::OPERATION;

    fn execute(self, state: &S) -> Self::Output {
        self.0.execute(state)
    }

    fn execute_with_metadata(self, state: &S) -> (String, Self::Output) {
        self.0.execute_with_metadata(state)
    }
}

impl<S: Ctx, O: Op<S>> From<O> for OpWrapper<S, O> {
    fn from(op: O) -> Self {
        OpWrapper(op, PhantomData)
    }
}

#[derive(Copy, Clone, Debug)]
struct LabeledOp<S: Ctx, T: Op<S>>(T, &'static str, PhantomData<S>); // TODO!

impl<S: Ctx, L: Op<S>, R: Op<S>> AddOp<OpWrapper<S, R>> for OpWrapper<S, L>
where
    L: Op<S, Output = <R as Op<S>>::Output>,
    L::Output: AddOp<<R as Op<S>>::Output, Output = <R as Op<S>>::Output>
{
    type Output = OpWrapper<S, Add<S, L, R>>;

    fn add(self, rhs: OpWrapper<S, R>) -> Self::Output {
        Add {
            lhs: self.unwrap(),
            rhs: rhs.unwrap(),
            _state: PhantomData
        }.into()
    }
}

trait Registers {
    fn read_reg(&self, num: u8) -> u16;

    fn write_reg(&self, num: u8, bytes: u16);
}

trait Memory {
    fn read_mem(&self, addr: u16) -> u8;

    fn write_mem(&self, addr: u16, byte: u8);
}

trait Ctx: Registers + Memory {}
impl<S: Registers + Memory> Ctx for S {}

struct State {
    general: Cell<[u16; 8]>,
    mem: RefCell<[u8; std::u16::MAX as usize]>
}

impl State {
    fn new() -> Self {
        State {
            general: Cell::new([0u16; 8]),
            mem: RefCell::new([0u8; std::u16::MAX as usize])
        }
    }
}

impl Registers for State {
    fn read_reg(&self, num: u8) -> u16 {
        self.general.get()[num as usize]
    }

    fn write_reg(&self, num: u8, bytes: u16) {
        let mut regs = self.general.get();
        regs[num as usize] = bytes;
        self.general.set(regs);
    }
}

impl Memory for State {
    fn read_mem(&self, addr: u16) -> u8 {
        self.mem.borrow()[addr as usize]
    }

    fn write_mem(&self, addr: u16, byte: u8) {
        self.mem.borrow_mut()[addr as usize] = byte;
    }
}


fn main() {
    // let a: Imm<_> = 89.into();
    // let b: Imm<_> = 90.into();

    let state = State::new();

    let r0 = Input::<State, _, _>::new(|r| r.read_reg(0));

    <State as Registers>::write_reg(&state, 0, 3);

    let a: OpWrapper<_, _> = 89.into();
    let b = 90.into();

    let c = 678.into();

    // let w: OpWrapper<_> = a.into();
    let c = a + b + c + r0;

    let (m, v) = c.execute_with_metadata(&state);

    println!("{} = {}", m, v);
}