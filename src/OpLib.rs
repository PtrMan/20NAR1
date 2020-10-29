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
    fn call(&self, _nar:&mut NarProc::ProcNar, _args:&Vec<Term>) {
    }
}

/// NAL9 operator to execute a sequence of operations and inject a event as input after doing so
/// 
/// ex:
/// `<(a &/ ^nal9_exeAndInject((<({SELF}*dummy0) --> ^left>,<({SELF}*dummy0) --> ^top>),b)) =/> b>.`
/// 
/*
pub struct Op_nal9__exec_and_inject {
}
impl NarProc::Op for Op_nal9__exec_and_inject {
    fn retName(&self) -> String {
        "nal9_exeAndInject".to_string()
    }
    fn call(&self, nar:&mut NarProc::ProcNar, _args:&Vec<Term>) {
    }
}*/