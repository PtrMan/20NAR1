
// TODO< select highest ranked task, remove it from array, select other task by priority distribution, do inference, put results into memory >
//     TODO< put processed task into randomly sampled bag-table! >



// TODO< add question variable >

use rand::Rng;
use rand::rngs::ThreadRng;

use std::rc::Rc;
use std::sync::Arc;

use Term::Term;
use Term::Copula;
use Term::retSubterms;
use Term::calcComplexity;
use Term::convTermToStr;
use Term::checkEqTerm;

use NarSentence::EnumPunctation;
use NarSentence::SentenceDummy;
use NarSentence::convSentenceTermPunctToStr;
use NarSentence::retTv;
use NarSentence::Evidence;

use NarMem;
use Tv::*;
use NarStamp::*;
use NarStamp;

// a --> b  b --> a
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


// a --> x.  x --> b.  |- a --> b.
pub fn inf0(a: &Term, punctA:EnumPunctation, aTv:&Tv, b: &Term, punctB:EnumPunctation, bTv:&Tv) -> Option<(Term,Tv)> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::INH, asubj, apred) => {
            match b {
                Term::Stmt(Copula::INH, bsubj, bpred) => {
                    if !checkEqTerm(&asubj, &bpred) && checkEqTerm(&apred, &bsubj) {
                        return Some(( Term::Stmt(Copula::INH, Box::clone(asubj), Box::clone(bpred)), ded(&aTv,&bTv) )); // a.subj --> b.pred
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
pub fn inf3(a: &Term, punctA:EnumPunctation, aTv:&Tv, b: &Term, punctB:EnumPunctation, bTv:&Tv) -> Option<(Term,Tv)> {
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
                                        return Some((Term::Stmt(Copula::INH, Box::clone(asubj), Box::new(resTerm)), aTv.clone()));
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
pub fn inf4(a: &Term, punctA:EnumPunctation, aTv:&Tv, b: &Term, punctB:EnumPunctation, bTv:&Tv) -> Option<(Term,Tv)> {
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
}

// a ==> x.  x ==> b.  |- a ==> b.
pub fn inf1(a: &Term, punctA:EnumPunctation, aTv:&Tv, b: &Term, punctB:EnumPunctation, bTv:&Tv) -> Option<(Term,Tv)> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }

    match a {
        Term::Stmt(Copula::IMPL, asubj, apred) => {
            match b {
                Term::Stmt(Copula::IMPL, bsubj, bpred) => {
                    if checkEqTerm(&apred, &bsubj) {
                        println!("TODO - compute TV correctly!");
                        return Some((Term::Stmt(Copula::IMPL, Box::clone(asubj), Box::clone(bpred)), Tv{f:1.0, c:0.99}));
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
    }
}

// (a && b) ==> x.
// unify a.
// |-
// b ==> x.
pub fn inf5(a: &Term, punctA:EnumPunctation, aTv:&Tv, b: &Term, punctB:EnumPunctation, bTv:&Tv, conjIdx:usize) -> Option<(Term,Tv)> {
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
                            
                            println!("TODO - compute TV correctly!");
                            return Some((conclTerm, aTv.clone()));
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



// a ==> x.
// unify a.
// |-
// x.
pub fn inf6(a: &Term, punctA:EnumPunctation, aTv:&Tv, b: &Term, punctB:EnumPunctation, bTv:&Tv) -> Option<(Term,Tv)> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::IMPL, asubj, apred) => {
            let unifyRes = unify(asubj, &b);
            if unifyRes.is_some() { // vars must unify
                let subst = unifySubst(&apred, &unifyRes.unwrap()); // substitute vars
                println!("TODO - compute TV correctly!");
                return Some((subst,aTv.clone()));
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
pub fn inf7(a: &Term, punctA:EnumPunctation, aTv:&Tv, b: &Term, punctB:EnumPunctation, bTv:&Tv) -> Option<(Term, Tv)> {
    if punctA != EnumPunctation::QUESTION || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Stmt(Copula::IMPL, _, apred) => {
            let unifyRes = unify(apred, &b);
            if unifyRes.is_some() { // vars must unify
                let subst = unifySubst(&a, &unifyRes.unwrap()); // substitute vars
                println!("TODO - compute TV correctly!");
                return Some((subst,aTv.clone()));
            };
            None
        },
        _ => None,
    }
}



// do binary inference
pub fn infBinaryInner(a: &Term, aPunct:EnumPunctation, aTv:&Tv, b: &Term, bPunct:EnumPunctation, bTv:&Tv, wereRulesApplied:&mut bool) -> Vec<(Term,Tv)> {
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
    match inf6(&a, aPunct, &aTv, &b, bPunct, &bTv) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    match inf7(&a, aPunct, &aTv, &b, bPunct, &bTv) {
        Some(x) => { res.push(x); *wereRulesApplied=true; } _ => {}
    }
    
    res
}

// do binary inference
pub fn infBinary(a: &Term, aPunct:EnumPunctation, aTv:&Tv, b: &Term, bPunct:EnumPunctation, bTv:&Tv, wereRulesApplied:&mut bool) -> Vec<(Term,Tv)> {
    let mut res = vec![];
    *wereRulesApplied = false; // because no rules were applied yet
    res.extend(infBinaryInner(&a, aPunct, &aTv, &b, bPunct, &bTv, wereRulesApplied).iter().cloned());
    res.extend(infBinaryInner(&b, bPunct, &aTv, &a, aPunct, &bTv, wereRulesApplied).iter().cloned());
    res
}










// test
//    <( <a --> b> && <c --> d> ) ==> x>
//    <a --> b>
//    concl:
//    <<c --> d> ==> x>
pub fn testManual0() {
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
    
    let mut wereRulesApplied = false;
    let infConcl = infBinary(&impl0, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.9}, &inh1, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.9}, &mut wereRulesApplied);
    for iInfConcl in infConcl {
        let (conclTerm, _conclTv) = iInfConcl;
        println!("{}", &convTermToStr(&conclTerm));
    }
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
    pub fn testManual1() {
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
        let infConcl = infBinary(&impl0, EnumPunctation::QUESTION, &Tv{f:1.0,c:0.9}, &inh2, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.9}, &mut wereRulesApplied);
        for iInfConcl in infConcl {
            let (conclTerm, _conclTv) = iInfConcl;
            let conclTermStr = convTermToStr(&conclTerm);
            println!("{}", &conclTermStr);
            if conclTermStr == "<<a --> b> ==> <c --> d>>" {
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
        let (term, tv) = iInfConcl;
        
        println!("TODO - infBinary must compute the punctation!");
        concl.push(SentenceDummy{
            term:Rc::new(term.clone()),
            evi:Evidence::TV(tv.clone()),
            stamp:merge(&pa.stamp, &pb.stamp),
            t:None, // time of occurence 
            punct:EnumPunctation::JUGEMENT, // BUG - we need to compute punctation in inference
            expDt:None
        });
    }

    if concl.len() > 0 && checkOverlap(&pa.stamp, &pb.stamp) { // check for overlap
      concl = vec![]; // flush conclusions because we don't have any conclusions when the premises overlapped
    }
    
    concl
}






pub struct Task {
    pub sentence:SentenceDummy,
    pub credit:f64,
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

    pub cycleCounter: i64, // counter for done reasoning cycles

    pub rng: ThreadRng,
}



// helper to select random task by credit
pub fn taskSelByCreditRandom(selVal:f64, arr: &Vec<Rc<RefCell<Task>>>)->usize {
    let sum:f64 = arr.iter().map(|iv| iv.borrow().credit).sum();
    let mut acc = 0.0;
    let mut idx = 0;
    for iv in arr {
        acc += iv.borrow().credit;
        if acc >= selVal*sum {
            return idx;
        }
        idx+=1;
    }
    
    arr.len()-1 // sel last
}

// helper to select task with highest prio
pub fn tasksSelHighestCreditIdx(arr: &Vec<Rc<RefCell<Task>>>) -> Option<usize> {
    if arr.len() == 0 {
        return None;
    }
    let mut idxRes = 0;
    let mut res = Rc::clone(&arr[0]);
    for idx in 1..arr.len() {
        let sel = &arr[idx];
        if sel.borrow().credit > res.borrow().credit {
            res = Rc::clone(&sel);
            idxRes = idx;
        }
    }
    Some(idxRes)
}



// /param calcCredit compute the credit?
pub fn memAddTask(mem:&mut Mem2, sentence:&SentenceDummy, calcCredit:bool) {
    NarMem::storeInConcepts(&mut mem.mem.borrow_mut(), sentence); // store sentence in memory, adressed by concepts
    

    match sentence.punct {
        EnumPunctation::JUGEMENT => {
            if true { // check if we should check if it already exist in the tasks
                for ijt in &mem.judgementTasks { // ijt:iteration-judgement-task
                    if checkSame(&sentence.stamp, &ijt.borrow().sentence.stamp) {
                        return; // don't add if it exists already! because we would skew the fairness if we would add it
                    }
                }
            }

            let mut task = Task {
                sentence:sentence.clone(),
                credit:1.0,
            };

            if calcCredit {
                divCreditByComplexity(&mut task); // punish more complicated terms
            }

            let x:RefCell<Task> = RefCell::new(task);
            let y = Rc::new(x);
            mem.judgementTasks.push(Rc::clone(&y));
            
            // populate hashmap lookup
            for iSubTerm in &retSubterms(&sentence.term) {
                if mem.judgementTasksByTerm.contains_key(iSubTerm) {
                    let mut v = mem.judgementTasksByTerm.get(iSubTerm).unwrap().clone();
                    v.push(Rc::clone(&y));
                    mem.judgementTasksByTerm.insert(iSubTerm.clone(), v);
                }
                else {
                    mem.judgementTasksByTerm.insert(iSubTerm.clone(), vec![Rc::clone(&y)]);
                }
            }
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
    


    if mem.judgementTasks.len() > 0 { // one working cycle - select for processing
        let selVal:f64 = mem.rng.gen_range(0.0,1.0);
        let selIdx = taskSelByCreditRandom(selVal, &mem.judgementTasks);

        let selPrimaryTask = &mem.judgementTasks[selIdx];
        let selPrimaryTaskTerm = selPrimaryTask.borrow().sentence.term.clone();

        {
            // attention mechanism which select the secondary task from the table 
            // TODO< selection by mass is unfair, because we select the same task multiple times.
            //       we should add each task only once here, this can be realized by giving each task
            //       a unique id and by storing the task in a hashmap to guarantue that each task is taken just once
            //     >
            println!("TODO - select secondary task canditate just once!");
            
            let mut secondaryElligable:Vec<Rc<RefCell<Task>>> = vec![]; // tasks which are eligable to get selected as the secondary
            
            for iSubTerm in &retSubterms(&selPrimaryTask.borrow().sentence.term.clone()) {
                if mem.judgementTasksByTerm.contains_key(iSubTerm) {
                    let itJudgementTasksByTerm:Vec<Rc<RefCell<Task>>> = mem.judgementTasksByTerm.get(iSubTerm).unwrap().to_vec();
                    for it in &itJudgementTasksByTerm {// append to elligable, because it contains the term
                        secondaryElligable.push(Rc::clone(&it));
                    }
                }
            }

            // sample from secondaryElligable by priority
            let selVal:f64 = mem.rng.gen_range(0.0,1.0);
            let secondarySelTaskIdx = taskSelByCreditRandom(selVal, &secondaryElligable);
            let secondarySelTask: &Rc<RefCell<Task>> = &secondaryElligable[secondarySelTaskIdx];

            // debug premsises
            {
                {
                    let taskSentenceAsStr = convSentenceTermPunctToStr(&selPrimaryTask.borrow().sentence);
                    println!("DBG  primary   task  {}  credit={}", taskSentenceAsStr, selPrimaryTask.borrow().credit);    
                }
                {
                    let taskSentenceAsStr = convSentenceTermPunctToStr(&secondarySelTask.borrow().sentence);
                    println!("DBG  secondary task  {}  credit={}", taskSentenceAsStr, secondarySelTask.borrow().credit);
                }
            }

            // do inference with premises
            let mut wereRulesApplied = false;
            let concl: Vec<SentenceDummy> = inference(&selPrimaryTask.borrow().sentence, &secondarySelTask.borrow().sentence, &mut wereRulesApplied);

            // put conclusions back into memory!
            {
                println!("TODO TODO TODO - put conclusions back into memory the right way");
                
                // Q&A - answer questions
                {
                    for iConcl in &concl {
                        if iConcl.punct == EnumPunctation::JUGEMENT { // only jugements can answer questions!
                            for iQTask in &mut mem.questionTasks {
                                if calcExp(&retTv(iConcl)) > iQTask.bestAnswerExp { // is the answer potentially better?
                                    let unifyRes: Option<Vec<Asgnment>> = unify(&iQTask.sentence.term, &iConcl.term); // try unify question with answer
                                    if unifyRes.is_some() { // was answer found?
                                        let unifiedRes: Term = unifySubst(&iQTask.sentence.term, &unifyRes.unwrap());
                                        
                                        if iQTask.handler.is_some() {
                                            let handler1 = iQTask.handler.as_ref().unwrap();
                                            let mut handler2 = handler1.borrow_mut();
                                            handler2.answer(&iQTask.sentence.term, &iConcl); // call callback because we found a answer
                                        }

                                        iQTask.bestAnswerExp = calcExp(&retTv(&iConcl)); // update exp of best found answer

                                        // print question and answer
                                        let msg = "answer: ".to_owned() + &convSentenceTermPunctToStr(&iQTask.sentence) + " " + &convSentenceTermPunctToStr(&iConcl);
                                        println!("{}", msg);
                                    }
                                }
                            }
                        }
                    }
                }
                
 
                for iConcl in &concl {
                    // TODO< check if task exists already, don't add if it exists >
                    memAddTask(mem, iConcl, true);
                }
            }
        }

        { // attention mechanism which selects the secondary task from concepts
            match mem.mem.borrow_mut().concepts.get_mut(&selPrimaryTaskTerm) {
                Some(arcConcept) => {
                    match Arc::get_mut(arcConcept) {
                        Some(concept) => {
                            println!("TODO< do stuff with beliefs inside concept!!! >!!!");
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

    // keep working tasks of judgements under AIKR
    {
        let maxJudgementTasks = 10; // maximal number of judgement tasks
        //if mem.judgementTasks.len() > maxJudgementTasks && cycleCounter % 111 == 0 //// commented for testing
        {
            mem.judgementTasks.sort_by(|a, b| b.borrow().credit.partial_cmp(&a.borrow().credit).unwrap());
            mem.judgementTasks = mem.judgementTasks[0..maxJudgementTasks.min(mem.judgementTasks.len())].to_vec(); // limit to keep under AIKR
        }
    }


}

pub fn createMem2()->Mem2 {
    let mem0:NarMem::Mem = NarMem::Mem{
        concepts:HashMap::new(),
    };
    
    Mem2{judgementTasks:vec![], judgementTasksByTerm:HashMap::new(), questionTasks:vec![], mem:Rc::new(RefCell::new(mem0)), rng:rand::thread_rng(), stampIdCounter:0, cycleCounter:0, }
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
                evi:Evidence::TV(Tv{f:1.0,c:0.9}),
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
                evi:Evidence::TV(Tv{f:1.0,c:0.9}),
                expDt:None
            };
            memAddTask(&mut mem, &sentence, true);
        }

        { // ?
            println!("TODO - questions don't have a tv!");
            let sentence = SentenceDummy {
                //isOp:false, // is it a operation?
                term:Rc::new(Term::Stmt(Copula::INH, Box::new(Term::Name("a".to_string())), Box::new(Term::Name("c".to_string())))),
                t:None, // time of occurence 
                punct:EnumPunctation::QUESTION,
                stamp:newStamp(&vec![2]),
                evi:Evidence::TV(Tv{f:1.0,c:0.0}),
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
            let taskSentenceAsStr = convSentenceTermPunctToStr(&iTask.borrow().sentence);
            
            let mut taskAsStr = taskSentenceAsStr.clone();

            let printStamp = true;
            if printStamp {
                taskAsStr = format!("{} {}", taskAsStr, NarStamp::convToStr(&iTask.borrow().sentence.stamp));
            }

            println!("task  {}  credit={}", taskAsStr, iTask.borrow().credit);
        }
    }
}












// called when answer is found
pub trait QHandler {
    fn answer(&mut self, question:&Term, answer:&SentenceDummy);
}
