
// TODO< select highest ranked task, remove it from array, select other task by priority distribution, do inference, put results into memory >
//     TODO< put processed task into randomly sampled bag-table! >



// TODO< add question variable >

// sentence
// TODO   < add tv >
// TODO   < add stamp and stamp overlap check >


use std::rc::Rc;
use std::sync::Arc;

use Term::Term;
use Term::Copula;
use Term::retSubterms;
use Term::calcComplexity;

use NarSentence::EnumPunctation;
use NarSentence::SentenceDummy;

use NarMem;


// a --> b  b --> a
pub fn inf2(a: &Term) -> Option<Term> {
    match a {
        Term::Cop(Copula::INH, asubj, apred) => {
            return Some(Term::Cop(Copula::INH, Box::clone(apred), Box::clone(asubj)));
        },
        _ => {},
    }
    None
}


// a --> x.  x --> b.  |- a --> b.
pub fn inf0(a: &Term, punctA:EnumPunctation, b: &Term, punctB:EnumPunctation) -> Option<Term> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Cop(Copula::INH, asubj, apred) => {
            match b {
                Term::Cop(Copula::INH, bsubj, bpred) => {
                    if checkEqTerm(&apred, &bsubj) {
                        return Some(Term::Cop(Copula::INH, Box::clone(asubj), Box::clone(bpred))); // a.subj --> b.pred
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
pub fn inf3(a: &Term, punctA:EnumPunctation, b: &Term, punctB:EnumPunctation) -> Option<Term> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Cop(Copula::INH, asubj, apred2) => {
            match &**apred2 {
                Term::SetInt(apred) => {
                    
                    match b {
                        Term::Cop(Copula::INH, bsubj, bpred2) => {
                            match &**bpred2 {
                                Term::SetInt(bpred) => {
                                    
                                    if checkEqTerm(&asubj, &bsubj) {
                                        // build result set
                                        // TODO< compute union of set >
                                        let mut union_:Vec<Box<Term>> = vec![];
                                        union_.extend(apred.iter().cloned());
                                        union_.extend(bpred.iter().cloned());
                                        
                                        let resTerm = Term::SetInt(union_);
                                        
                                        return Some(Term::Cop(Copula::IMPL, Box::clone(asubj), Box::new(resTerm)));
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
pub fn inf4(a: &Term, punctA:EnumPunctation, b: &Term, punctB:EnumPunctation) -> Option<Term> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }

    match a {
        Term::Cop(Copula::INH, asubj2, apred) => {
            match &**asubj2 {
                Term::SetExt(asubj) => {
                    
                    match b {
                        Term::Cop(Copula::INH, bsubj2, bpred) => {
                            match &**bsubj2 {
                                Term::SetExt(bsubj) => {
                                    
                                    if checkEqTerm(&apred, &bpred) {
                                        // build result set
                                        // TODO< compute union of set >
                                        let mut union_:Vec<Box<Term>> = vec![];
                                        union_.extend(asubj.iter().cloned());
                                        union_.extend(bsubj.iter().cloned());
                                        
                                        let resTerm = Term::SetInt(union_);
                                        
                                        return Some(Term::Cop(Copula::IMPL, Box::new(resTerm), Box::clone(apred)));
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
pub fn inf1(a: &Term, punctA:EnumPunctation, b: &Term, punctB:EnumPunctation) -> Option<Term> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }

    match a {
        Term::Cop(Copula::IMPL, asubj, apred) => {
            match b {
                Term::Cop(Copula::IMPL, bsubj, bpred) => {
                    if checkEqTerm(&apred, &bsubj) {
                        return Some(Term::Cop(Copula::IMPL, Box::clone(asubj), Box::clone(bpred)));
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
        Term::DepVar(namea) => {
            match b2 {
                Term::DepVar(nameb) => false, // can't unify var with var
                Term::IndepVar(nameb) => false, // can't unify var with var
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
        Term::IndepVar(namea) => {
            match b2 {
                Term::DepVar(nameb) => false, // can't unify var with var
                Term::IndepVar(nameb) => false, // can't unify var with var
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
        
        
        Term::Cop(copulaa, subja, preda) => {
            match b2 {
                Term::Cop(copulab, subjb, predb) if copulaa == copulab => {
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
        Term::DepVar(name) => {
            // search for variable
            for iasgn in subst {
                if checkEqTerm(&t, &iasgn.var) {
                    return iasgn.val.clone();
                }
            }
            (*t).clone()
        },
        Term::IndepVar(name) => {
            // search for variable
            for iasgn in subst {
                if checkEqTerm(&t, &iasgn.var) {
                    return iasgn.val.clone();
                }
            }
            (*t).clone()
        },
        
        Term::Cop(copula, subj, pred) => {Term::Cop(*copula, Box::new(unifySubst(subj, subst)), Box::new(unifySubst(pred, subst)))},
        Term::Name(name) => (*t).clone(),
        
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
    }
}

// (a && b) ==> x.
// unify a.
// |-
// b ==> x.
pub fn inf5(a: &Term, punctA:EnumPunctation, b: &Term, punctB:EnumPunctation, idx:usize) -> Option<Term> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Cop(Copula::IMPL, aconj, apred) => {
            match &**aconj {
                Term::Conj(arr) => {
                    if arr.len() == 2 {
                        let unifyRes = unify(&arr[idx], &b);
                        if unifyRes.is_some() { // vars must unify
                            let subst = unifySubst(&arr[1-idx], &unifyRes.unwrap()); // substitute vars
                            return Some(Term::Cop(Copula::IMPL, Box::new(subst), Box::clone(apred)));
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
pub fn inf6(a: &Term, punctA:EnumPunctation, b: &Term, punctB:EnumPunctation) -> Option<Term> {
    if punctA != EnumPunctation::JUGEMENT || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Cop(Copula::IMPL, asubj, apred) => {
            let unifyRes = unify(asubj, &b);
            if unifyRes.is_some() { // vars must unify
                let subst = unifySubst(&apred, &unifyRes.unwrap()); // substitute vars
                return Some(subst);
            };
            None
        },
        _ => None,
    }
}


// a ==> x?
// unify x.
// |-
// a ==> x.
pub fn inf7(a: &Term, punctA:EnumPunctation, b: &Term, punctB:EnumPunctation) -> Option<Term> {
    if punctA != EnumPunctation::QUESTION || punctB != EnumPunctation::JUGEMENT {
        return None;
    }
    
    match a {
        Term::Cop(Copula::IMPL, _, apred) => {
            let unifyRes = unify(apred, &b);
            if unifyRes.is_some() { // vars must unify
                let subst = unifySubst(&a, &unifyRes.unwrap()); // substitute vars
                return Some(subst);
            };
            None
        },
        _ => None,
    }
}



// do binary inference
pub fn infBinaryInner(a: &Term, aPunct:EnumPunctation, b: &Term, bPunct:EnumPunctation) -> Vec<Term> {
    let mut res = vec![];
    
    match inf0(&a, aPunct, &b, bPunct) {
        Some(x) => { res.push(x); } _ => {}
    }
    match inf1(&a, aPunct, &b, bPunct) {
        Some(x) => { res.push(x); } _ => {}
    }
    match inf3(&a, aPunct, &b, bPunct) {
        Some(x) => { res.push(x); } _ => {}
    }
    match inf4(&a, aPunct, &b, bPunct) {
        Some(x) => { res.push(x); } _ => {}
    }
    match inf5(&a, aPunct, &b, bPunct, 0) {
        Some(x) => { res.push(x); } _ => {}
    }
    match inf5(&a, aPunct, &b, bPunct, 1) {
        Some(x) => { res.push(x); } _ => {}
    }
    match inf6(&a, aPunct, &b, bPunct) {
        Some(x) => { res.push(x); } _ => {}
    }
    match inf7(&a, aPunct, &b, bPunct) {
        Some(x) => { res.push(x); } _ => {}
    }
    
    res
}

// do binary inference
pub fn infBinary(a: &Term, aPunct:EnumPunctation, b: &Term, bPunct:EnumPunctation) -> Vec<Term> {
    let mut res = vec![];
    res.extend(infBinaryInner(&a, aPunct, &b, bPunct).iter().cloned());
    res.extend(infBinaryInner(&b, bPunct, &a, aPunct).iter().cloned());
    res
}




pub fn checkEqTerm(a:&Term, b:&Term) -> bool {
    match a {
        Term::Cop(copulaa, subja, preda) => {
            match b {
                Term::Cop(copulab, subjb, predb) => copulaa == copulab && checkEqTerm(&subja, &subjb) && checkEqTerm(&preda, &predb),
                _ => false
            }
        }
        Term::Name(namea) => {
            match b {
                Term::Name(nameb) => namea == nameb,
                _ => false
            }
        },
        Term::Seq(seqa) => {
            match b {
                Term::Seq(seqb) => {
                    if seqa.len() == seqb.len() {
                        for idx in 0..seqa.len() {
                            if !checkEqTerm(&seqa[idx], &seqb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::SetInt(seta) => {
            match b {
                Term::SetInt(setb) => {
                    if seta.len() == setb.len() {
                        for idx in 0..seta.len() {
                            if !checkEqTerm(&seta[idx], &setb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::SetExt(seta) => {
            match b {
                Term::SetExt(setb) => {
                    if seta.len() == setb.len() {
                        for idx in 0..seta.len() {
                            if !checkEqTerm(&seta[idx], &setb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::DepVar(namea) => {
            match b {
                Term::DepVar(nameb) => namea == nameb,
                _ => false
            }
        },
        Term::IndepVar(namea) => {
            match b {
                Term::IndepVar(nameb) => namea == nameb,
                _ => false
            }
        },
        Term::Conj(elementsa) => {
            match b {
                Term::Conj(elementsb) => {
                    if elementsa.len() == elementsb.len() {
                        for idx in 0..elementsa.len() {
                            if !checkEqTerm(&elementsa[idx], &elementsb[idx]) {return false};
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

pub fn convTermToStr(t:&Term) -> String {
    match t {
        Term::Cop(Copula, subj, pred) => {
            let subjStr = convTermToStr(subj);
            let predStr = convTermToStr(pred);
            let copStr = match Copula {Copula::SIM=>{"<->"},Copula::INH=>{"-->"},Copula::PREDIMPL=>"=/>",Copula::IMPL=>{"==>"}};
            format!("<{} {} {}>", subjStr, copStr, predStr)
        }
        Term::Name(name) => name.to_string(),
        Term::Seq(seq) => {
            let mut inner = convTermToStr(&seq[0]);
            for i in 1..seq.len() {
                inner = format!("{} &/ {}", inner, convTermToStr(&seq[i]));
            }
            format!("( {} )", inner)
        },
        Term::SetInt(set) => {
            let mut inner = convTermToStr(&set[0]);
            for i in 1..set.len() {
                inner = format!("{} {}", inner, convTermToStr(&set[i]));
            }
            format!("[{}]", inner)
        },
        Term::SetExt(set) => {
            let mut inner = convTermToStr(&set[0]);
            for i in 1..set.len() {
                inner = format!("{} {}", inner, convTermToStr(&set[i]));
            }
            format!("{{{}}}", inner)
        },
        Term::DepVar(name) => {
            format!("#{}", name)
        },
        Term::IndepVar(name) => {
            format!("${}", name)
        },
        Term::Conj(elements) => {
            let mut inner = convTermToStr(&elements[0]);
            for i in 1..elements.len() {
                inner = format!("{} && {}", inner, convTermToStr(&elements[i]));
            }
            format!("( {} )", inner)
        },
    }
}





// test
//    <( <a --> b> && <c --> d> ) ==> x>
//    <a --> b>
//    concl:
//    <<c --> d> ==> x>
pub fn testManual0() {
    let a0 = Term::Name("a".to_string());
    let b0 = Term::Name("b".to_string());
    let inh0 = Term::Cop(Copula::INH, Box::new(a0), Box::new(b0));
    
    let c0 = Term::Name("c".to_string());
    let d0 = Term::Name("d".to_string());
    let inh1 = Term::Cop(Copula::INH, Box::new(c0), Box::new(d0));
    
    let conj0 = Term::Conj(vec![Box::new(inh0), Box::new(inh1)]);
    
    let x0 = Term::Name("x".to_string());
    let impl0 = Term::Cop(Copula::IMPL, Box::new(conj0), Box::new(x0));
    
    
    let a1 = Term::Name("a".to_string());
    let b1 = Term::Name("b".to_string());
    let inh1 = Term::Cop(Copula::INH, Box::new(a1), Box::new(b1));
    
    println!("{}", &convTermToStr(&impl0));
    println!("{}", &convTermToStr(&inh1));
    println!("concl:");
    
    let infConcl = infBinary(&impl0, EnumPunctation::JUGEMENT, &inh1, EnumPunctation::JUGEMENT);
    for iInfConcl in infConcl {
        println!("{}", &convTermToStr(&iInfConcl));
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
        let inh0 = Term::Cop(Copula::INH, Box::new(a0), Box::new(b0));
        
        let c0 = Term::IndepVar("c".to_string());
        let d0 = Term::Name("d".to_string());
        let inh1 = Term::Cop(Copula::INH, Box::new(c0), Box::new(d0));
        
        let impl0 = Term::Cop(Copula::IMPL, Box::new(inh0), Box::new(inh1));
        
        
        let c1 = Term::Name("c".to_string());
        let d1 = Term::Name("d".to_string());
        let inh2 = Term::Cop(Copula::INH, Box::new(c1), Box::new(d1));
        
        println!("{}", &convTermToStr(&impl0));
        println!("{}", &convTermToStr(&inh2));
        println!("concl:");
        
        let mut success=false;
        
        let infConcl = infBinary(&impl0, EnumPunctation::QUESTION, &inh2, EnumPunctation::JUGEMENT);
        for iInfConcl in infConcl {
            let concl = convTermToStr(&iInfConcl);
            println!("{}", &concl);
            if concl == "<<a --> b> ==> <c --> d>>" {
                success=true;
            }
        }
        
        assert_eq!(success, true);
    }
}








pub struct Task {
    pub sentence:SentenceDummy,
    pub credit:f64,
}

pub struct Task2 {
    pub sentence:SentenceDummy,
    pub prio:f64, // priority
}

use std::collections::HashMap;
//use std::cell::Cell;
use std::cell::{RefCell};
//use std::cell::{Ref};
//use std::rc::Rc;

/*
pub struct Mem<'a,'b> {
    pub judgementTasks:Vec<Ref<'a, RefCell<Task>>>,
    pub judgementTasksByTerm:HashMap<Term, Vec<Ref<'a, RefCell<Task>>>>, // for fast lookup
    pub questionTasks:Vec<Box<&'b Task2>>,
}
*/
pub struct Mem2 {
    pub judgementTasks:Vec<Rc<RefCell<Task>>>,
    pub judgementTasksByTerm:HashMap<Term, Vec<Rc<RefCell<Task>>>>, // for fast lookup
    pub questionTasks:Vec<Box<Task2>>,

    pub mem: Rc<RefCell<NarMem::Mem>>,
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



pub fn memAddTask(mem:&mut Mem2, sentence:&SentenceDummy) {
    NarMem::storeInConcepts(&mut mem.mem.borrow_mut(), sentence); // store sentence in memory, adressed by concepts
    
    match sentence.punct {
        EnumPunctation::JUGEMENT => {
            let x:RefCell<Task> = RefCell::new(Task {
                sentence:sentence.clone(),
                credit:0.0,
            });
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
            mem.questionTasks.push(Box::new(Task2 {
                sentence:sentence.clone(),
                prio:1.0,
            }));
        },
        EnumPunctation::GOAL => {
            println!("ERROR: goal is not implemented!");
        },
    }
    
}



// not working prototype of attention mechanism based on credits
pub fn expNarsWorkingCycle0() {
    // TODO< create and fill concepts! by sentence when storing sentence into memory >
    let mut mem:Mem2;
    {
        let mut mem0:NarMem::Mem = NarMem::Mem{
            concepts:HashMap::new(),
        };
    
        mem = Mem2{judgementTasks:vec![], judgementTasksByTerm:HashMap::new(), questionTasks:vec![], mem:Rc::new(RefCell::new(mem0))};
    }
    
    // add testing tasks
    {
        { // .
            let sentence = SentenceDummy {
                isOp:false, // is it a operation?
                term:Rc::new(Term::Cop(Copula::INH, Box::new(Term::Name("a".to_string())), Box::new(Term::Name("b".to_string())))),
                t:0, // time of occurence 
                punct:EnumPunctation::JUGEMENT,
            };
            memAddTask(&mut mem, &sentence);
        }

        { // .
            let sentence = SentenceDummy {
                isOp:false, // is it a operation?
                term:Rc::new(Term::Cop(Copula::INH, Box::new(Term::Name("b".to_string())), Box::new(Term::Name("c".to_string())))),
                t:0, // time of occurence 
                punct:EnumPunctation::JUGEMENT,
            };
            memAddTask(&mut mem, &sentence);
        }

        { // ?
            let sentence = SentenceDummy {
                isOp:false, // is it a operation?
                term:Rc::new(Term::Cop(Copula::INH, Box::new(Term::Name("a".to_string())), Box::new(Term::Name("c".to_string())))),
                t:0, // time of occurence 
                punct:EnumPunctation::QUESTION,
            };
            memAddTask(&mut mem, &sentence);
        }
    }




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
    
    // let them pay for their complexity
    {
        for iIdx in 0..mem.judgementTasks.len() {
            let complexity:f64 = calcComplexity(&*mem.judgementTasks[iIdx].borrow().sentence.term) as f64;
            
            let x:&RefCell<Task> = &(*mem.judgementTasks[iIdx]);
            x.borrow_mut().credit /= complexity;
        }
    }


    /////////storeInConcepts(&mut mem2, s:&SentenceDummy) 

    { // one working cycle - select for processing
        println!("TODO - select by credit distribution");
        let mut selIdx:usize = 0;

        let selTask = &mem.judgementTasks[selIdx];

        match mem.mem.borrow_mut().concepts.get_mut(&selTask.borrow().sentence.term.clone()) {
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




    // debug credit of tasks
    {
        for iTask in &mem.judgementTasks {
            let taskSentenceAsStr = convSentenceTermPunctToStr(&iTask.borrow().sentence);
            println!("task  {}  credit={}", taskSentenceAsStr, iTask.borrow().credit);
        }
    }
}

// convert only term and punctation to string
pub fn convSentenceTermPunctToStr(s:&SentenceDummy) -> String {
    let punct = match s.punct{
        EnumPunctation::QUESTION=>"?",
        EnumPunctation::JUGEMENT=>".",
        EnumPunctation::GOAL=>"!",
    };    
    convTermToStr(&s.term) + punct
}





