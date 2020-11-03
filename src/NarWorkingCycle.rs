//! implementation of basic working cycle (for declarative reasoner)

// TODO< select highest ranked task, remove it from array, select other task by priority distribution, do inference, put results into memory >
//     TODO< put processed task into randomly sampled bag-table! >



// TODO< add question variable >

use std::sync::mpsc;
use std::thread;
use rand::Rng;
use rand::rngs::ThreadRng;

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
use crate::NarUnify::*;

/* commented because not needed
/// a --> b |- b --> a
pub fn inf2(a: &Term, punct:EnumPunctation, aTv: &Tv) -> Option<(Term, Tv, EnumPunctation)> {
    //if punctA != EnumPunctation::JUGEMENT {
    //    return None;
    //}

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

/// (!a) |- a
pub fn infNeg(a: &Term, punct:EnumPunctation, aTv: &Option<Tv>) -> Option<(Term, Tv, EnumPunctation)> {
    if punct != EnumPunctation::JUGEMENT {
        return None;
    }

    match a {
        Term::Neg(aterm) => {
            return Some(((**aterm).clone(), neg(&aTv.as_ref().unwrap()), EnumPunctation::JUGEMENT));
        },
        _ => {},
    }
    None
}


/// structural
/// a --> (X | Y).
/// |-
/// a --> X.
/// a --> Y.
/// ...
pub fn infStructPred1(a: &Term, punct:EnumPunctation, aTv: &Option<Tv>, idx:usize) -> Option<(Term, Tv, EnumPunctation)> {
    if punct != EnumPunctation::JUGEMENT {
        return None;
    }

    match a {
        Term::Stmt(Copula::INH, subj, pred) => {
            match &**pred {
                Term::IntInt(arr) => {
                    if idx < arr.len() {
                        let concl = Term::Stmt(Copula::INH, Box::clone(subj), Box::clone(&arr[idx]));
                        return Some((concl,aTv.as_ref().unwrap().clone(),EnumPunctation::JUGEMENT));
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}

/// structural
/// (X | Y) --> a.
/// |-
/// X --> a.
/// Y --> a.
/// ...
pub fn infStructSubj1(a: &Term, punct:EnumPunctation, aTv: &Option<Tv>, idx:usize) -> Option<(Term, Tv, EnumPunctation)> {
    if punct != EnumPunctation::JUGEMENT {
        return None;
    }

    match a {
        Term::Stmt(Copula::INH, subj, pred) => {
            match &**subj {
                Term::IntInt(arr) => {
                    if idx < arr.len() {
                        let concl = Term::Stmt(Copula::INH, Box::clone(&arr[idx]), Box::clone(pred));
                        return Some((concl,aTv.as_ref().unwrap().clone(),EnumPunctation::JUGEMENT));
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}


/// structural
/// (X * Y) --> rel.
/// |-
/// <X --> (rel /1 Y)>.
/// <Y --> (rel /2 X)>.
pub fn infStructProd0(a: &Term, punct:EnumPunctation, aTv: &Option<Tv>) -> Option<(Term, Tv, EnumPunctation)> {
    if punct != EnumPunctation::JUGEMENT {
        return None;
    }

    match a {
        Term::Stmt(Copula::INH, prod, pred) => {
            match &**prod {
                Term::Prod(arr) => {
                    let prod0 = &arr[0];
                    let prod1 = &arr[1];

                    let concl = Term::Stmt(Copula::INH, Box::clone(&prod0), Box::new(Term::Img(Box::clone(pred), 0, vec![Box::clone(&prod1)])));
                    return Some((concl,aTv.as_ref().unwrap().clone(),EnumPunctation::JUGEMENT));
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}

/// structural
/// (X * Y) --> rel.
/// |-
/// <X --> (rel /1 Y)>.
/// <Y --> (rel /2 X)>.
pub fn infStructProd1(a: &Term, punct:EnumPunctation, aTv: &Option<Tv>) -> Option<(Term, Tv, EnumPunctation)> {
    if punct != EnumPunctation::JUGEMENT {
        return None;
    }

    match a {
        Term::Stmt(Copula::INH, prod, pred) => {
            match &**prod {
                Term::Prod(arr) => {
                    if arr.len() == 2 {
                        let prod0 = &arr[0];
                        let prod1 = &arr[1];
                        let concl = Term::Stmt(Copula::INH, Box::clone(&prod1), Box::new(Term::Img(Box::clone(pred), 1, vec![Box::clone(&prod0)])));
                        return Some((concl,aTv.as_ref().unwrap().clone(),EnumPunctation::JUGEMENT));
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}

/// structural
/// <X --> (rel /1 Y)>.?
/// |-
/// (X * Y) --> rel.?
pub fn infStructImg0(a: &Term, punct:EnumPunctation, aTv: &Option<Tv>) -> Option<(Term, Tv, EnumPunctation)> {
    if punct != EnumPunctation::JUGEMENT && punct != EnumPunctation::QUESTION {
        return None;
    }

    match a {
        Term::Stmt(Copula::INH, subj, pred) => {
            match &**pred {
                Term::Img(predImg,0,arr) => {
                    if arr.len() == 1 {
                        let arr0 = &arr[0];
                        let concl = Term::Stmt(Copula::INH, Box::new(Term::Prod(vec![Box::clone(&subj), Box::clone(&arr0)])), Box::clone(&predImg));
                        return Some((concl,aTv.as_ref().unwrap().clone(),EnumPunctation::JUGEMENT));
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}


/// structural
/// <Y --> (rel /2 X)>.?
/// |-
/// (X * Y) --> rel.?
pub fn infStructImg1(a: &Term, punct:EnumPunctation, aTv: &Option<Tv>) -> Option<(Term, Tv, EnumPunctation)> {
    if punct != EnumPunctation::JUGEMENT && punct != EnumPunctation::QUESTION {
        return None;
    }

    match a {
        Term::Stmt(Copula::INH, subj, pred) => {
            match &**pred {
                Term::Img(predImg,1,arr) => {
                    if arr.len() == 1 {
                        let arr0 = &arr[0];
                        let concl = Term::Stmt(Copula::INH, Box::new(Term::Prod(vec![Box::clone(&arr0), Box::clone(&subj)])), Box::clone(&predImg));
                        return Some((concl,aTv.as_ref().unwrap().clone(),EnumPunctation::JUGEMENT));
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
    None
}




/// see ONA
/// [a] <-> [b]. |- a <-> b.
pub fn infStructSetInt(a: &Term, punctA:EnumPunctation, aTv:&Option<Tv>) -> Option<(Term,Tv,EnumPunctation)> {
    if punctA != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::SIM, asubj2, apred2) => {
            match &**asubj2 {
                Term::SetInt(asubj) if asubj.len() == 1 => {
                    match &**apred2 {
                        Term::SetInt(apred) if apred.len() == 1 => {
                            println!("TODO - compute tv");
                            return Some((Term::Stmt(Copula::SIM, Box::clone(&asubj[0]), Box::clone(&apred[0])), aTv.as_ref().unwrap().clone(), EnumPunctation::JUGEMENT));
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
        },
        _ => {},
    }
    None
}

/// see ONA
/// {a} <-> {b}. |- a <-> b.
pub fn infStructSetExt(a: &Term, punctA:EnumPunctation, aTv:&Option<Tv>) -> Option<(Term,Tv,EnumPunctation)> {
    if punctA != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::SIM, asubj2, apred2) => {
            match &**asubj2 {
                Term::SetExt(asubj) if asubj.len() == 1 => {
                    match &**apred2 {
                        Term::SetExt(apred) if apred.len() == 1 => {
                            println!("TODO - compute tv");
                            return Some((Term::Stmt(Copula::SIM, Box::clone(&asubj[0]), Box::clone(&apred[0])), aTv.as_ref().unwrap().clone(), EnumPunctation::JUGEMENT));
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
        },
        _ => {},
    }
    None
}




/// generalized rule with two judgement premises
/// works only when conclusion is composed out of a and b
pub fn infGeneralizedJudgJudg(
    a: &Term, punctA:EnumPunctation, aTv:&Option<Tv>, 
    b: &Term, punctB:EnumPunctation, bTv:&Option<Tv>,

    aCopula: Copula,
    bCopula: Copula,
    conclCopula: Copula,

    aSide:i32,
    bSide:i32,
    tvFn: fn(&Tv, &Tv) -> Tv // function for conclusion TV computation
) -> Option<(Term,Tv,EnumPunctation)> {
    // helper to select subj or pred based on number (side)
    fn sel<'a>(subj:&'a Box<Term>,pred:&'a Box<Term>,side:i32)->&'a Box<Term> {
        if side == 1 {subj} else {pred}
    }


    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(acop, asubj, apred) if *acop == aCopula => {
            match b {
                Term::Stmt(bcop, bsubj, bpred) if *bcop == bCopula => {
                    if 
                        checkEqTerm(sel(&asubj,&apred,aSide), sel(&bsubj,&bpred,bSide)) && // sides must be the same term
                        !checkEqTerm(sel(&asubj,&apred,-aSide), sel(&bsubj,&bpred,-bSide)) // other sides must not be equal!
                    {
                        return Some(( Term::Stmt(conclCopula, Box::clone(sel(&asubj,&apred,-aSide)), Box::clone(sel(&bsubj,&bpred,-bSide))), tvFn(&aTv.as_ref().unwrap(),&bTv.as_ref().unwrap()), EnumPunctation::JUGEMENT));
                    }
                },
                _ => {},
            }
        },
        _ => {},
    }
    None
}




/// a --> x.  a --> y.  |- x <-> y.
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


/// x --> a.  y --> a.  |- x <-> y.
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


/// x --> [a].  x --> [b].  |- x --> [a b].
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


/// {a} --> x.  {b} --> x.  |- {a b} --> x.
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



/// see https://cis.temple.edu/~pwang/Writing/NAL-Specification.pdf?page=50
/// (a && b) ==> x.
/// unify a.
/// |- ded
/// b ==> x.
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


/// see https://cis.temple.edu/~pwang/Writing/NAL-Specification.pdf?page=42
/// a ==> x.
/// unify a.
/// |- ded
/// x.
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

/// necessary for symbolic manipulation for example in https://github.com/orgs/NARS-team/teams/all/discussions/71
/// a ==> x?
/// unify x.
/// |-
/// a ==> x.
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



/// do binary inference
pub fn infBinaryInner(a: &Term, aPunct:EnumPunctation, aTv:&Option<Tv>, b: &Term, bPunct:EnumPunctation, bTv:&Option<Tv>, wereRulesApplied:&mut bool) -> Vec<(Term,Tv,EnumPunctation)> {
    let mut res = vec![];
    
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
    match infGeneralizedJudgJudg( // S =/> M, M =/> P |-ded S =/> P
        &a, aPunct, &aTv, &b, bPunct, &bTv,

        Copula::PREDIMPL,
        Copula::PREDIMPL,
        Copula::PREDIMPL,
        -1,1,ded) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match infGeneralizedJudgJudg( // S ==> M, M ==> P |-ded S ==> P
        &a, aPunct, &aTv, &b, bPunct, &bTv,

        Copula::IMPL,
        Copula::IMPL,
        Copula::IMPL,
        -1,1,ded) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match infGeneralizedJudgJudg( // S --> M, M --> P |-ded S --> P
        &a, aPunct, &aTv, &b, bPunct, &bTv,

        Copula::INH,
        Copula::INH,
        Copula::INH,
        -1,1,ded) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match infGeneralizedJudgJudg( // S --> M, P --> M |-abd S --> P
        &a, aPunct, &aTv, &b, bPunct, &bTv,

        Copula::INH,
        Copula::INH,
        Copula::INH,
        -1,-1,abd) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match infGeneralizedJudgJudg( // M --> S, M --> P |-ind S --> P
        &a, aPunct, &aTv, &b, bPunct, &bTv,

        Copula::INH,
        Copula::INH,
        Copula::INH,
        1,1,ind) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    
    res
}

/// do binary inference
pub fn infBinary(a: &Term, aPunct:EnumPunctation, aTv:&Option<Tv>, b: &Term, bPunct:EnumPunctation, bTv:&Option<Tv>, wereRulesApplied:&mut bool) -> Vec<(Term,Tv,EnumPunctation)> {
    let mut res = vec![];
    *wereRulesApplied = false; // because no rules were applied yet
    res.extend(infBinaryInner(&a, aPunct, &aTv, &b, bPunct, &bTv, wereRulesApplied).iter().cloned());
    res.extend(infBinaryInner(&b, bPunct, &aTv, &a, aPunct, &bTv, wereRulesApplied).iter().cloned());
    res
}

pub fn infSinglePremise(a: &Term, punct:EnumPunctation, aTv:&Option<Tv>) -> Vec<(Term,Tv,EnumPunctation)> {
    let mut res = vec![];

    match infNeg(&a, punct, &aTv) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructSubj1(&a, punct, &aTv, 0) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructSubj1(&a, punct, &aTv, 1) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructSubj1(&a, punct, &aTv, 2) {
        Some(x) => { res.push(x); } _ => {}
    }

    match infStructPred1(&a, punct, &aTv, 0) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructPred1(&a, punct, &aTv, 1) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructPred1(&a, punct, &aTv, 2) {
        Some(x) => { res.push(x); } _ => {}
    }

    match infStructProd0(&a, punct, &aTv) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructProd1(&a, punct, &aTv) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructImg0(&a, punct, &aTv) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructImg1(&a, punct, &aTv) {
        Some(x) => { res.push(x); } _ => {}
    }

    match infStructSetInt(&a, punct, &aTv) {
        Some(x) => { res.push(x); } _ => {}
    }
    match infStructSetExt(&a, punct, &aTv) {
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


/// do inference of two sentences
/// /param wereRulesApplied is true if any rules were applied
pub fn inference2(
    paTerm:&Term, paPunct:EnumPunctation, paStamp:&Stamp, paTv:&Option<Tv>,  
    pbTerm:&Term, pbPunct:EnumPunctation, pbStamp:&Stamp, pbTv:&Option<Tv>, 
    wereRulesApplied:&mut bool
)->Vec<SentenceDummy> {
    *wereRulesApplied = false;

    let mut concl = vec![];

    let infConcl = infBinary(&paTerm, paPunct, paTv, &pbTerm, pbPunct, pbTv, wereRulesApplied);
    for iInfConcl in infConcl {
        let (term, tv, punct) = iInfConcl;
        concl.push(SentenceDummy{
            term:Arc::new(term.clone()),
            evi:if true {Some(Evidence::TV(tv.clone()))} else {None},
            stamp:merge(&paStamp, &pbStamp),
            t:None, // time of occurence 
            punct:punct,
            expDt:None
        });
    }

    if concl.len() > 0 && checkOverlap(&paStamp, &pbStamp) { // check for overlap
      concl = vec![]; // flush conclusions because we don't have any conclusions when the premises overlapped
    }
    
    concl
}


/// do inference of two sentences
/// /param wereRulesApplied is true if any rules were applied
pub fn inference(pa:&SentenceDummy, pb:&SentenceDummy, wereRulesApplied:&mut bool)->Vec<SentenceDummy> {
    inference2(
        &pa.term, pa.punct, &pa.stamp, &retTv(&pa),  
        &pb.term, pb.punct, &pb.stamp, &retTv(&pb), 
        wereRulesApplied
    )
}

pub fn infSinglePremise2(pa:&SentenceDummy) -> Vec<SentenceDummy> {
    let mut concl = vec![];

    let infConcl = infSinglePremise(&pa.term, pa.punct, &retTv(pa));
    for iInfConcl in infConcl {
        let (term, tv, punct) = iInfConcl;
        
        concl.push(SentenceDummy{
            term:Arc::new(term.clone()),
            evi:if true {Some(Evidence::TV(tv.clone()))} else {None},
            stamp:pa.stamp.clone(),
            t:None, // time of occurence 
            punct:punct,
            expDt:None
        });
    }

    concl
}

/// judgement task
pub struct Task {
    pub sentence:SentenceDummy,
    /// how much "worth" is the task for the system
    pub credit:f64,
    /// create assigned for Q&A
    pub qaCredit:f64,
    /// unique id to quickly find unique tasks
    pub id:i64,
    /// time when this task was put into the working table
    pub derivTime:i64,
}

/// compute "real" credit of task by insertion based time decay
pub fn taskCalcCredit(task:&Task, cycleCounter:i64) -> f64 {
    let decayFactor:f64 = 0.001; // how fast does it decay?
    
    let dt:i64 = cycleCounter - task.derivTime;
    let decayFactor:f64 = (-decayFactor * (dt as f64)).exp();

    let qaCredit:f64 = task.qaCredit*0.2; // limit Q&A credit to a low range, to give other tasks a higher chance
    (qaCredit + task.credit)*decayFactor // multiply because we want to decay the actual "base credit"
}

/// task for a question
pub struct Task2 {
    pub sentence:SentenceDummy,
    /// handler which is called when a better answer is found
    pub handler:Option< Arc<RwLock< dyn QHandler>> >,
    /// expectation of best answer
    pub bestAnswerExp:f64,
    /// priority
    pub prio:f64,
}


/// stores the message for the actual work
pub struct DeriverWorkMessage {
    pub primary: Arc<Mutex<Task>>,
    pub secondary: Vec<Arc<Mutex<Task>>>,
    pub cycleCounter: i64,
}

use std::collections::HashMap;
use std::cell::{RefCell};
use parking_lot::RwLock;
use std::thread::JoinHandle;
use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::atomic::{AtomicI64, Ordering};

/// shared (memory) state of declarative memory, accessed and modified by worker threads
/// all other memory is in nonshared portion!
pub struct DeclarativeShared {
    // TODO< put these two into one structure which is protected by one RwLock >
    pub judgementTasks:Vec<Arc<Mutex<Task>>>,
    pub judgementTasksByTerm:Arc<RwLock< HashMap<Term, Vec<Arc<Mutex<Task>>>> >>, // for fast lookup

    pub questionTasks:Arc<RwLock< Vec<Box<Task2>> >>,

    pub mem: Arc<RwLock<NarMem::Mem>>,
    /// counter for stamp id
    pub stampIdCounter: AtomicI64,
    /// counter for id of task, mainly used for fast checking if two tasks are the same!
    pub taskIdCounter: Arc<AtomicI64>,
    
    /// counter for done reasoning cycles
    pub cycleCounter: i64,
}

/// memory of NAR for eternal beliefs
pub struct Mem2 {
    pub shared:Arc<RwLock<DeclarativeShared>>,

    /// global handlers for Q&A
    pub globalQaHandlers: Arc<RwLock<  Vec<Arc<RwLock< dyn QHandler>>>  >>,
    pub rng: RwLock<ThreadRng>,
    
    /// array of workers
    pub deriverWorkers: Vec<JoinHandle<()>>,
    /// sender to worker
    pub deriverWorkersTx: Vec<SyncSender<DeriverWorkMessage>>,
}

pub fn createMem2(cfg__nConceptBeliefs:usize)->Arc<RwLock<Mem2>> {
    let mem0:NarMem::Mem = NarMem::Mem{
        concepts:HashMap::new(),
    };
    let memArc:Arc<RwLock<NarMem::Mem>> = Arc::new(RwLock::new(mem0));

    let shared = DeclarativeShared {
        judgementTasks:vec![], 
        judgementTasksByTerm:Arc::new(RwLock::new(HashMap::new())), 

        questionTasks:Arc::new(RwLock::new(vec![])), 
        mem:Arc::clone(&memArc),
        stampIdCounter:AtomicI64::new(0), 
        taskIdCounter:Arc::new(AtomicI64::new(1000)), // high number to easy debugging to prevent confusion
        cycleCounter:0,
    };


    let mut res:Mem2 = Mem2{
        shared:Arc::new(RwLock::new(shared)),

        globalQaHandlers:Arc::new(RwLock::new(vec![])), 
        rng:RwLock::new(rand::thread_rng()),

        deriverWorkers:vec![],
        deriverWorkersTx:vec![],
    };
    let resArc:Arc<RwLock<Mem2>> = Arc::new(RwLock::new(res));

    { // create workers for derivation
        let (tx, rx) = sync_channel(4); // create channel with fixed size, reason is that we want to limit backlog!
        resArc.write().deriverWorkersTx.push(tx);

        let sharedArc:Arc<RwLock<DeclarativeShared>> = Arc::clone(&resArc.read().shared);
        let globalQaHandlers = Arc::clone(&resArc.read().globalQaHandlers);
        let cfg__nConceptBeliefs = cfg__nConceptBeliefs;
        resArc.write().deriverWorkers.push(thread::spawn(move|| {
            let cfgEnInstrumentation = true;
            let mut rng = rand::thread_rng();

            loop {
                let msgRes = rx.recv();
                if !msgRes.is_ok() {
                    break; // other side has hung up, terminate this worker
                }
                let msg:DeriverWorkMessage = msgRes.unwrap(); // receive message
                println!("[WORKER] received MSG!");

                /////////
                // DERIVE
                /////////
                let mut concl:Vec<SentenceDummy> = vec![];

                { // single premise derivation
                    let mut concl2: Vec<SentenceDummy> = infSinglePremise2(&msg.primary.lock().unwrap().sentence);
                    concl.append(&mut concl2);
                }

                let enInferenceSampleSecondaryByCredit = false; // do we sample secondary premise randomly by credit?
                let enInferenceSecondaryAll = true; // do we select and process all secondary premises (like in ALANN)

                if enInferenceSampleSecondaryByCredit { // sample secondary premise randomly by credit?
                    // sample from secondaryElligable by priority
                    let selVal:f64 = rng.gen_range(0.0,1.0);
                    let secondarySelTaskIdx = taskSelByCreditRandom(selVal, &msg.secondary, msg.cycleCounter);
                    let secondarySelTask: &Arc<Mutex<Task>> = &msg.secondary[secondarySelTaskIdx];

                    // debug premises
                    {
                        println!("TRACE do inference...");

                        {
                            let taskSentenceAsStr = convSentenceTermPunctToStr(&msg.primary.lock().unwrap().sentence, false);
                            //println!("TRACE   primary   task  {}  credit={}", taskSentenceAsStr, taskCalcCredit(&selPrimaryTask.lock().unwrap(), mem.cycleCounter));    
                        }
                        {
                            let taskSentenceAsStr = convSentenceTermPunctToStr(&secondarySelTask.lock().unwrap().sentence, false);
                            //println!("TRACE   secondary task  {}  credit={}", taskSentenceAsStr, taskCalcCredit(&secondarySelTask.lock().unwrap(), mem.cycleCounter));
                        }
                    }

                    // do inference with premises
                    let mut wereRulesApplied = false;
                    let mut concl2: Vec<SentenceDummy> = inference(&msg.primary.lock().unwrap().sentence, &secondarySelTask.lock().unwrap().sentence, &mut wereRulesApplied);
                    concl.append(&mut concl2);
                }


                if enInferenceSecondaryAll {
                    let timeStart = Instant::now();

                    let secondaryElligablePartA = &msg.secondary[..msg.secondary.len()/2];
                    let secondaryElligablePartB2 = msg.secondary[msg.secondary.len()/2..].to_vec();
                    let secondaryElligablePartB:Vec<(Term,EnumPunctation,Stamp,Option<Tv>)> = msg.secondary.iter().map(|s| {
                        let s2:&SentenceDummy = &s.lock().unwrap().sentence;
                        ((*s2.term).clone(), s2.punct, s2.stamp.clone(), retTv(&s2))
                    }).collect();

                    let selPrimarySentenceTuple;
                    {
                        let s2:&SentenceDummy = &msg.primary.lock().unwrap().sentence;
                        selPrimarySentenceTuple = ((*s2.term).clone(), s2.punct, s2.stamp.clone(), retTv(&s2))
                    }

                    let handleB = thread::spawn(move|| {
                        let mut res = vec![];
                        for iSecondarySentence in &secondaryElligablePartB {
                            let mut wereRulesApplied = false;
                            let mut concl2: Vec<SentenceDummy> = inference2(
                                &selPrimarySentenceTuple.0, selPrimarySentenceTuple.1, &selPrimarySentenceTuple.2, &selPrimarySentenceTuple.3,
                                &iSecondarySentence.0, iSecondarySentence.1, &iSecondarySentence.2, &iSecondarySentence.3, 
                                &mut wereRulesApplied
                            );
                            res.append(&mut concl2);
                        }
                        res
                    });

                    let selPrimaryTaskSentence:&SentenceDummy = &msg.primary.lock().unwrap().sentence;
                    for iSecondaryTask in secondaryElligablePartA {
                        // do inference and add conclusions to array
                        if !Arc::ptr_eq(&msg.primary, &iSecondaryTask) { // arcs must not point to same task!
                            let mut wereRulesApplied = false;
                            let mut concl2: Vec<SentenceDummy> = inference(selPrimaryTaskSentence, &iSecondaryTask.lock().unwrap().sentence, &mut wereRulesApplied);
                            concl.append(&mut concl2);
                        }
                    }
                    
                    let mut conclPartB = handleB.join().unwrap();
                    concl.append(&mut conclPartB);

                    if cfgEnInstrumentation {
                        println!("[instr] secondard inf took {}us", timeStart.elapsed().as_micros());
                    }

                }



                { // attention mechanism which selects the secondary task from concepts
                    let keyTerm = msg.primary.lock().unwrap().sentence.term.clone();
                    match sharedArc.read().mem.read().concepts.get(&keyTerm) {
                        Some(concept) => {
                            println!("sample concept {}", convTermToStr(&concept.name));
        
                            let processAllBeliefs:bool = true; // does the deriver process all beliefs?
                            let processSampledBelief:bool = false; // does it just sample one belief?
        
                            if processAllBeliefs { // code for processing all beliefs! is slower but should be more complete
                                // MECHANISM<
                                // process of all revelant beliefs of a concept as the first premise with a selected belief as the second premise
                                // >
                                // TODO< limit secondary beliefs to keep reasoning strictly under AIKR >
                                for iBelief in &concept.beliefs {
                                    let iBeliefGuard = iBelief.read();
                                    // do inference and add conclusions to array
                                    let mut wereRulesApplied = false;
                                    let mut concl2: Vec<SentenceDummy> = inference(&msg.primary.lock().unwrap().sentence, &iBeliefGuard, &mut wereRulesApplied);
                                    concl.append(&mut concl2);
                                }
                            }
                            if processSampledBelief { // code for sampling, is faster
                                // MECHANISM<
                                // sample belief from concept
                                // This has the advantage that it's super cheap, but it can "hit" not fruitful premises
                                // >
                                let selVal:f64 = rng.gen_range(0.0,1.0);
                                let selBeliefIdx:usize = conceptSelByAvRandom(selVal, &concept.beliefs);
                                let selBelief:&SentenceDummy = &concept.beliefs[selBeliefIdx].read();
        
                                // do inference and add conclusions to array
                                let mut wereRulesApplied = false;
                                let mut concl2: Vec<SentenceDummy> = inference(&msg.primary.lock().unwrap().sentence, selBelief, &mut wereRulesApplied);
                                concl.append(&mut concl2);
                            }
                        },
                        None => {} // concept doesn't exist, ignore
                    }
                }




                ////////////
                // write back
                ////////////


                // put conclusions back into memory!
                {
                    // MECHANISM< Q&A - answer questions >
                    {
                        for iConcl in &concl {
                            if iConcl.punct == EnumPunctation::JUGEMENT { // only jugements can answer questions!
                                for mut iQTask in &mut *sharedArc.read().questionTasks.write() {
                                    qaTryAnswer(&mut iQTask, &iConcl, &globalQaHandlers.read());
                                }
                            }
                        }
                    }
                    
                    for iConcl in &concl {
                        // TODO< check if task exists already, don't add if it exists >
                        memAddTask(Arc::clone(&sharedArc), iConcl, true, cfg__nConceptBeliefs);
                    }
                }
            }
        }));
    }
    

    resArc
}

/// helper to select random task by credit
pub fn taskSelByCreditRandom(selVal:f64, arr: &Vec<Arc<Mutex<Task>>>, cycleCounter:i64)->usize {
    let sum:f64 = arr.iter().map(|iv| taskCalcCredit(&iv.lock().unwrap(), cycleCounter)).sum();
    let mut acc = 0.0;
    let mut idx = 0;
    for iv in arr {
        acc += taskCalcCredit(&iv.lock().unwrap(), cycleCounter);
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

/// helper to select random belief by AV
/// expect that the arr isn't question!
pub fn conceptSelByAvRandom(selVal:f64, arr: &Vec<Arc<RwLock<SentenceDummy>>>)->usize {
    let sum:f64 = arr.iter().map(|iv| {
        let ivGuard = iv.read();
        if ivGuard.punct == EnumPunctation::QUESTION {panic!("TV expected!");}; // questions don't have TV as we need confidence!
        retTv(&ivGuard).unwrap().c
    }).sum();
    let mut acc = 0.0;
    let mut idx = 0;
    for iv in arr {
        let ivGuard = iv.read();
        if ivGuard.punct == EnumPunctation::QUESTION {panic!("TV expected!");}; // questions don't have TV as we need confidence!

        acc += retTv(&ivGuard).unwrap().c;
        if acc >= selVal*sum {
            return idx;
        }
        idx+=1;
    }
    
    arr.len()-1 // sel last
}

/// helper to select task with highest prio
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


/// stores missing entries of mem.judgementTasksByTerm by subterm of term
///
/// IMPL< is actually a helper function for memAddTask, still exposed as public for code reuse >
pub fn populateTaskByTermLookup(judgementTasksByTerm:Arc<RwLock< HashMap<Term, Vec<Arc<Mutex<Task>>>> >>, term:&Term, task:&Arc<Mutex<Task>>) {
    let mut judgementTasksByTermGuard = judgementTasksByTerm.write();
    
    for iSubTerm in &retSubterms(&term) {
        if judgementTasksByTermGuard.contains_key(iSubTerm) {
            let mut v = judgementTasksByTermGuard.get(iSubTerm).unwrap().clone();
            v.push(Arc::clone(&task));
            judgementTasksByTermGuard.insert(iSubTerm.clone(), v);
        }
        else {
            judgementTasksByTermGuard.insert(iSubTerm.clone(), vec![Arc::clone(&task)]);
        }
    }
}

/// tries to revise the belief if possible
///
/// returns if it has done revision
pub fn memReviseBelief(mem:Arc<RwLock<NarMem::Mem>>, sentence:&SentenceDummy) -> bool {
    // MECHANISM< belief revision
    // revises beliefs if the term matches and if the stamps don't overlap
    // >
    
    // try to revise
    let mut wasRevised = false;
    match sentence.punct {
        EnumPunctation::JUGEMENT => {
            
            for iTerm in retSubterms(&*sentence.term) { // enumerate all terms, we need to do this to add the sentence to all relevant names
                match mem.write().concepts.get_mut(&iTerm.clone()) {
                    Some(arcConcept) => {
                        match Arc::get_mut(arcConcept) {
                            Some(concept) => {
                                let mut delBeliefIdx:Option<usize> = None;

                                let mut additionalBelief:Option<SentenceDummy> = None; // stores the additional belief
                                
                                for iBeliefIdx in 0..concept.beliefs.len() {
                                    let iBelief = &concept.beliefs[iBeliefIdx].read();
                                    if checkEqTerm(&iBelief.term, &sentence.term) && !NarStamp::checkOverlap(&iBelief.stamp, &sentence.stamp) {
                                        let stamp = NarStamp::merge(&iBelief.stamp, &sentence.stamp);
                                        let tvA:Tv = retTv(&iBelief).unwrap();
                                        let tvB:Tv = retTv(&sentence).unwrap();
                                        let evi:Evidence = Evidence::TV(rev(&tvA,&tvB));
                                        
                                        delBeliefIdx = Some(iBeliefIdx);
                                        additionalBelief = Some(SentenceDummy {
                                            term:iBelief.term.clone(),
                                            t:iBelief.t,
                                            punct:iBelief.punct,
                                            stamp:stamp,
                                            expDt:iBelief.expDt, // exponential time delta, used for =/>
                                            evi:Some(evi),
                                        }); // add revised belief!

                                        wasRevised = true;
                                        break; // breaking here is fine, because belief should be just once in table!
                                    }
                                }

                                if delBeliefIdx.is_some() {
                                    concept.beliefs.remove(delBeliefIdx.unwrap());
                                }

                                if additionalBelief.is_some() {
                                    concept.beliefs.push(Arc::new(RwLock::new(additionalBelief.unwrap())));
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
    };
    wasRevised
}

/// /param calcCredit compute the credit?
pub fn memAddTask(shared:Arc<RwLock<DeclarativeShared>>, sentence:&SentenceDummy, calcCredit:bool, cfg__nConceptBeliefs:usize) {
    // try to revise
    let wasRevised = memReviseBelief(Arc::clone(&shared.read().mem), sentence);
    if wasRevised {
        return;
    }
    // we are here if it can't revise
    
    NarMem::storeInConcepts(&mut shared.read().mem.write(), sentence, cfg__nConceptBeliefs); // store sentence in memory, adressed by concepts

    match sentence.punct {
        EnumPunctation::JUGEMENT => {
            let task = {
                let sharedGuard = shared.read();
                if true { // check if we should check if it already exist in the tasks
                    for ijt in &sharedGuard.judgementTasks { // ijt:iteration-judgement-task
                        let ijt2 = ijt.lock().unwrap();
                        if checkEqTerm(&sentence.term, &ijt2.sentence.term) && checkSame(&sentence.stamp, &ijt2.sentence.stamp) {
                            return; // don't add if it exists already! because we would skew the fairness if we would add it
                        }
                    }
                }

                let taskId:i64 = sharedGuard.taskIdCounter.fetch_add(1, Ordering::SeqCst); // TODO< is this ordering ok? >
                let mut task = Task {
                    sentence:sentence.clone(),
                    credit:1.0,
                    qaCredit:0.0, // no question was posed!
                    id:taskId,
                    derivTime:sharedGuard.cycleCounter
                };
                if calcCredit {
                    divCreditByComplexity(&mut task); // punish more complicated terms
                }

                task
            };
            

            let taskArc = Arc::new(Mutex::new(task));
            {
                shared.write().judgementTasks.push(Arc::clone(&taskArc));    
            }


            
            // populate hashmap lookup
            let sharedGuard = shared.read();
            populateTaskByTermLookup(Arc::clone(&sharedGuard.judgementTasksByTerm), &sentence.term, &taskArc);
        },
        EnumPunctation::QUESTION => {
            println!("TODO - check if we should check if it already exist in the tasks");
            
            let sharedGuard = shared.read();
            sharedGuard.questionTasks.write().push(Box::new(Task2 {
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

/// helper for attention
pub fn divCreditByComplexity(task:&mut Task) {
    task.credit /= calcComplexity(&task.sentence.term) as f64;
    task.qaCredit /= calcComplexity(&task.sentence.term) as f64;
}

/// tries to find a better answer for a question task
/// # Arguments
/// * `qTask` - the question task to find a answer to
/// * `concl` - candidate answer to get evaluated
/// * `globalQaHandlers` - 
pub fn qaTryAnswer(qTask: &mut Task2, concl: &SentenceDummy, globalQaHandlers: &Vec<Arc<RwLock< dyn QHandler>>>) {
    if concl.punct != EnumPunctation::JUGEMENT { // only jugements can answer questions!
        return;
    }

    if calcExp(&retTv(concl).unwrap()) > qTask.bestAnswerExp { // is the answer potentially better?
        let unifyRes: Option<Vec<Asgnment>> = unify(&qTask.sentence.term, &concl.term); // try unify question with answer
        if unifyRes.is_some() { // was answer found?
            let _unifiedRes: Term = unifySubst(&qTask.sentence.term, &unifyRes.unwrap());
            
            if qTask.handler.is_some() {
                // call Q&A handler for task
                let handler1 = qTask.handler.as_ref().unwrap();
                let mut handlerGuard = handler1.write();
                handlerGuard.answer(&qTask.sentence.term, &concl); // call callback because we found a answer
            }

            // call global Q&A handlers
            for iHandler in globalQaHandlers {
                let mut handlerGuard = iHandler.write();
                handlerGuard.answer(&qTask.sentence.term, &concl);
            }

            qTask.bestAnswerExp = calcExp(&retTv(&concl).unwrap()); // update exp of best found answer

            // print question and answer
            let msg = "TRACE answer: ".to_owned() + &convSentenceTermPunctToStr(&qTask.sentence, true) + " " + &convSentenceTermPunctToStr(&concl, true);
            println!("{}", msg);
        }
    }
}

/// performs one reasoning cycle
/// # Arguments
/// * `mem` - memory
pub fn reasonCycle(mem:Arc<RwLock<Mem2>>) {
    let cfgEnInstrumentation:bool = true; // enable instrumentation

    mem.read().shared.write().cycleCounter+=1;

    
    {
        let memGuard = mem.read();
        let mut sharedGuard = memGuard.shared.read(); // get read guard because we need only read here
        {
            
            // transfer credits from questionTasks to Judgement tasks
            for iTask in &*sharedGuard.questionTasks.read() {
                {
                    for iSubTerm in &retSubterms(&iTask.sentence.term) { // iterate over all terms
                        let optTasksBySubtermGuard = sharedGuard.judgementTasksByTerm.read();
                        let optTasksBySubterm = optTasksBySubtermGuard.get(&iSubTerm);
                        match optTasksBySubterm {
                            Some(tasksBySubterms) => {
                                for iIdx in 0..tasksBySubterms.len() {
                                    tasksBySubterms[iIdx].lock().unwrap().qaCredit += (*iTask).prio;
                                }
                            },
                            None => {},
                        }
                    } 
                }
            }
        }    
    }

    {
        let memGuard = mem.read();
        let mut sharedGuard = memGuard.shared.read(); // get read guard because we need only read here
        //let memGuard = mem.read();
        //let sharedGuard = memGuard.shared.read(); // get read guard because we need only read here
        
        // give base credit
        // JUSTIFICATION< else the tasks die down for forward inference >
        for iIdx in 0..sharedGuard.judgementTasks.len() {
            sharedGuard.judgementTasks[iIdx].lock().unwrap().credit += 0.5;
        }
        
        // let them pay for their complexity
        for iIdx in 0..sharedGuard.judgementTasks.len() {
            divCreditByComplexity(&mut sharedGuard.judgementTasks[iIdx].lock().unwrap());
        }
    }

    // sample question to answer
    {
        let memGuard = mem.read();
        let mut sharedGuard = memGuard.shared.read(); // get read guard because we need only read here

        let len = sharedGuard.questionTasks.read().len();
        if len > 0 {
            let selVal:f64 = mem.read().rng.write().gen_range(0.0,1.0);

            //let memGuard = mem.read();
            //let sharedGuard = memGuard.shared.read();
            let mut selTask = {
                let qIdx = task2SelByCreditRandom(selVal, &*sharedGuard.questionTasks.read());
                &mut *(sharedGuard.questionTasks.write())[qIdx]
            };
            

            // * enumerate subterms
            for iSubTerm in &retUniqueSubterms(&(*selTask).sentence.term.clone()) {

                // * retrieve concept by subterm
                match sharedGuard.mem.read().concepts.get(&iSubTerm) {
                    Some(concept) => {
                        // try to answer question with all beliefs which may be relevant
                        for iBelief in &concept.beliefs {
                            qaTryAnswer(&mut selTask, &iBelief.read(), &memGuard.globalQaHandlers.read());
                        }
                    },
                    None => {} // concept doesn't exist, ignore
                }

            }
        }
    }
    
    let mut msg: Option<DeriverWorkMessage> = None; // message which we have to send to worker for derivation
    {
        let memGuard = mem.read();
        let mut sharedGuard = memGuard.shared.read(); // get read guard because we need only read here
        if sharedGuard.judgementTasks.len() > 0 { // one working cycle - select for processing
            let selVal:f64 = mem.read().rng.write().gen_range(0.0,1.0);
            let selIdx = taskSelByCreditRandom(selVal, &sharedGuard.judgementTasks, sharedGuard.cycleCounter);
    
            //let memGuard = mem.read();
            //let sharedGuard = memGuard.shared.read();
            let selPrimaryTask = &sharedGuard.judgementTasks[selIdx];
            let selPrimaryTaskTerm:Arc<Term>;
            {
                selPrimaryTaskTerm = selPrimaryTask.lock().unwrap().sentence.term.clone();
            }

            {
                // attention mechanism which select the secondary task from the table 
                
                let mut secondaryElligable:Vec<Arc<Mutex<Task>>> = vec![]; // tasks which are eligable to get selected as the secondary
                
                let selPrimaryTaskTerm:Arc<Term>;
                {
                    selPrimaryTaskTerm = selPrimaryTask.lock().unwrap().sentence.term.clone();
                }
    
    
                for iSubTerm in &retUniqueSubterms(&selPrimaryTaskTerm) {
                    if sharedGuard.judgementTasksByTerm.read().contains_key(iSubTerm) {
                        let itJudgementTasksByTerm:Vec<Arc<Mutex<Task>>> = sharedGuard.judgementTasksByTerm.read().get(iSubTerm).unwrap().to_vec();
                        for it in &itJudgementTasksByTerm {// append to elligable, because it contains the term
                            let itId;
                            {
                                itId = it.lock().unwrap().id;
                            }
    
                            // code to figure out if task already exists in secondaryElligable
                            let mut existsById = false;
                            {
                                for iSec in &secondaryElligable {
                                    if iSec.lock().unwrap().id == itId {
                                        existsById = true;
                                        break; // OPT
                                    }
                                }
                            }
                            
                            if !existsById {
                                secondaryElligable.push(Arc::clone(&it));
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
    
                        println!("TRACE  primary term = {}", convTermToStr(&selPrimaryTask.lock().unwrap().sentence.term));
        
                        for iSecondaryElligable in &secondaryElligable {
                            println!("TRACE   consider secondary term = {}", convTermToStr(&iSecondaryElligable.lock().unwrap().sentence.term));
                            
                            match *selPrimaryTask.lock().unwrap().sentence.term { // is primary <=> or ==>
                                Term::Stmt(cop,_,_) if cop == Copula::IMPL || cop == Copula::EQUIV => {
                                    secondaryElligableFiltered.push(Arc::clone(iSecondaryElligable)); // consider
                                    println!("TRACE      ...   eligable, reason: primary is ==> or <=> !");
                                    continue;
                                },
                                _ => {}
                            }
        
                            // else special handling
                            match (*iSecondaryElligable.lock().unwrap().sentence.term).clone() {
                                Term::Stmt(cop,secondarySubj,_) if cop == Copula::IMPL || cop == Copula::EQUIV => {
                                    match *secondarySubj {
                                        Term::Conj(conjterms) => {
                                            // we need to check if conjterm unifies with primary
                                            let mut anyUnify = false;
                                            for iConjTerm in &conjterms {
                                                if unify(&selPrimaryTask.lock().unwrap().sentence.term.clone(), &iConjTerm).is_some() {
                                                    anyUnify = true;
                                                    break;
                                                }
                                            }
        
                                            if anyUnify {
                                                secondaryElligableFiltered.push(Arc::clone(iSecondaryElligable)); // consider
                                                println!("TRACE      ...   eligable, reason: secondary is &&==> or &&<=> and subterm of && unifies!");
                                                continue;
                                            }
                                        },
                                        _ => {
                                            // we need to check if secondarySubj unfies with primary
                                            if unify(&selPrimaryTask.lock().unwrap().sentence.term.clone(), &secondarySubj).is_some() {
                                                secondaryElligableFiltered.push(Arc::clone(iSecondaryElligable)); // consider
                                                println!("TRACE      ...   eligable, reason: secondary is &&==> or &&<=> and subterm of && unifies!");
                                                continue;
                                            }
                                        }
                                    }
                                },
                                _ => {
                                    secondaryElligableFiltered.push(Arc::clone(iSecondaryElligable)); // consider
                                    println!("TRACE      ...   eligable, reason: non-NAL-5&6 derivation!");
                                }
                            }
                        }
                        secondaryElligable = secondaryElligableFiltered;
                    }
                }
    
                let dbgSecondaryElligable = false; // do we want to debug elligable secondary tasks?
                if dbgSecondaryElligable {
                    println!("TRACE secondary eligable:");
                    for iSecondaryElligable in &secondaryElligable {
                        println!("TRACE    {}", convSentenceTermPunctToStr(&iSecondaryElligable.lock().unwrap().sentence, true));
                    }
                }
    
                if secondaryElligable.len() > 0 { // must contain any premise to get selected    
                    // build message of work
                    msg = Some(DeriverWorkMessage {
                        primary: Arc::clone(&selPrimaryTask),
                        secondary: secondaryElligable.iter().map(|iv| Arc::clone(iv)).collect(), // clone
                        cycleCounter: sharedGuard.cycleCounter,
                    });
                }
            }
        }
    
    }
    
    {
        if msg.is_some() { // do we have a message to send to worker?
            // submit message to worker
            let unwrappedMsg = msg.unwrap();
            let memGuard = mem.read();
            memGuard.deriverWorkersTx[0].send(unwrappedMsg).unwrap();
        }
    }

    let intervalCheckTasks = 111; // cycle counter to check for AIKR of tasks - should be prime
    let maxJudgementTasks = 30; // maximal number of judgement tasks

    // keep working tasks of judgements under AIKR
    {
        let memGuard = mem.read();
        let mut sharedGuard = memGuard.shared.write(); // get read guard because we need only read here
        if sharedGuard.cycleCounter % intervalCheckTasks == 0 //&& mem.judgementTasks.len() > maxJudgementTasks //// commented for testing
        {
            println!("[d] ENTER: keep working tasks under AIKR");

            let memCycleCounter:i64 = sharedGuard.cycleCounter;

            sharedGuard.judgementTasks.sort_by(|a, b| 
                taskCalcCredit(&b.lock().unwrap(), memCycleCounter).partial_cmp(
                    &taskCalcCredit(&a.lock().unwrap(), memCycleCounter)
                ).unwrap());
            sharedGuard.judgementTasks = sharedGuard.judgementTasks[0..maxJudgementTasks.min(sharedGuard.judgementTasks.len())].to_vec(); // limit to keep under AIKR
            
            println!("[d] EXIT: keep working tasks under AIKR");
        }
    }

    // keep judgement tasks by term under AIKR
    {
        let memGuard = mem.read();
        let mut sharedGuard = memGuard.shared.write(); // get read guard because we need only read here
        if sharedGuard.cycleCounter % intervalCheckTasks == 0 {
            sharedGuard.judgementTasksByTerm = Arc::new(RwLock::new(HashMap::new())); // flush, because we will repopulate

            // repopulate judgementTasksByTerm
            // IMPL< we had to split it because mem was accessed twice! >
            let mut termAndTask = vec![];
            for iJudgementTask in &sharedGuard.judgementTasks {
                let termRc:&Arc<Term> = &iJudgementTask.lock().unwrap().sentence.term;
                let term:Term = (**termRc).clone();

                termAndTask.push((term, Arc::clone(iJudgementTask)));
            }

            for (term, task) in &termAndTask { // iterate over prepared tuples
                // populate hashmap lookup
                populateTaskByTermLookup(Arc::clone(&sharedGuard.judgementTasksByTerm), &term, &task);
            }
        }
    }


    let intervalCheckConcepts = 173;
    let nConcepts = 10000; // number of concepts

    { // limit number of concepts
        let memGuard = mem.read();
        let sharedGuard = memGuard.shared.read(); // get read guard because we need only read here
        if sharedGuard.cycleCounter % intervalCheckConcepts == 0 {
            NarMem::limitMemory(&mut sharedGuard.mem.write(), nConcepts);
        }
    }
}


pub fn debugCreditsOfTasks(mem: &Mem2) -> Vec<String> {
    let mut res = Vec::new();
    
    // debug credit of tasks
    {
        for iTask in &mem.shared.read().judgementTasks {
            let taskSentenceAsStr = convSentenceTermPunctToStr(&iTask.lock().unwrap().sentence, true);
            
            let mut taskAsStr = taskSentenceAsStr.clone();

            let printStamp = true;
            if printStamp {
                taskAsStr = format!("{} {}", taskAsStr, NarStamp::convToStr(&iTask.lock().unwrap().sentence.stamp));
            }

            res.push(format!("task  {}  credit={}", taskAsStr, taskCalcCredit(&iTask.lock().unwrap(), mem.shared.read().cycleCounter)));
        }
    }

    res
}












/// called when answer is found
pub trait QHandler: Sync + Send {
    fn answer(&mut self, question:&Term, answer:&SentenceDummy);
}
