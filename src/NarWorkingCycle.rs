
// TODO< select highest ranked task, remove it from array, select other task by priority distribution, do inference, put results into memory >
//     TODO< put processed task into randomly sampled bag-table! >



// TODO< add question variable >

use rand::Rng;
use rand::rngs::ThreadRng;

use std::rc::Rc;
use std::sync::Arc;

use crate::Term::Term;
use crate::Term::Copula;
use crate::Term::retSubterms;
use crate::Term::retUniqueSubterms;
use crate::Term::calcComplexity;
use crate::Term::convTermToStr;
use crate::Term::checkEqTerm;

use crate::NarSentence::EnumPunctation;
use crate::NarSentence::SentenceDummy;
use crate::NarSentence::convSentenceTermPunctToStr;
use crate::NarSentence::retTv;
use crate::NarSentence::Evidence;

use crate::NarMem;
use crate::Tv::*;
use crate::NarStamp::*;
use crate::NarStamp;

/* commented because not needed
// a --> b |- b --> a
pub fn inf2(a: &Term, aTv: &Tv) -> Option<(Term, Tv)> {
    match a {
        Term::Stmt(Copula::INH, asubj, apred) => {
            println!("TODO - compute tv");
            return Some((Term::Stmt(Copula::INH, Box::clone(apred), Box::clone(asubj)), aTv.clone()));
        },
        _ => {},
    }
    None
}
*/

