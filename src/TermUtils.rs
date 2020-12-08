//! utilities for terms and manipulation of terms

use crate::Term::*;

/// decodes a operator into the arguments and name
/// returns None if the term can't be decoded
/// expects term to be <{(arg0 * arg1 * ...)} --> ^opname>
pub fn decodeOp(term:&Term) -> Option<(Vec<Term>,String)> {
    match term {
        Term::Stmt(Copula::INH, subj, pred) => {
            match &**pred {
                Term::Name(predName) => {
                    match &**subj {
                        Term::SetExt(subj2) if subj2.len() == 1 => {
                            match &*subj2[0] {
                                Term::Prod(args) if args.len() >= 1 => {
                                    return Some((args.iter().map(|v| (**v).clone()).collect(), predName.clone()));
                                },
                                _ => {return None;}
                            }
                        },
                        _ => {return None;}
                    }
                },
                _ => {return None;}
            }
        },
        _ => {return None;}
    }
}

/// encode op, used to get called from external code
pub fn encodeOp(args:&Vec<Term>, name:&String) -> Term {
    let argProd = Term::Prod(args.iter().map(|v| Box::new(v.clone())).collect()); // build product of arg
    Term::Stmt(Copula::INH, Box::new(Term::SetExt(vec![Box::new(argProd)])), Box::new(Term::Name(name.clone())))
}