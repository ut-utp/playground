use super::{Op, OpKind, OpGraphNode, OpWrapper};

use core::fmt::Display;

#[derive(Copy, Clone, Debug)]
pub struct Imm<T: Display> {
    imm: T
}

impl<T: Display> From<T> for OpWrapper<Imm<T>> {
    fn from(imm: T) -> Self {
        Imm { imm }.into()
    }
}

impl<T: Display> Op for Imm<T> {
    type Output = T;
    const OPERATION: OpKind = OpKind::Immediate;

    fn execute(self) -> Self::Output {
        self.imm
    }

    fn execute_with_metadata(self) -> (String, Self::Output) {
        (format!("{}", self.imm), self.imm)
    }

    // fn execute_with_graph(self) ->
}
