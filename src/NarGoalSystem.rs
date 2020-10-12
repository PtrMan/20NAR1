// goal system

use std::rc::Rc;
use core::cell::RefCell;
use std::sync::Arc;
use crate::rand::Rng;

use crate::Term::*;

//use crate::Tv::calcExp;
//use crate::Tv::ded;
use crate::Tv;

use crate::NarStamp;

use crate::NarSentence;
use crate::NarSentence::EnumPunctation;
use crate::NarSentence::SentenceDummy;
use crate::NarSentence::retTv;
use crate::NarSentence::newEternalSentenceByTv;

pub struct GoalSystem {
    pub entries: Vec<Rc<RefCell<Entry>>>,
    pub nMaxEntries: usize, // max number of entries
}

// entry for goal system
pub struct Entry {
    pub sentence: Arc<SentenceDummy>,
    pub utility: f64,
}

pub fn addEntry(goalSystem: &mut GoalSystem, goal: Arc<SentenceDummy>) {
    // we check for same stamp - ignore it if the goal is exactly the same, because we don't need to store same goals
    for iv in &goalSystem.entries {
        if 
            // optimization< checking term first is faster! >
            //checkEqTerm(&iv.sentence.term, &goal.term) &&
            NarStamp::checkSame(&iv.borrow().sentence.stamp, &goal.stamp)
        {
            return;
        }
    }

    goalSystem.entries.push(Rc::new(RefCell::new(Entry{sentence:Arc::clone(&goal), utility:1.0})));
}

// called when it has to stay under AIKR
pub fn limitMemory(goalSystem: &mut GoalSystem) {
    // * recalc utility
    for iv in &goalSystem.entries {
        let mut iv2 = iv.borrow_mut();
        // TODO - consider age!
        iv2.utility = Tv::calcExp(&retTv(&iv2.sentence).unwrap());
    }

    // * sort by utility
    goalSystem.entries.sort_by(|a, b| b.borrow().utility.partial_cmp(&a.borrow().utility).unwrap());

    // * limit (AIKR)
    while goalSystem.entries.len() > goalSystem.nMaxEntries {
        goalSystem.entries.remove(goalSystem.nMaxEntries);
    }
}

pub fn sample(goalSystem: &GoalSystem, rng: &mut rand::rngs::ThreadRng) -> Option<Arc<SentenceDummy>> {
    if goalSystem.entries.len() == 0 {
        return None;
    }
    
    let sumPriorities:f64 = goalSystem.entries.iter().map(|iv| 1.0).sum();
    
    let selPriority:f64 = rng.gen_range(0.0, 1.0) * sumPriorities;

    // select
    let mut sum:f64 = 0.0;
    for iv in &goalSystem.entries {
        sum += 1.0;
        if sum >= selPriority {
            return Some(Arc::clone(&iv.borrow().sentence));
        }
    }

    // shouldn't happen
    return Some(Arc::clone(&goalSystem.entries[goalSystem.entries.len()-1].borrow().sentence));
}

// does inference of goal with a belief
// returns derivation
pub fn infer(goal: &SentenceDummy, belief: &SentenceDummy)-> Option<SentenceDummy> {
    // check if term is same and inference can be done
    match &*belief.term {
        Term::Stmt(Copula::PREDIMPL, subj, pred) => {
            if !checkEqTerm(&goal.term, &pred) {
                return None; // can't do inference because terms have to be equal
            }
        },
        _ => {
            // don't do anything here
            return None;
        }
    }
    
    if NarStamp::checkOverlap(&goal.stamp, &belief.stamp) {
        return None; // overlap -> can't derive anything
    }


    
    // we need to derive goals from matching implSeqs by goal deduction
    // a =/> b.
    // b!
    // |- ded
    // a!
    let tvCompound = retTv(&belief).unwrap();
    let tvComponent = retDesire(&goal);
    let tvConcl = Tv::ded(&tvCompound, &tvComponent);
    
    let stamp = NarStamp::merge(&goal.stamp, &belief.stamp);

    match &*belief.term {
        Term::Stmt(Copula::PREDIMPL, subj, _) => {
            return Some(newEternalSentenceByTv(&subj,EnumPunctation::GOAL,&tvConcl,stamp));
        },
        _ => {
            // don't do anything here
            return None;
        }
    }
}



// filters belief candidates which can be used for inference with the goal
pub fn retBeliefCandidates(goal: &SentenceDummy, evidence: &Vec<Rc<RefCell<SentenceDummy>>>) -> Vec<Rc<RefCell<SentenceDummy>>> {
    let mut res = Vec::new();
    
    for iBelief in &*evidence {
        match &*(**iBelief).borrow().term {
            Term::Stmt(Copula::PREDIMPL, subj, pred) => {
                if checkEqTerm(&goal.term, &pred) {
                    res.push(Rc::clone(iBelief));
                }
            },
            _ => {}
        }
    }

    res
}


// select highest ranked goal for state
pub fn selHighestExpGoalByState(goalSystem: &GoalSystem, state:&Term) -> (f64, Option<Rc<RefCell<Entry>>>) {
    let mut res:(f64, Option<Rc<RefCell<Entry>>>) = (0.0, None);

    for iv in &goalSystem.entries {
        match &(*(iv.borrow().sentence).term) {
            Term::Seq(seq) if seq.len() >= 1 && checkEqTerm(&seq[0], &state) => { // does first event of seq match to state?
                
                let exp = Tv::calcExp(&retTv(&iv.borrow().sentence).unwrap());
                let (resExp, _) = res;
                if exp > resExp {
                    res = (exp, Some(Rc::clone(&iv))); // TODO<hand over a reference to the Entry>
                }
            },
            _ => {}
        }

    }

    res
}

pub fn sampleAndInference(goalSystem: &mut GoalSystem, evidence: &Vec<Rc<RefCell<SentenceDummy>>>, rng: &mut rand::rngs::ThreadRng) {
    // * sample goal
    let sampledGoalOpt: Option<Arc<SentenceDummy>> = sample(&goalSystem, rng);

    if !sampledGoalOpt.is_some() {
        return; // no goal was sampled -> give up
    }
    let sampledGoal = sampledGoalOpt.unwrap();

    // * try to find candidates for inference
    let envidenceCandidates: Vec<Rc<RefCell<SentenceDummy>>> = retBeliefCandidates(&sampledGoal, evidence);

    // * try to do inference
    let mut concls:Vec<Arc<SentenceDummy>> = Vec::new();
    for iBelief in &envidenceCandidates {
        let conclOpt:Option<SentenceDummy> = infer(&sampledGoal, &(**iBelief).borrow());
        if conclOpt.is_some() {
            concls.push(Arc::new(conclOpt.unwrap()));
        }
    }

    // * try to add goals
    for iConcl in &concls {
        addEntry(goalSystem, Arc::clone(iConcl));
    }
}



// helper
pub fn retDesire(goal: &SentenceDummy) -> Tv::Tv {
    retTv(&goal).unwrap() // interpret tv as desire
}

// helper for debugging: return all goals as text
pub fn dbgRetGoalsAsText(goalSystem: &GoalSystem) -> String {
    let mut res:String = String::new();

    for iv in &goalSystem.entries {
        res += &NarSentence::convSentenceTermPunctToStr(&(*iv).borrow().sentence, true);
        res += &"\n".to_string();
    }

    res
}