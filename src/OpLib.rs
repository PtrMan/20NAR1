// library of standard ops

use crate::NarProc;
use crate::Term::{Term};

// ops for pong environment
pub struct OpNop {
    pub name: String, // name of this op
}

impl NarProc::Op for OpNop {
    fn retName(&self) -> String {
        self.name.clone()
    }
    fn call(&self, _args:&Vec<Term>) {
    }
}
