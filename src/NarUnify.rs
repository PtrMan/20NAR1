//! Logic unifier for Terms for (any) NARS

use crate::Term::Term;
use crate::Term::Copula;
use crate::Term::retSubterms;
use crate::Term::retUniqueSubterms;
use crate::Term::checkEqTerm;

/// structure to store assignment of var
// PUBLICAPI
pub struct Asgnment {
    pub var:Term,
    pub val:Term,
}


fn unify2(a2:&Term,b2:&Term,assignments:&mut Vec<Asgnment>) -> bool {
    match a2 {
        Term::QVar(_namea) => {
            match b2 {
                Term::QVar(_nameb) => false, // can't unify var with var
                Term::DepVar(_nameb) => false, // can't unify var with var
                Term::IndepVar(_nameb) => false, // can't unify var with var
                _ => {
                    if checkAssigned(&a2, &assignments) {
                        return checkSameVal(&a2, &b2, &assignments);
                    }
                    else {
                        assignments.push(Asgnment{var:a2.clone(),val:b2.clone(),}); // add assignment
                        true
                    }
                }
            }
        },
        Term::DepVar(_namea) => {
            match b2 {
                Term::QVar(_nameb) => false, // can't unify var with var
                Term::DepVar(_nameb) => false, // can't unify var with var
                Term::IndepVar(_nameb) => false, // can't unify var with var
                _ => {
                    if checkAssigned(&a2, &assignments) {
                        return checkSameVal(&a2, &b2, &assignments);
                    }
                    else {
                        assignments.push(Asgnment{var:a2.clone(),val:b2.clone(),}); // add assignment
                        true
                    }
                }
            }
        },
        Term::IndepVar(_namea) => {
            match b2 {
                Term::QVar(_nameb) => false, // can't unify var with var
                Term::DepVar(_nameb) => false, // can't unify var with var
                Term::IndepVar(_nameb) => false, // can't unify var with var
                _ => {
                    if checkAssigned(&a2, &assignments) {
                        return checkSameVal(&a2, &b2, &assignments);
                    }
                    else {
                        assignments.push(Asgnment{var:a2.clone(),val:b2.clone(),}); // add assignment
                        true
                    }
                }
            }
        },
        
        
        Term::Stmt(copulaa, subja, preda) => {
            match b2 {
                Term::Stmt(copulab, subjb, predb) if copulaa == copulab => {
                    unify2(&subja, &subjb, assignments) && unify2(&preda, &predb, assignments)
                },
                _ => false
            }
        },
        Term::Name(namea) => {
            match b2 {
                Term::Name(nameb) => namea == nameb,
                _ => false
            }
        },
        Term::Seq(seqa) => {
            match b2 {
                Term::Seq(seqb) => {
                    if seqa.len() == seqb.len() {
                        for idx in 0..seqa.len() {
                            if !unify2(&seqa[idx], &seqb[idx], assignments) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::SetInt(seta) => {
            match b2 {
                Term::SetInt(setb) => {
                    if seta.len() == setb.len() {
                        for idx in 0..seta.len() {
                            if !unify2(&seta[idx], &setb[idx], assignments) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::SetExt(seta) => {
            match b2 {
                Term::SetExt(setb) => {
                    if seta.len() == setb.len() {
                        for idx in 0..seta.len() {
                            if !unify2(&seta[idx], &setb[idx], assignments) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::Conj(elementsa) => {
            match b2 {
                Term::Conj(elementsb) => {
                    if elementsa.len() == elementsb.len() {
                        for idx in 0..elementsa.len() {
                            if !unify2(&elementsa[idx], &elementsb[idx], assignments) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::Prod(elementsa) => {
            match b2 {
                Term::Prod(elementsb) => {
                    if elementsa.len() == elementsb.len() {
                        for idx in 0..elementsa.len() {
                            if !unify2(&elementsa[idx], &elementsb[idx], assignments) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::Img(rela,idxa,elementsa) => {
            match b2 {
                Term::Img(relb,idxb,elementsb) if idxa==idxb => {
                    if !unify2(&rela, &relb, assignments) {return false};
                    
                    if elementsa.len() == elementsb.len() {
                        for idx in 0..elementsa.len() {
                            if !unify2(&elementsa[idx], &elementsb[idx], assignments) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::IntInt(seta) => {
            match b2 {
                Term::IntInt(setb) => {
                    if seta.len() == setb.len() {
                        for idx in 0..seta.len() {
                            if !unify2(&seta[idx], &setb[idx], assignments) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::ExtInt(seta) => {
            match b2 {
                Term::ExtInt(setb) => {
                    if seta.len() == setb.len() {
                        for idx in 0..seta.len() {
                            if !unify2(&seta[idx], &setb[idx], assignments) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::Par(elementsa) => {
            match b2 {
                Term::Par(elementsb) => {
                    if elementsa.len() == elementsb.len() {
                        for idx in 0..elementsa.len() {
                            if !unify2(&elementsa[idx], &elementsb[idx], assignments) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },

        Term::Neg(terma) => {
            match b2 {
                Term::Neg(termb) => {
                    unify2(&terma, &termb, assignments)
                },
                _ => false
            }
        },
    }
}

/// check if the variable is already assigned
fn checkAssigned(var:&Term, assignments:&Vec<Asgnment>) -> bool {
    assignments.iter().any(|asgn| checkEqTerm(&asgn.var, &var))
}

fn checkSameVal(var:&Term, val:&Term, assignments:&Vec<Asgnment>) -> bool {
    for i in assignments {
        if checkEqTerm(&i.var, &var) {
            return checkEqTerm(&i.val, &val);
        }
    }
    false
}

/// tries to unify terms
// PUBLICAPI
pub fn unify(a: &Term, b: &Term) -> Option<Vec<Asgnment>> {
    let mut assignments:Vec<Asgnment> = vec![];
    if unify2(&a,&b,&mut assignments) {
        return Some(assignments);        
    }
    None
}

// substitute variables with values
// PUBLICAPI
pub fn unifySubst(t: &Term, subst: &Vec<Asgnment>) -> Term {
    match t {
        Term::QVar(_name) => {
            // search for variable
            for iasgn in subst {
                if checkEqTerm(&t, &iasgn.var) {
                    return iasgn.val.clone();
                }
            }
            (*t).clone()
        },

        Term::DepVar(_name) => {
            // search for variable
            for iasgn in subst {
                if checkEqTerm(&t, &iasgn.var) {
                    return iasgn.val.clone();
                }
            }
            (*t).clone()
        },
        Term::IndepVar(_name) => {
            // search for variable
            for iasgn in subst {
                if checkEqTerm(&t, &iasgn.var) {
                    return iasgn.val.clone();
                }
            }
            (*t).clone()
        },
        
        Term::Stmt(copula, subj, pred) => {Term::Stmt(*copula, Box::new(unifySubst(subj, subst)), Box::new(unifySubst(pred, subst)))},
        Term::Name(_) => (*t).clone(),
        
        Term::Seq(subterms) => {
            let mut arr = vec![];
            for i in subterms {
                arr.push(Box::new(unifySubst(i, subst)));
            }
            Term::Seq(arr)
        },
        Term::SetInt(subterms) => {
            let mut arr = vec![];
            for i in subterms {
                arr.push(Box::new(unifySubst(i, subst)));
            }
            Term::SetInt(arr)
        },
        Term::SetExt(subterms) => {
            let mut arr = vec![];
            for i in subterms {
                arr.push(Box::new(unifySubst(i, subst)));
            }
            Term::SetExt(arr)
        },
        Term::Conj(subterms) => {
            let mut arr = vec![];
            for i in subterms {
                arr.push(Box::new(unifySubst(i, subst)));
            }
            Term::Conj(arr)
        },
        Term::Prod(subterms) => {
            let mut arr = vec![];
            for i in subterms {
                arr.push(Box::new(unifySubst(i, subst)));
            }
            Term::Prod(arr)
        },
        Term::Img(rel,idx,subterms) => {
            let rel2 = unifySubst(rel, subst);

            let mut arr = vec![];
            for i in subterms {
                arr.push(Box::new(unifySubst(i, subst)));
            }
            Term::Img(Box::new(rel2),*idx,arr)
        },
        Term::IntInt(subterms) => {
            let mut arr = vec![];
            for i in subterms {
                arr.push(Box::new(unifySubst(i, subst)));
            }
            Term::IntInt(arr)
        },
        Term::ExtInt(subterms) => {
            let mut arr = vec![];
            for i in subterms {
                arr.push(Box::new(unifySubst(i, subst)));
            }
            Term::ExtInt(arr)
        },
        Term::Par(subterms) => {
            let mut arr = vec![];
            for i in subterms {
                arr.push(Box::new(unifySubst(i, subst)));
            }
            Term::Par(arr)
        },
        Term::Neg(term2) => {Term::Neg(Box::new(unifySubst(term2, subst)))},
    }
}
