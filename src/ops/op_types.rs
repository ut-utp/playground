use super::{Op, OpKind};

#[derive(Copy, Clone, Debug)]
pub struct OpWrapper<T: Op>(T);

impl<T: Op> OpWrapper<T> {
    pub(crate) fn unwrap(self) -> T {
        self.0
    }
}

impl<T: Op> Op for OpWrapper<T> {
    type Output = <T as Op>::Output;
    const OPERATION: OpKind = <T as Op>::OPERATION;

    fn execute(self) -> Self::Output {
        self.0.execute()
    }

    fn execute_with_metadata(self) -> (String, Self::Output) {
        self.0.execute_with_metadata()
    }
}

impl<O: Op> From<O> for OpWrapper<O> {
    fn from(op: O) -> Self {
        OpWrapper(op)
    }
}


#[derive(Copy, Clone, Debug)]
pub struct LabeledOp<T: Op>(T, &'static str); // TODO!
