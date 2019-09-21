use super::{Op, OpKind, OpGraphNode, OpWrapper};

use core::ops::Add as AddOp;

#[derive(Copy, Clone, Debug)]
pub struct Add<L: Op, R: Op>
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
    const OPERATION: OpKind = OpKind::Addition;

    fn execute(self) -> Self::Output {
        let lhs = self.lhs.execute();
        let rhs = self.rhs.execute();

        lhs + rhs
    }

    fn execute_with_metadata(self) -> (String, Self::Output) {
        let (ml, lhs) = self.lhs.execute_with_metadata();
        let (mr, rhs) = self.rhs.execute_with_metadata();

        (format!("({} + {})", ml, mr), lhs + rhs)
    }
}

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
