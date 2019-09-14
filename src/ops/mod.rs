
pub trait Op {
    type Output;
    const OPERATION: OpKind;

    fn execute(self) -> Self::Output;
    fn execute_with_metadata(self) -> (String, Self::Output);

    // fn execute_with_graph(self) -> OpGraphNode;
}

mod op_types;

pub(crate) use op_types::OpWrapper;
pub use op_types::LabeledOp;

mod add;
mod imm;


pub enum OpKind {
    Immediate,
    // RegisterInput,
    Addition,

}

// Until https://github.com/rust-lang/rfcs/pull/2593 happens, we're going to
// have to keep the trait impls and this enum in sync manually.
pub enum OpData {
    Immediate()
}

pub struct OpGraphNode {
    kind: OpKind,
    data: OpData,
}