// structural
// a --> (X | Y).
// |-
// a --> X.
// a --> Y.
// ...
pub fn infStructPred1(a: &Term, aTv: &Option<Tv>, idx:usize) -> Option<(Term, Tv)> {
    match a {
        Term::Stmt(Copula::INH, subj, pred) => {
            match &**pred {
                Term::IntInt(arr) => {
                    if idx < arr.len() {
                        let concl = Term::Stmt(Copula::INH, Box::clone(subj), Box::clone(&arr[idx]));
                        return Some((concl,aTv.as_ref().unwrap().clone()));
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}

// structural
// (X | Y) --> a.
// |-
// X --> a.
// Y --> a.
// ...
pub fn infStructSubj1(a: &Term, aTv: &Option<Tv>, idx:usize) -> Option<(Term, Tv)> {
    match a {
        Term::Stmt(Copula::INH, subj, pred) => {
            match &**subj {
                Term::IntInt(arr) => {
                    if idx < arr.len() {
                        let concl = Term::Stmt(Copula::INH, Box::clone(&arr[idx]), Box::clone(pred));
                        return Some((concl,aTv.as_ref().unwrap().clone()));
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}


// structural
// (X * Y) --> rel.
// |-
// <X --> (rel /1 Y)>.
// <Y --> (rel /2 X)>.
pub fn infStructProd0(a: &Term, aTv: &Option<Tv>) -> Option<(Term, Tv)> {
    match a {
        Term::Stmt(Copula::INH, prod, pred) => {
            match &**prod {
                Term::Prod(arr) => {
                    let prod0 = &arr[0];
                    let prod1 = &arr[1];

                    let concl = Term::Stmt(Copula::INH, Box::clone(&prod0), Box::new(Term::Img(Box::clone(pred), 0, vec![Box::clone(&prod1)])));
                    return Some((concl,aTv.as_ref().unwrap().clone()));
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}

// structural
// (X * Y) --> rel.
// |-
// <X --> (rel /1 Y)>.
// <Y --> (rel /2 X)>.
pub fn infStructProd1(a: &Term, aTv: &Option<Tv>) -> Option<(Term, Tv)> {
    match a {
        Term::Stmt(Copula::INH, prod, pred) => {
            match &**prod {
                Term::Prod(arr) => {
                    if arr.len() == 2 {
                        let prod0 = &arr[0];
                        let prod1 = &arr[1];
                        let concl = Term::Stmt(Copula::INH, Box::clone(&prod1), Box::new(Term::Img(Box::clone(pred), 1, vec![Box::clone(&prod0)])));
                        return Some((concl,aTv.as_ref().unwrap().clone()));
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}

// structural
// <X --> (rel /1 Y)>.
// |-
// (X * Y) --> rel.
pub fn infStructImg0(a: &Term, aTv: &Option<Tv>) -> Option<(Term, Tv)> {
    match a {
        Term::Stmt(Copula::INH, subj, pred) => {
            match &**pred {
                Term::Img(predImg,0,arr) => {
                    if arr.len() == 1 {
                        let arr0 = &arr[0];
                        let concl = Term::Stmt(Copula::INH, Box::new(Term::Prod(vec![Box::clone(&subj), Box::clone(&arr0)])), Box::clone(&predImg));
                        return Some((concl,aTv.as_ref().unwrap().clone()));
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}


// structural
// <Y --> (rel /2 X)>.
// |-
// (X * Y) --> rel.
pub fn infStructImg1(a: &Term, aTv: &Option<Tv>) -> Option<(Term, Tv)> {
    match a {
        Term::Stmt(Copula::INH, subj, pred) => {
            match &**pred {
                Term::Img(predImg,1,arr) => {
                    if arr.len() == 1 {
                        let arr0 = &arr[0];
                        let concl = Term::Stmt(Copula::INH, Box::new(Term::Prod(vec![Box::clone(&arr0), Box::clone(&subj)])), Box::clone(&predImg));
                        return Some((concl,aTv.as_ref().unwrap().clone()));
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}



// a --> x.  x --> b.  |- a --> b.
pub fn inf0(a: &Term, punctA:EnumPunctation, aTv:&Option<Tv>, b: &Term, punctB:EnumPunctation, bTv:&Option<Tv>) -> Option<(Term,Tv,EnumPunctation)> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::INH, asubj, apred) => {
            match b {
                Term::Stmt(Copula::INH, bsubj, bpred) => {
                    if !checkEqTerm(&asubj, &bpred) && checkEqTerm(&apred, &bsubj) {
                        return Some(( Term::Stmt(Copula::INH, Box::clone(asubj), Box::clone(bpred)), ded(&aTv.as_ref().unwrap(),&bTv.as_ref().unwrap()), EnumPunctation::JUGEMENT)); // a.subj --> b.pred
                    }
                },
                _ => {},
            }
        },
        _ => {},
    }
    None
}


// a --> x.  a --> y.  |- x <-> y.
pub fn infCompPred(a: &Term, punctA:EnumPunctation, aTv:&Option<Tv>, b: &Term, punctB:EnumPunctation, bTv:&Option<Tv>) -> Option<(Term,Tv,EnumPunctation)> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::INH, asubj, apred) => {
            match b {
                Term::Stmt(Copula::INH, bsubj, bpred) => {
                    if !checkEqTerm(&apred, &bpred) && checkEqTerm(&asubj, &bsubj) {
                        return Some(( Term::Stmt(Copula::SIM, Box::clone(apred), Box::clone(bpred)), comp(&aTv.as_ref().unwrap(),&bTv.as_ref().unwrap()), EnumPunctation::JUGEMENT)); // a.subj --> b.pred
                    }
                },
                _ => {},
            }
        },
        _ => {},
    }
    None
}


// x --> a.  y --> a.  |- x <-> y.
pub fn infCompSubj(a: &Term, punctA:EnumPunctation, aTv:&Option<Tv>, b: &Term, punctB:EnumPunctation, bTv:&Option<Tv>) -> Option<(Term,Tv,EnumPunctation)> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::INH, asubj, apred) => {
            match b {
                Term::Stmt(Copula::INH, bsubj, bpred) => {
                    if !checkEqTerm(&asubj, &bsubj) && checkEqTerm(&apred, &bpred) {
                        return Some(( Term::Stmt(Copula::SIM, Box::clone(asubj), Box::clone(bsubj)), comp(&aTv.as_ref().unwrap(),&bTv.as_ref().unwrap()), EnumPunctation::JUGEMENT)); // a.subj --> b.pred
                    }
                },
                _ => {},
            }
        },
        _ => {},
    }
    None
}


// x --> [a].  x --> [b].  |- x --> [a b].
pub fn inf3(a: &Term, punctA:EnumPunctation, aTv:&Option<Tv>, b: &Term, punctB:EnumPunctation, bTv:&Option<Tv>) -> Option<(Term,Tv,EnumPunctation)> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::INH, asubj, apred2) => {
            match &**apred2 {
                Term::SetInt(apred) => {
                    
                    match b {
                        Term::Stmt(Copula::INH, bsubj, bpred2) => {
                            match &**bpred2 {
                                Term::SetInt(bpred) => {
                                    
                                    if checkEqTerm(&asubj, &bsubj) {
                                        // build result set
                                        // TODO< compute union of set >
                                        let mut union_:Vec<Box<Term>> = vec![];
                                        union_.extend(apred.iter().cloned());
                                        union_.extend(bpred.iter().cloned());
                                        
                                        let resTerm = Term::SetInt(union_);
                                        
                                        println!("TODO - compute tv");
                                        return Some((Term::Stmt(Copula::INH, Box::clone(asubj), Box::new(resTerm)), aTv.as_ref().unwrap().clone(), EnumPunctation::JUGEMENT));
                                    }
                                    
                                },
                                _ => {}
                            }
                        },
                        _ => {},
                    }
                    
                    
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}


// {a} --> x.  {b} --> x.  |- {a b} --> x.
pub fn inf4(_a: &Term, _punctA:EnumPunctation, _aTv:&Option<Tv>, _b: &Term, _punctB:EnumPunctation, _bTv:&Option<Tv>) -> Option<(Term,Tv,EnumPunctation)> {
    return None; // is disabled because it violates AIKR to some degree!
    /*
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }

    match a {
        Term::Stmt(Copula::INH, asubj2, apred) => {
            match &**asubj2 {
                Term::SetExt(asubj) => {
                    
                    match b {
                        Term::Stmt(Copula::INH, bsubj2, bpred) => {
                            match &**bsubj2 {
                                Term::SetExt(bsubj) => {
                                    
                                    if checkEqTerm(&apred, &bpred) {
                                        // build result set
                                        // TODO< compute union of set >
                                        let mut union_:Vec<Box<Term>> = vec![];
                                        union_.extend(asubj.iter().cloned());
                                        union_.extend(bsubj.iter().cloned());
                                        
                                        let resTerm = Term::SetExt(union_);
                                        
                                        println!("TODO - compute tv");
                                        return Some((Term::Stmt(Copula::INH, Box::new(resTerm), Box::clone(apred)), aTv.clone()));
                                    }
                                    
                                },
                                _ => {}
                            }
                        },
                        _ => {},
                    }
                    
                    
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
    */
}

// a ==> x.  x ==> b.  |- a ==> b.
pub fn inf1(a: &Term, punctA:EnumPunctation, aTv:&Option<Tv>, b: &Term, punctB:EnumPunctation, bTv:&Option<Tv>) -> Option<(Term,Tv,EnumPunctation)> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }

    match a {
        Term::Stmt(Copula::IMPL, asubj, apred) => {
            match b {
                Term::Stmt(Copula::IMPL, bsubj, bpred) => {
                    if checkEqTerm(&apred, &bsubj) && !checkEqTerm(&asubj, &bpred) {
                        return Some((Term::Stmt(Copula::IMPL, Box::clone(asubj), Box::clone(bpred)), ded(&aTv.as_ref().unwrap(),&bTv.as_ref().unwrap()), EnumPunctation::JUGEMENT));
                    }
                },
                _ => {},
            }
        },
        _ => {},
    }
    None
}


pub struct Asgnment { // structure to store assignment of var
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
    }
}

// check if the variable is already assigned
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

// tries to unify terms
pub fn unify(a: &Term, b: &Term) -> Option<Vec<Asgnment>> {
    let mut assignments:Vec<Asgnment> = vec![];
    if unify2(&a,&b,&mut assignments) {
        return Some(assignments);        
    }
    None
}

// substitute variables with values
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
    }
}

// see https://cis.temple.edu/~pwang/Writing/NAL-Specification.pdf?page=50
// (a && b) ==> x.
// unify a.
// |- ded
// b ==> x.
pub fn inf5(a: &Term, punctA:EnumPunctation, aTv:&Option<Tv>, b: &Term, punctB:EnumPunctation, bTv:&Option<Tv>, conjIdx:usize) -> Option<(Term,Tv,EnumPunctation)> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::IMPL, aconj, apred) => {
            match &**aconj {
                Term::Conj(arr) => {
                    if conjIdx < arr.len() { // index in conj must be in bounds
                        let unifyRes = unify(&arr[conjIdx], &b);
                        if unifyRes.is_some() { // vars must unify
                            let unifyVal = unifyRes.unwrap();

                            let mut conclConj:Vec<Box<Term>> = vec![]; // array of conjunction of result
                            for idx2 in 0..arr.len() {
                                if idx2 == conjIdx {
                                    continue; // skip the unified subterm!
                                }
                                let subst = unifySubst(&arr[idx2], &unifyVal); // substitute vars
                                conclConj.push(Box::new(subst));
                            }

                            let substPred = unifySubst(&apred, &unifyVal); // substitute vars
                            
                            let conclTerm = if conclConj.len() == 1 {
                                Term::Stmt(Copula::IMPL, Box::clone(&conclConj[0]), Box::new(substPred)) // build implication with single term
                            }
                            else {
                                Term::Stmt(Copula::IMPL, Box::new(Term::Conj(conclConj)), Box::new(substPred)) // build implication with conjunction
                            };
                            
                            return Some((conclTerm, ded(&aTv.as_ref().unwrap(),&bTv.as_ref().unwrap()), EnumPunctation::JUGEMENT));
                        }
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}


// see https://cis.temple.edu/~pwang/Writing/NAL-Specification.pdf?page=42
// a ==> x.
// unify a.
// |- ded
// x.
pub fn infImplDed(a: &Term, punctA:EnumPunctation, aTv:&Option<Tv>, b: &Term, punctB:EnumPunctation, bTv:&Option<Tv>) -> Option<(Term,Tv,EnumPunctation)> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::IMPL, asubj, apred) => {
            let unifyRes = unify(asubj, &b);
            if unifyRes.is_some() { // vars must unify
                let subst = unifySubst(&apred, &unifyRes.unwrap()); // substitute vars
                return Some((subst,ded(&aTv.as_ref().unwrap(), &bTv.as_ref().unwrap()), EnumPunctation::JUGEMENT));
            };
            None
        },
        _ => None,
    }
}

// necessary for symbolic manipulation for example in https://github.com/orgs/NARS-team/teams/all/discussions/71
// a ==> x?
// unify x.
// |-
// a ==> x.
pub fn inf7(a: &Term, punctA:EnumPunctation, _aTv:&Option<Tv>, b: &Term, punctB:EnumPunctation, bTv:&Option<Tv>) -> Option<(Term, Tv, EnumPunctation)> {
    if punctA != EnumPunctation::QUESTION || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::IMPL, _, apred) => {
            let unifyRes = unify(apred, &b);
            if unifyRes.is_some() { // vars must unify
                let subst = unifySubst(&a, &unifyRes.unwrap()); // substitute vars
                println!("TODO - compute TV correctly!");
                return Some((subst,bTv.as_ref().unwrap().clone(), EnumPunctation::JUGEMENT));
            };
            None
        },
        _ => None,
    }
}



// do binary inference
pub fn infBinaryInner(a: &Term, aPunct:EnumPunctation, aTv:&Option<Tv>, b: &Term, bPunct:EnumPunctation, bTv:&Option<Tv>, wereRulesApplied:&mut bool) -> Vec<(Term,Tv,EnumPunctation)> {
    let mut res = vec![];
    
    match inf0(&a, aPunct, &aTv, &b, bPunct, &bTv) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match inf1(&a, aPunct, &aTv, &b, bPunct, &bTv) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match inf3(&a, aPunct, &aTv, &b, bPunct, &bTv) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match inf4(&a, aPunct, &aTv, &b, bPunct, &bTv) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match inf5(&a, aPunct, &aTv, &b, bPunct, &bTv, 0) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match inf5(&a, aPunct, &aTv, &b, bPunct, &bTv, 1) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match inf5(&a, aPunct, &aTv, &b, bPunct, &bTv, 2) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match inf5(&a, aPunct, &aTv, &b, bPunct, &bTv, 3) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match infImplDed(&a, aPunct, &aTv, &b, bPunct, &bTv) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match inf7(&a, aPunct, &aTv, &b, bPunct, &bTv) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match infCompPred(&a, aPunct, &aTv, &b, bPunct, &bTv) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match infCompSubj(&a, aPunct, &aTv, &b, bPunct, &bTv) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    
    res
}

// do binary inference
pub fn infBinary(a: &Term, aPunct:EnumPunctation, aTv:&Option<Tv>, b: &Term, bPunct:EnumPunctation, bTv:&Option<Tv>, wereRulesApplied:&mut bool) -> Vec<(Term,Tv,EnumPunctation)> {
    let mut res = vec![];
    *wereRulesApplied = false; // because no rules were applied yet
    res.extend(infBinaryInner(&a, aPunct, &aTv, &b, bPunct, &bTv, wereRulesApplied).iter().cloned());
    res.extend(infBinaryInner(&b, bPunct, &aTv, &a, aPunct, &bTv, wereRulesApplied).iter().cloned());
    res
}

pub fn infSinglePremise(a: &Term, _aPunct:EnumPunctation, aTv:&Option<Tv>) -> Vec<(Term,Tv)> {
    let mut res = vec![];
    
    match infStructSubj1(&a, &aTv, 0) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructSubj1(&a, &aTv, 1) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructSubj1(&a, &aTv, 2) {
        Some(x) => { res.push(x); } _ => {}
    }

    match infStructPred1(&a, &aTv, 0) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructPred1(&a, &aTv, 1) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructPred1(&a, &aTv, 2) {
        Some(x) => { res.push(x); } _ => {}
    }

    match infStructProd0(&a, &aTv) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructProd1(&a, &aTv) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructImg0(&a, &aTv) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructImg1(&a, &aTv) {
        Some(x) => { res.push(x); } _ => {}
    }

    res
}













#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    //    <<a --> b> ==> <c --> d>>
    //    <a --> b>
    //    concl:
    //    <<a --> b> ==> <c --> d>>
    pub fn impl_a() {
        let a0 = Term::Name("a".to_string());
        let b0 = Term::Name("b".to_string());
        let inh0 = Term::Stmt(Copula::INH, Box::new(a0), Box::new(b0));
        
        let c0 = Term::IndepVar("c".to_string());
        let d0 = Term::Name("d".to_string());
        let inh1 = Term::Stmt(Copula::INH, Box::new(c0), Box::new(d0));
        
        let impl0 = Term::Stmt(Copula::IMPL, Box::new(inh0), Box::new(inh1));
        
        
        let c1 = Term::Name("c".to_string());
        let d1 = Term::Name("d".to_string());
        let inh2 = Term::Stmt(Copula::INH, Box::new(c1), Box::new(d1));
        
        println!("{}", &convTermToStr(&impl0));
        println!("{}", &convTermToStr(&inh2));
        println!("concl:");
        
        let mut success=false;
        
        let mut wereRulesApplied = false;
        let infConcl = infBinary(&impl0, EnumPunctation::QUESTION, &None, &inh2, EnumPunctation::JUGEMENT, &Some(Tv{f:1.0,c:0.9}), &mut wereRulesApplied);
        for iInfConcl in infConcl {
            let (conclTerm, _conclTv, _punct) = iInfConcl;
            let conclTermStr = convTermToStr(&conclTerm);
            println!("{}", &conclTermStr);
            if conclTermStr == "<<a --> b> ==> <c --> d>>" {
                success=true;
            }
        }
        
        assert_eq!(success, true);
    }

    #[test]
    // test
    //    <( <a --> b> && <c --> d> ) ==> x>
    //    <a --> b>
    //    concl:
    //    <<c --> d> ==> x>
    pub fn implConj2_a() {
        let a0 = Term::Name("a".to_string());
        let b0 = Term::Name("b".to_string());
        let inh0 = Term::Stmt(Copula::INH, Box::new(a0), Box::new(b0));
        
        let c0 = Term::Name("c".to_string());
        let d0 = Term::Name("d".to_string());
        let inh1 = Term::Stmt(Copula::INH, Box::new(c0), Box::new(d0));
        
        let conj0 = Term::Conj(vec![Box::new(inh0), Box::new(inh1)]);
        
        let x0 = Term::Name("x".to_string());
        let impl0 = Term::Stmt(Copula::IMPL, Box::new(conj0), Box::new(x0));
        
        
        let a1 = Term::Name("a".to_string());
        let b1 = Term::Name("b".to_string());
        let inh1 = Term::Stmt(Copula::INH, Box::new(a1), Box::new(b1));
        
        println!("{}", &convTermToStr(&impl0));
        println!("{}", &convTermToStr(&inh1));
        println!("concl:");

        let mut success = false;
        
        let mut wereRulesApplied = false;
        let infConcl = infBinary(&impl0, EnumPunctation::JUGEMENT, &Some(Tv{f:1.0,c:0.9}), &inh1, EnumPunctation::JUGEMENT, &Some(Tv{f:1.0,c:0.9}), &mut wereRulesApplied);
        for iInfConcl in infConcl {
            let (conclTerm, _conclTv, _punct) = iInfConcl;
            let conclTermStr = convTermToStr(&conclTerm);
            println!("{}", &conclTermStr);
            if conclTermStr == "<<c --> d> ==> x>" {
                success=true;
            }
        }

        assert_eq!(success, true);
    }
}




// do inference of two sentences
// /param wereRulesApplied is true if any rules were applied
pub fn inference(pa:&SentenceDummy, pb:&SentenceDummy, wereRulesApplied:&mut bool)->Vec<SentenceDummy> {
    *wereRulesApplied = false;

    let mut concl = vec![];

    let infConcl = infBinary(&pa.term, pa.punct, &retTv(&pa), &pb.term, pb.punct, &retTv(&pb), wereRulesApplied);
    for iInfConcl in infConcl {
        let (term, tv, punct) = iInfConcl;
        concl.push(SentenceDummy{
            term:Rc::new(term.clone()),
            evi:if true {Some(Evidence::TV(tv.clone()))} else {None},
            stamp:merge(&pa.stamp, &pb.stamp),
            t:None, // time of occurence 
            punct:punct,
            expDt:None
        });
    }

    if concl.len() > 0 && checkOverlap(&pa.stamp, &pb.stamp) { // check for overlap
      concl = vec![]; // flush conclusions because we don't have any conclusions when the premises overlapped
    }
    
    concl
}

pub fn infSinglePremise2(pa:&SentenceDummy) -> Vec<SentenceDummy> {
    let mut concl = vec![];

    let infConcl = infSinglePremise(&pa.term, pa.punct, &retTv(pa));
    for iInfConcl in infConcl {
        let (term, tv) = iInfConcl;
        
        println!("TODO - infSinglePremise must compute the punctation!");
        concl.push(SentenceDummy{
            term:Rc::new(term.clone()),
            evi:if true {Some(Evidence::TV(tv.clone()))} else {None},
            stamp:pa.stamp.clone(),
            t:None, // time of occurence 
            punct:EnumPunctation::JUGEMENT, // BUG - we need to compute punctation in inference
            expDt:None
        });
    }

    concl
}


pub struct Task {
    pub sentence:SentenceDummy,
    pub credit:f64,
    pub id:i64, // unique id to quickly find unique tasks
    pub derivTime:i64, // time when this task was put into the working table
}

// compute "real" credit of task by insertion based time decay
pub fn taskCalcCredit(task:&Task, cycleCounter:i64) -> f64 {
    let decayFactor:f64 = 0.001; // how fast does it decay?
    
    let dt:i64 = cycleCounter - task.derivTime;
    let decayFactor:f64 = (-decayFactor * (dt as f64)).exp();

    task.credit*decayFactor // multiply because we want to decay the actual "base credit"
}

pub struct Task2 {
    pub sentence:SentenceDummy,
    pub handler:Option< Rc<RefCell< dyn QHandler>> >, // handler which is called when a better answer is found
    pub bestAnswerExp:f64, // expectation of best answer
    pub prio:f64, // priority
}

use std::collections::HashMap;
use std::cell::{RefCell};

pub struct Mem2 {
    pub judgementTasks:Vec<Rc<RefCell<Task>>>,
    pub judgementTasksByTerm:HashMap<Term, Vec<Rc<RefCell<Task>>>>, // for fast lookup
    pub questionTasks:Vec<Box<Task2>>,

    pub mem: Rc<RefCell<NarMem::Mem>>,
    pub stampIdCounter: i64, // counter for stamp id
    pub taskIdCounter: i64, // counter for id of task, mainly used for fast checking if two tasks are the same!

    pub cycleCounter: i64, // counter for done reasoning cycles

    pub rng: ThreadRng,
}



// helper to select random task by credit
pub fn taskSelByCreditRandom(selVal:f64, arr: &Vec<Rc<RefCell<Task>>>, cycleCounter:i64)->usize {
    let sum:f64 = arr.iter().map(|iv| taskCalcCredit(&iv.borrow(), cycleCounter)).sum();
    let mut acc = 0.0;
    let mut idx = 0;
    for iv in arr {
        acc += taskCalcCredit(&iv.borrow(), cycleCounter);
        if acc >= selVal*sum {
            return idx;
        }
        idx+=1;
    }
    
    arr.len()-1 // sel last
}

pub fn task2SelByCreditRandom(selVal:f64, arr: &Vec<Box<Task2>>)->usize {
    let sum:f64 = arr.iter().map(|iv| (*iv).prio).sum();
    let mut acc = 0.0;
    let mut idx = 0;
    for iv in arr {
        acc += (*iv).prio;
        if acc >= selVal*sum {
            return idx;
        }
        idx+=1;
    }
    
    arr.len()-1 // sel last
}

// helper to select random belief by AV
// expect that the arr isn't question!
pub fn conceptSelByAvRandom(selVal:f64, arr: &Vec<Arc<SentenceDummy>>)->usize {
    let sum:f64 = arr.iter().map(|iv| {
        if iv.punct == EnumPunctation::QUESTION {panic!("TV expected!");}; // questions don't have TV as we need confidence!
        retTv(&*iv).unwrap().c
    }).sum();
    let mut acc = 0.0;
    let mut idx = 0;
    for iv in arr {
        if iv.punct == EnumPunctation::QUESTION {panic!("TV expected!");}; // questions don't have TV as we need confidence!

        acc += retTv(&*iv).unwrap().c;
        if acc >= selVal*sum {
            return idx;
        }
        idx+=1;
    }
    
    arr.len()-1 // sel last
}

// helper to select task with highest prio
pub fn tasksSelHighestCreditIdx(arr: &Vec<Rc<RefCell<Task>>>, cycleCounter:i64) -> Option<usize> {
    if arr.len() == 0 {
        return None;
    }
    let mut idxRes = 0;
    let mut res = Rc::clone(&arr[0]);
    for idx in 1..arr.len() {
        let sel = &arr[idx];
        if taskCalcCredit(&sel.borrow(), cycleCounter) > taskCalcCredit(&res.borrow(), cycleCounter) {
            res = Rc::clone(&sel);
            idxRes = idx;
        }
    }
    Some(idxRes)
}


// stores missing entries of mem.judgementTasksByTerm by subterm of term
//
// IMPL< is actually a helper function for memAddTask, still exposed as public for code reuse >
pub fn populateTaskByTermLookup(mem:&mut Mem2, term:&Term, task:&Rc<RefCell<Task>>) {
    for iSubTerm in &retSubterms(&term) {
        if mem.judgementTasksByTerm.contains_key(iSubTerm) {
            let mut v = mem.judgementTasksByTerm.get(iSubTerm).unwrap().clone();
            v.push(Rc::clone(&task));
            mem.judgementTasksByTerm.insert(iSubTerm.clone(), v);
        }
        else {
            mem.judgementTasksByTerm.insert(iSubTerm.clone(), vec![Rc::clone(&task)]);
        }
    }
}

// /param calcCredit compute the credit?
pub fn memAddTask(mem:&mut Mem2, sentence:&SentenceDummy, calcCredit:bool) {
    // try to revise
    let mut wasRevised = false;
    match sentence.punct {
        EnumPunctation::JUGEMENT => {
            
            for iTerm in retSubterms(&*sentence.term) { // enumerate all terms, we need to do this to add the sentence to all relevant names
                match ((*mem.mem).borrow_mut()).concepts.get_mut(&iTerm.clone()) {
                    Some(arcConcept) => {
                        match Arc::get_mut(arcConcept) {
                            Some(concept) => {
                                let mut delBeliefIdx:Option<usize> = None;
                                
                                for iBeliefIdx in 0..concept.beliefs.len() {
                                    let iBelief = &concept.beliefs[iBeliefIdx];
                                    if checkEqTerm(&iBelief.term, &sentence.term) && !NarStamp::checkOverlap(&iBelief.stamp, &sentence.stamp) {
                                        let stamp = NarStamp::merge(&iBelief.stamp, &sentence.stamp);
                                        let tvA:Tv = retTv(&concept.beliefs[iBeliefIdx]).unwrap();
                                        let tvB:Tv = retTv(&sentence).unwrap();
                                        let evi:Evidence = Evidence::TV(rev(&tvA,&tvB));
                                        
                                        delBeliefIdx = Some(iBeliefIdx);
                                        concept.beliefs.push(Arc::new(SentenceDummy {
                                            term:iBelief.term.clone(),
                                            t:iBelief.t,
                                            punct:iBelief.punct,
                                            stamp:stamp,
                                            expDt:iBelief.expDt, // exponential time delta, used for =/>
                                            evi:Some(evi),
                                        })); // add revised belief!

                                        wasRevised = true;
                                        break; // breaking here is fine, because belief should be just once in table!
                                    }
                                }

                                if delBeliefIdx.is_some() {
                                    concept.beliefs.remove(delBeliefIdx.unwrap());
                                }
                            }
                            None => {
                                println!("INTERNAL ERROR - couldn't aquire arc!");
                            }
                        }
                    },
                    None => {}
                }
            }
        },
        _ => {}
    }

    if wasRevised {
        return;
    }
    // we are here if it can't revise
    
    NarMem::storeInConcepts(&mut mem.mem.borrow_mut(), sentence); // store sentence in memory, adressed by concepts
    

    match sentence.punct {
        EnumPunctation::JUGEMENT => {
            if true { // check if we should check if it already exist in the tasks
                for ijt in &mem.judgementTasks { // ijt:iteration-judgement-task
                    if checkEqTerm(&sentence.term, &ijt.borrow().sentence.term) && checkSame(&sentence.stamp, &ijt.borrow().sentence.stamp) {
                        return; // don't add if it exists already! because we would skew the fairness if we would add it
                    }
                }
            }

            let mut task = Task {
                sentence:sentence.clone(),
                credit:1.0,
                id:mem.taskIdCounter,
                derivTime:mem.cycleCounter
            };
            mem.taskIdCounter+=1;

            if calcCredit {
                divCreditByComplexity(&mut task); // punish more complicated terms
            }

            let x:RefCell<Task> = RefCell::new(task);
            let taskRc = Rc::new(x);
            mem.judgementTasks.push(Rc::clone(&taskRc));
            
            // populate hashmap lookup
            populateTaskByTermLookup(mem, &sentence.term, &taskRc);
        },
        EnumPunctation::QUESTION => {
            println!("TODO - check if we should check if it already exist in the tasks");
            
            mem.questionTasks.push(Box::new(Task2 {
                sentence:sentence.clone(),
                handler:None,
                bestAnswerExp:0.0, // because has no answer yet
                prio:1.0,
            }));
        },
        EnumPunctation::GOAL => {
            println!("ERROR: goal is not implemented!");
        },
    }
    
}

// helper for attention
pub fn divCreditByComplexity(task:&mut Task) {
    task.credit /= calcComplexity(&task.sentence.term) as f64;
}

// tries to find a better answer for a question task
// /param qTask the question task to find a answer to
// /param concl candidate answer to get evaluated
pub fn qaTryAnswer(qTask: &mut Task2, concl: &SentenceDummy) {
    if concl.punct != EnumPunctation::JUGEMENT { // only jugements can answer questions!
        return;
    }

    if calcExp(&retTv(concl).unwrap()) > qTask.bestAnswerExp { // is the answer potentially better?
        let unifyRes: Option<Vec<Asgnment>> = unify(&qTask.sentence.term, &concl.term); // try unify question with answer
        if unifyRes.is_some() { // was answer found?
            let _unifiedRes: Term = unifySubst(&qTask.sentence.term, &unifyRes.unwrap());
            
            if qTask.handler.is_some() {
                let handler1 = qTask.handler.as_ref().unwrap();
                let mut handler2 = handler1.borrow_mut();
                handler2.answer(&qTask.sentence.term, &concl); // call callback because we found a answer
            }

            qTask.bestAnswerExp = calcExp(&retTv(&concl).unwrap()); // update exp of best found answer

            // print question and answer
            let msg = "TRACE answer: ".to_owned() + &convSentenceTermPunctToStr(&qTask.sentence, true) + " " + &convSentenceTermPunctToStr(&concl, true);
            println!("{}", msg);
        }
    }
}

// performs one reasoning cycle
// /param cycleCounter counter of done cycles of reasoner
pub fn reasonCycle(mem:&mut Mem2) {
    mem.cycleCounter+=1;

    // transfer credits from questionTasks to Judgement tasks
    for iTask in &mem.questionTasks {
        {
            for iSubTerm in &retSubterms(&iTask.sentence.term) { // iterate over all terms
                let optTasksBySubterm = mem.judgementTasksByTerm.get(&iSubTerm);
                match optTasksBySubterm {
                    Some(tasksBySubterms) => {
                        for iIdx in 0..tasksBySubterms.len() {
                            let x:&RefCell<Task> = &(*tasksBySubterms[iIdx]);
                            x.borrow_mut().credit += (*iTask).prio;
                        }
                    },
                    None => {},
                }
            } 
        }
    }

    // give base credit
    // JUSTIFICATION< else the tasks die down for forward inference >
    for iIdx in 0..mem.judgementTasks.len() {
        mem.judgementTasks[iIdx].borrow_mut().credit += 0.5;
    }
    
    // let them pay for their complexity
    for iIdx in 0..mem.judgementTasks.len() {
        divCreditByComplexity(&mut *mem.judgementTasks[iIdx].borrow_mut());
    }

    // sample question to answer
    {
        if mem.questionTasks.len() > 0 {
            let selVal:f64 = mem.rng.gen_range(0.0,1.0);
            let qIdx = task2SelByCreditRandom(selVal, &mem.questionTasks);
            let mut selTask = &mut mem.questionTasks[qIdx];

            // * enumerate subterms
            for iSubTerm in &retUniqueSubterms(&(*selTask).sentence.term.clone()) {

                // * retrieve concept by subterm
                match mem.mem.borrow_mut().concepts.get_mut(&iSubTerm) {
                    Some(arcConcept) => {
                        match Arc::get_mut(arcConcept) {
                            Some(concept) => {
                                // try to answer question with all beliefs which may be relevant
                                for iBelief in &concept.beliefs {
                                    qaTryAnswer(&mut selTask, &iBelief);
                                }
                            }
                            None => {
                                println!("INTERNAL ERROR - couldn't aquire arc!");
                            }
                        }
                    },
                    None => {} // concept doesn't exist, ignore
                }

            }
        }
    }
    
    let mut concl: Vec<SentenceDummy> = vec![];

    if mem.judgementTasks.len() > 0 { // one working cycle - select for processing
        let selVal:f64 = mem.rng.gen_range(0.0,1.0);
        let selIdx = taskSelByCreditRandom(selVal, &mem.judgementTasks, mem.cycleCounter);

        let selPrimaryTask = &mem.judgementTasks[selIdx];
        let selPrimaryTaskTerm = selPrimaryTask.borrow().sentence.term.clone();

        { // single premise derivation
            let mut concl2: Vec<SentenceDummy> = infSinglePremise2(&selPrimaryTask.borrow().sentence);
            concl.append(&mut concl2);
        }

        {
            // attention mechanism which select the secondary task from the table 
            
            let mut secondaryElligable:Vec<Rc<RefCell<Task>>> = vec![]; // tasks which are eligable to get selected as the secondary
            
            for iSubTerm in &retUniqueSubterms(&selPrimaryTask.borrow().sentence.term.clone()) {
                if mem.judgementTasksByTerm.contains_key(iSubTerm) {
                    let itJudgementTasksByTerm:Vec<Rc<RefCell<Task>>> = mem.judgementTasksByTerm.get(iSubTerm).unwrap().to_vec();
                    for it in &itJudgementTasksByTerm {// append to elligable, because it contains the term
                        
                        // code to figure out if task already exists in secondaryElligable
                        let mut existsById = false;
                        {
                            for iSec in &secondaryElligable {
                                if iSec.borrow().id == it.borrow().id {
                                    existsById = true;
                                    break; // OPT
                                }
                            }
                        }
                        
                        if !existsById {
                            secondaryElligable.push(Rc::clone(&it));
                        }
                    }
                }
            }

            { // filter secondary elligable 
                /*
        the selection of secondary premise should consider the structure of the primary premise.
        Motivation for this is a higher efficiency of the deriver by prefering to select premises which can lead to conclusions.
        
        cases
        a) primary is not <=> or ==>
           consider secondary only if
           a.1) secondary is of form &&==> or &&<=> and sub-term of && unifies with term of primary
                reason: deriver should prefer to unify terms to "cut down" the conj
                ex:
                   primary  : <a --> b>
                   secondary: (<$0 --> b>&&<z --> Z>) ==> <Z --> B>
           a.2) secondary is of form ==> or <=> without && and subject unfies with term of primary
                ex:
                   primary  : <a --> b>
                   secondary: <$0 --> b> ==> <Z --> B>
           a.3) secondary is not of form <=> or ==>
                reason: non-NAL-5&6 derivation!

        b) primary is <=> or ==>
           consider secondary!
                */

                // NOTE< 08:00 08.08.2020 : is disabled because I am searching for a stupid bug which prevents inference >
                // NOTE< 09:00 08.08.2020 : is disabled because it is not necessary with ALANN's method to select all candidates >
                let enFunctionalityNal5PremiseFiler1 = false; // do we enable filtering mechanism to make &&==> and &&<=> inference more efficient?

                if enFunctionalityNal5PremiseFiler1 {

                    let mut secondaryElligableFiltered = vec![];

                    println!("TRACE  primary term = {}", convTermToStr(&selPrimaryTask.borrow().sentence.term));
    
                    for iSecondaryElligable in &secondaryElligable {
                        println!("TRACE   consider secondary term = {}", convTermToStr(&iSecondaryElligable.borrow().sentence.term));
                        
                        match *selPrimaryTask.borrow().sentence.term { // is primary <=> or ==>
                            Term::Stmt(cop,_,_) if cop == Copula::IMPL || cop == Copula::EQUIV => {
                                secondaryElligableFiltered.push(Rc::clone(iSecondaryElligable)); // consider
                                println!("TRACE      ...   eligable, reason: primary is ==> or <=> !");
                                continue;
                            },
                            _ => {}
                        }
    
                        // else special handling
                        match (*iSecondaryElligable.borrow().sentence.term).clone() {
                            Term::Stmt(cop,secondarySubj,_) if cop == Copula::IMPL || cop == Copula::EQUIV => {
                                match *secondarySubj {
                                    Term::Conj(conjterms) => {
                                        // we need to check if conjterm unifies with primary
                                        let mut anyUnify = false;
                                        for iConjTerm in &conjterms {
                                            if unify(&selPrimaryTask.borrow().sentence.term.clone(), &iConjTerm).is_some() {
                                                anyUnify = true;
                                                break;
                                            }
                                        }
    
                                        if anyUnify {
                                            secondaryElligableFiltered.push(Rc::clone(iSecondaryElligable)); // consider
                                            println!("TRACE      ...   eligable, reason: secondary is &&==> or &&<=> and subterm of && unifies!");
                                            continue;
                                        }
                                    },
                                    _ => {
                                        // we need to check if secondarySubj unfies with primary
                                        if unify(&selPrimaryTask.borrow().sentence.term.clone(), &secondarySubj).is_some() {
                                            secondaryElligableFiltered.push(Rc::clone(iSecondaryElligable)); // consider
                                            println!("TRACE      ...   eligable, reason: secondary is &&==> or &&<=> and subterm of && unifies!");
                                            continue;
                                        }
                                    }
                                }
                            },
                            _ => {
                                secondaryElligableFiltered.push(Rc::clone(iSecondaryElligable)); // consider
                                println!("TRACE      ...   eligable, reason: non-NAL-5&6 derivation!");
                            }
                        }
                    }
                    secondaryElligable = secondaryElligableFiltered;




                }
                
            }

            if secondaryElligable.len() > 0 { // must contain any premise to get selected
                let dbgSecondaryElligable = false; // do we want to debug elligable secondary tasks?
                if dbgSecondaryElligable {
                    println!("TRACE secondary eligable:");
                    for iSecondaryElligable in &secondaryElligable {
                        println!("TRACE    {}", convSentenceTermPunctToStr(&iSecondaryElligable.borrow().sentence, true));
                    }
                }
                
                let enInferenceSampleSecondaryByCredit = false; // do we sample secondary premise randomly by credit?
                let enInferenceSecondaryAll = true; // do we select and process all secondary premises (like in ALANN)

                if enInferenceSampleSecondaryByCredit { // sample secondary premise randomly by credit?
                    // sample from secondaryElligable by priority
                    let selVal:f64 = mem.rng.gen_range(0.0,1.0);
                    let secondarySelTaskIdx = taskSelByCreditRandom(selVal, &secondaryElligable, mem.cycleCounter);
                    let secondarySelTask: &Rc<RefCell<Task>> = &secondaryElligable[secondarySelTaskIdx];

                    // debug premises
                    {
                        println!("TRACE do inference...");

                        {
                            let taskSentenceAsStr = convSentenceTermPunctToStr(&selPrimaryTask.borrow().sentence, false);
                            println!("TRACE   primary   task  {}  credit={}", taskSentenceAsStr, taskCalcCredit(&selPrimaryTask.borrow(), mem.cycleCounter));    
                        }
                        {
                            let taskSentenceAsStr = convSentenceTermPunctToStr(&secondarySelTask.borrow().sentence, false);
                            println!("TRACE   secondary task  {}  credit={}", taskSentenceAsStr, taskCalcCredit(&secondarySelTask.borrow(), mem.cycleCounter));
                        }
                    }

                    // do inference with premises
                    let mut wereRulesApplied = false;
                    let mut concl2: Vec<SentenceDummy> = inference(&selPrimaryTask.borrow().sentence, &secondarySelTask.borrow().sentence, &mut wereRulesApplied);
                    concl.append(&mut concl2);
                }

                if enInferenceSecondaryAll {
                    for iSecondaryTask in &secondaryElligable {
                        // do inference and add conclusions to array
                        let mut wereRulesApplied = false;
                        let mut concl2: Vec<SentenceDummy> = inference(&selPrimaryTask.borrow().sentence, &iSecondaryTask.borrow().sentence, &mut wereRulesApplied);
                        concl.append(&mut concl2);
                    }
                }
            }
        }

        { // attention mechanism which selects the secondary task from concepts
            match mem.mem.borrow_mut().concepts.get_mut(&selPrimaryTaskTerm) {
                Some(arcConcept) => {
                    match Arc::get_mut(arcConcept) {
                        Some(concept) => {
                            println!("sample concept {}", convTermToStr(&*concept.name));

                            let processAllBeliefs:bool = true; // does the deriver process all beliefs?
                            let processSampledBelief:bool = false; // does it just sample one belief?

                            if processAllBeliefs { // code for processing all beliefs! is slower but should be more complete
                                for iBelief in &concept.beliefs {
                                    // do inference and add conclusions to array
                                    let mut wereRulesApplied = false;
                                    let mut concl2: Vec<SentenceDummy> = inference(&selPrimaryTask.borrow().sentence, iBelief, &mut wereRulesApplied);
                                    concl.append(&mut concl2);
                                }
                            }
                            if processSampledBelief { // code for sampling, is faster
                                // sample belief from concept
                                let selVal:f64 = mem.rng.gen_range(0.0,1.0);
                                let selBeliefIdx:usize = conceptSelByAvRandom(selVal, &concept.beliefs);
                                let selBelief:&SentenceDummy = &concept.beliefs[selBeliefIdx];

                                // do inference and add conclusions to array
                                let mut wereRulesApplied = false;
                                let mut concl2: Vec<SentenceDummy> = inference(&selPrimaryTask.borrow().sentence, selBelief, &mut wereRulesApplied);
                                concl.append(&mut concl2);
                            }
                        }
                        None => {
                            println!("INTERNAL ERROR - couldn't aquire arc!");
                        }
                    }
                },
                None => {} // concept doesn't exist, ignore
            }
        }
    }


    // put conclusions back into memory!
    {
        // Q&A - answer questions
        {
            for iConcl in &concl {
                if iConcl.punct == EnumPunctation::JUGEMENT { // only jugements can answer questions!
                    for mut iQTask in &mut mem.questionTasks {
                        qaTryAnswer(&mut iQTask, &iConcl);
                    }
                }
            }
        }
        

        for iConcl in &concl {
            // TODO< check if task exists already, don't add if it exists >
            memAddTask(mem, iConcl, true);
        }
    }

    let intervalCheckTasks = 111; // cycle counter to check for AIKR of tasks - should be prime
    let maxJudgementTasks = 30; // maximal number of judgement tasks

    // keep working tasks of judgements under AIKR
    {
        if mem.cycleCounter % intervalCheckTasks == 0 //&& mem.judgementTasks.len() > maxJudgementTasks //// commented for testing
        {
            let memCycleCounter:i64 = mem.cycleCounter;
            mem.judgementTasks.sort_by(|a, b| 
                taskCalcCredit(&b.borrow(), memCycleCounter).partial_cmp(
                    &taskCalcCredit(&a.borrow(), memCycleCounter)
                ).unwrap());
            mem.judgementTasks = mem.judgementTasks[0..maxJudgementTasks.min(mem.judgementTasks.len())].to_vec(); // limit to keep under AIKR
        }
    }

    // keep judgement tasks by term under AIKR
    {
        if mem.cycleCounter % intervalCheckTasks == 0 {
            mem.judgementTasksByTerm = HashMap::new(); // flush, because we will repopulate

            // repopulate judgementTasksByTerm
            // IMPL< we had to split it because mem was accessed twice! >
            let mut termAndTask = vec![];
            for iJudgementTask in &mem.judgementTasks {
                let termRc:&Rc<Term> = &iJudgementTask.borrow_mut().sentence.term;
                let term:Term = (**termRc).clone();

                termAndTask.push((term, Rc::clone(iJudgementTask)));
            }

            for (term, task) in &termAndTask { // iterate over prepared tuples
                // populate hashmap lookup
                populateTaskByTermLookup(mem, &term, &task);
            }
        }
    }


}

pub fn createMem2()->Mem2 {
    let mem0:NarMem::Mem = NarMem::Mem{
        concepts:HashMap::new(),
    };
    
    Mem2{judgementTasks:vec![], judgementTasksByTerm:HashMap::new(), questionTasks:vec![], mem:Rc::new(RefCell::new(mem0)), rng:rand::thread_rng(), stampIdCounter:0, taskIdCounter:1000, // high number to easy debugging to prevent confusion
        cycleCounter:0,
    }
}

// not working prototype of attention mechanism based on credits
pub fn expNarsWorkingCycle0() {
    // TODO< create and fill concepts! by sentence when storing sentence into memory >
    let mut mem:Mem2 = createMem2();
    
    // add testing tasks
    {
        { // .
            let sentence = SentenceDummy {
                //isOp:false, // is it a operation?
                term:Rc::new(Term::Stmt(Copula::INH, Box::new(Term::Name("a".to_string())), Box::new(Term::Name("b".to_string())))),
                t:None, // time of occurence 
                punct:EnumPunctation::JUGEMENT,
                stamp:newStamp(&vec![0]),
                evi:Some(Evidence::TV(Tv{f:1.0,c:0.9})),
                expDt:None
            };
            memAddTask(&mut mem, &sentence, true);
        }

        { // .
            let sentence = SentenceDummy {
                //isOp:false, // is it a operation?
                term:Rc::new(Term::Stmt(Copula::INH, Box::new(Term::Name("b".to_string())), Box::new(Term::Name("c".to_string())))),
                t:None, // time of occurence 
                punct:EnumPunctation::JUGEMENT,
                stamp:newStamp(&vec![1]),
                evi:Some(Evidence::TV(Tv{f:1.0,c:0.9})),
                expDt:None
            };
            memAddTask(&mut mem, &sentence, true);
        }

        { // ?
            let sentence = SentenceDummy {
                //isOp:false, // is it a operation?
                term:Rc::new(Term::Stmt(Copula::INH, Box::new(Term::Name("a".to_string())), Box::new(Term::Name("c".to_string())))),
                t:None, // time of occurence 
                punct:EnumPunctation::QUESTION,
                stamp:newStamp(&vec![2]),
                evi:None,
                expDt:None
            };
            memAddTask(&mut mem, &sentence, true);
        }
    }

    reasonCycle(&mut mem);

    debugCreditsOfTasks(&mut mem);
}

pub fn debugCreditsOfTasks(mem: &Mem2) {
    // debug credit of tasks
    {
        for iTask in &mem.judgementTasks {
            let taskSentenceAsStr = convSentenceTermPunctToStr(&iTask.borrow().sentence, true);
            
            let mut taskAsStr = taskSentenceAsStr.clone();

            let printStamp = true;
            if printStamp {
                taskAsStr = format!("{} {}", taskAsStr, NarStamp::convToStr(&iTask.borrow().sentence.stamp));
            }

            println!("task  {}  credit={}", taskAsStr, taskCalcCredit(&iTask.borrow(), mem.cycleCounter));
        }
    }
}












// called when answer is found
pub trait QHandler {
    fn answer(&mut self, question:&Term, answer:&SentenceDummy);
}
