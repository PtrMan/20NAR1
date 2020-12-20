// library of standard ops

use std::rc::Rc;

use crate::NarProc;
use crate::Term::{Term, convTermToStr, checkEqTerm, Copula};
use crate::TermUtils::decodeOp;

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
    fn isBabbleable(&self) -> bool {true}
}

/// NAL9 operator to execute a sequence of operations and inject a event as input after doing so
/// 
/// ex:
/// `<(a,<{({SELF}*(<{({SELF}*dummy0)} --> ^nop>,<{({SELF}*dummy0)} --> ^nop>)*b)} --> ^nal9_exeAndInject>) =/> g>..`
/// 
pub struct Op_nal9__exec_and_inject {
}
impl NarProc::Op for Op_nal9__exec_and_inject {
    fn retName(&self) -> String {
        "^nal9_exeAndInject".to_string()
    }
    fn call(&self, nar:&mut NarProc::ProcNar, args:&Vec<Term>) {
        println!("ENTER");

        if args.len() != 3 {
            return; // soft error
        }

        // second parameter must be sequence of ops to call!
        let opsSeq: Vec<Term> = match &args[1] {
            Term::Seq(seq) => {
                seq.iter().map(|iv| (**iv).clone()).collect()
            },
            _ => {
                return; // soft error
            }
        };
        
        // third parameter is event to inject
        let injEvent: Term = args[2].clone();

        println!("EXEC");


        // * execute ops sequentially
        for iOpTerm in &opsSeq {
            match decodeOp(iOpTerm) {
                Some((opArgs, opName)) => {
                    let opOpt = NarProc::ret_op_by_name(nar, &opName);
                    if opOpt.is_some() {
                        println!("{}!", &convTermToStr(&iOpTerm)); // print execution
                        opOpt.unwrap().call(nar, &opArgs);
                    }
                },
                None => {} // ignore
            }
        }

        // * inject event
        nar.trace.push(Rc::new(NarProc::SimpleSentence {name:injEvent.clone(),evi:nar.t,occT:nar.t}));
    }
    fn isBabbleable(&self) -> bool {false} // can't be used for babbling because it doesn't make any sense
}


pub struct Op__nlp_rel_0 {
}
impl NarProc::Op for Op__nlp_rel_0 {
    fn retName(&self) -> String {
        "^nlpRel0".to_string()
    }
    fn call(&self, nar:&mut NarProc::ProcNar, args:&Vec<Term>) {
        println!("ENTER NLP rel 0");

        if args.len() != 2 {
            return; // soft error
        }

        // second parameter is relation to add
        let rel:Term = args[1].clone();

        // rewrite "and" sequence to ExtInt
        let rewrittenRel:Term = match rel.clone() {
            Term::Stmt(Copula::INH, subj, pred) => {
                match *subj {
                    Term::Prod(arr) => {
                        let right:Term = (*arr[1]).clone();
                        let rewrittenRight:Term = match right {
                            Term::Seq(arr) if arr.len() == 3 && checkEqTerm(&arr[1], &Term::Name("and".to_string())) => {
                                Term::ExtInt(vec![Box::new((*arr[0]).clone()), Box::new((*arr[2]).clone())]) // rewrite to ExtInt
                            },
                            _ => {
                                right
                            }
                        };
                        let rewrittenProd:Term = Term::Prod(vec![Box::new((*arr[0]).clone()), Box::new(rewrittenRight)]);
                        Term::Stmt(Copula::INH, Box::new(rewrittenProd), Box::new((*pred).clone()))
                    },
                    _ => {
                        Term::Stmt(Copula::INH, subj, pred)
                    }
                }
            },
            _ => {
                rel.clone()
            }
        };

        println!("H rel {}", &convTermToStr(&rel)); // print relation
        println!("H rewritten rel {}", &convTermToStr(&rewrittenRel)); // print relation
        
        //TODO< add rewrittenRel relation to declarative memory! >
    }
    fn isBabbleable(&self) -> bool {false} // can't be used for babbling because it is not useful to babble it
}