// goal system

use std::rc::Rc;
use core::cell::RefCell;
use std::sync::{Arc, Mutex};
use rand::Rng;

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
use crate::NarWorkingCycle;
use crate::NarMem;

pub struct GoalSystem {
    //pub entries: Vec<Rc<RefCell<Entry>>>, //REMOVE!!!
    pub batchesByDepth: Vec<Rc<RefCell<BatchByDepth>>>, // we are storing the entries batched by depth
    pub nMaxEntries: i64, // max number of entries
    pub nMaxDepth: i64, // soft limit of depth
}

// entry for goal system
pub struct Entry {
    pub sentence: Arc<SentenceDummy>,
    pub utility: f64,
    pub evidence: Option<Arc<Mutex<SentenceDummy>>>, // evidence which was used to derive this sentence. This is used to create the anticipations
                                                      // sentence: (a, ^b)!
                                                      // evidence: (a, ^b) =/> c.  (actual impl seq was this)
    pub createTime: i64, // time of the creation of this entry
    pub depth: i64, // depth of the goal
    pub desirability: f32, // from -1.0 to 1.0
}

pub struct BatchByDepth {
    pub entries: Vec<Rc<RefCell<Entry>>>,
    pub depth: i64,
}

pub fn makeGoalSystem(nMaxEntries:i64, nMaxDepth: i64) -> GoalSystem {
    let mut batchesByDepth: Vec<Rc<RefCell<BatchByDepth>>> = vec![];
    for iDepth in 0..nMaxDepth {
        batchesByDepth.push(Rc::new(RefCell::new(BatchByDepth{entries: vec![], depth:iDepth,})));
    }
    //println!("XXX {}", batchesByDepth.len());
    
    GoalSystem {
        batchesByDepth: batchesByDepth,
        nMaxEntries: nMaxEntries,
        nMaxDepth: nMaxDepth,
    }
}

// return array of all entries, is a helper and shouldn't get called to often
pub fn retEntries(goalSystem: &GoalSystem) -> Vec<Rc<RefCell<Entry>>> {
    let mut res: Vec<Rc<RefCell<Entry>>> = vec![];
    for iEntry in &goalSystem.batchesByDepth {
        for iVal in &iEntry.borrow().entries {
            res.push(Rc::clone(iVal));
        }
    }
    res
}

// /param t is the procedural reasoner NAR time
pub fn addEntry(goalSystem: &mut GoalSystem, t:i64, goal: Arc<SentenceDummy>, evidence: Option<Arc<Mutex<SentenceDummy>>>, depth:i64) {
    if false {println!("goal system: addEntry {}", &NarSentence::convSentenceTermPunctToStr(&goal, true))}; // print goal which is tried to put into system
    
    // we check for same stamp - ignore it if the goal is exactly the same, because we don't need to store same goals
    for iv in &retEntries(goalSystem) {
        if 
            // optimization< checking term first is faster! >
            //checkEqTerm(&iv.sentence.term, &goal.term) &&
            NarStamp::checkSame(&iv.borrow().sentence.stamp, &goal.stamp)
        {
            return;
        }
    }

    addEntry2(goalSystem, Rc::new(RefCell::new(Entry{sentence:Arc::clone(&goal), utility:1.0, evidence:evidence, createTime:t, depth:depth, desirability:1.0})));
}

pub fn addEntry2(goalSystem: &mut GoalSystem, e: Rc<RefCell<Entry>>) {
    let chosenBatch:Rc<RefCell<BatchByDepth>> = Rc::clone(&goalSystem.batchesByDepth[e.borrow().depth.min(goalSystem.nMaxDepth-1) as usize]);
    chosenBatch.borrow_mut().entries.push(e);
}

// called when it has to stay under AIKR
// /param t is the procedural reasoner NAR time
pub fn limitMemory(goalSystem: &mut GoalSystem, t: i64) {
    let mut arr:Vec<Rc<RefCell<Entry>>> = retEntries(goalSystem); // working array with all entries
    
    // * recalc utility
    for iv in &arr {
        let mut iv2 = iv.borrow_mut();
        // consider age
        let age: i64 = t-iv2.createTime;
        let decay = ((age as f64)*-0.01).exp(); // compute decay by age

        iv2.utility = 
            Tv::calcExp(&retTv(&iv2.sentence).unwrap())*
            (iv2.desirability as f64).abs()* // times the desirability because not so desired goals should get forgotten
            decay;
    }

    // * sort by utility
    arr.sort_by(|a, b| b.borrow().utility.partial_cmp(&a.borrow().utility).unwrap());

    // * limit (AIKR)
    while arr.len() as i64 > goalSystem.nMaxEntries {
        arr.remove(goalSystem.nMaxEntries as usize);
    }

    goalSystem.batchesByDepth = vec![]; // flush
    // rebuild
    for iDepth in 0..goalSystem.nMaxDepth {
        goalSystem.batchesByDepth.push(Rc::new(RefCell::new(BatchByDepth{entries: vec![], depth:iDepth,})));
    }

    // fill
    for iVal in arr {
        addEntry2(goalSystem, iVal);
    }
}

// sample a goal from the goal table of the goal system
// returns (sentence, depth)
pub fn sample(goalSystem: &GoalSystem, rng: &mut rand::rngs::ThreadRng) -> Option<(Arc<SentenceDummy>, i64)> {
    // select batch (or return)
    let selBatchRef = {

        let sumPriorities:f64 = goalSystem.batchesByDepth.iter().map(|iv| 1.0).sum();
    
        let selPriority:f64 = rng.gen_range(0.0, 1.0) * sumPriorities;
    
        // select
        let mut selBatch:Rc<RefCell<BatchByDepth>> = Rc::clone(&goalSystem.batchesByDepth[0]); // default, should never be used
        let mut sum:f64 = 0.0;
        for iv in &goalSystem.batchesByDepth {
            sum += 1.0;
            if sum >= selPriority {
                selBatch = Rc::clone(iv);
                break;
            }
        }
        selBatch
    };
    let selBatch = selBatchRef.borrow();


    
    
    
    if selBatch.entries.len() == 0 {
        return None;
    }
    
    let sumPriorities:f64 = selBatch.entries.iter().map(|iv| (iv.borrow().desirability as f64).max(0.0)).sum();
    
    let selPriority:f64 = rng.gen_range(0.0, 1.0) * sumPriorities;

    // select
    let mut sum:f64 = 0.0;
    for iv in &selBatch.entries {
        assert!(sum <= sumPriorities); // priorities are summed in the wrong way in this loop if this invariant is violated
        sum += (iv.borrow().desirability as f64).max(0.0); // desired goals should be favored to get sampled
        if sum >= selPriority {
            let selEntry = iv.borrow();
            return Some((Arc::clone(&selEntry.sentence), selEntry.depth));
        }
    }

    // shouldn't happen
    let selEntry = selBatch.entries[selBatch.entries.len()-1].borrow();
    return Some((Arc::clone(&selEntry.sentence), selEntry.depth));
}

// does inference of goal with a belief
// returns derivation
pub fn infer(goal: &SentenceDummy, belief: &SentenceDummy)-> Option<SentenceDummy> {
    // check if term is same and inference can be done
    match &*belief.term {
        Term::Stmt(Copula::PREDIMPL, _subj, pred) => {
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
pub fn retBeliefCandidates(goal: &SentenceDummy, procMem:&NarMem::Mem) -> Vec<Arc<Mutex<SentenceDummy>>> {
    let mut res = Vec::new();

    // query memory for potential evidence which we can use
    let potentialEvidence = NarMem::ret_beliefs_by_terms_nonunique(procMem, &[(*goal.term).clone()]);
    
    // filter
    for iBelief in &potentialEvidence {
        match &*(iBelief.lock().unwrap()).term {
            Term::Stmt(Copula::PREDIMPL, _subj, pred) => {
                if checkEqTerm(&goal.term, &pred) {
                    res.push(Arc::clone(iBelief));
                }
            },
            _ => {}
        }
    }

    res
}


// select highest ranked goal for state
// returns entity and unified result
pub fn selHighestExpGoalByState(goalSystem: &GoalSystem, state:&Term) -> (f64, Option<(Rc<RefCell<Entry>>, Term)>) {
    let mut res:(f64, Option<(Rc<RefCell<Entry>>, Term)>) = (0.0, None);

    for iv in &retEntries(goalSystem) {
        match &(*(iv.borrow().sentence).term) {
            
            Term::Seq(seq) if seq.len() >= 1 => {
                // try to unify first part of sequence with observed state
                // we can only consider if it matches!
                let asgnmts:Option<Vec<NarWorkingCycle::Asgnment>> = NarWorkingCycle::unify(&seq[0], &state);

                if asgnmts.is_some() { // does first event of seq match to state with unification?, must unify!
                    let exp = Tv::calcExp(&retTv(&iv.borrow().sentence).unwrap());
                    let (resExp, _) = res;
                    if exp > resExp {
                        let unifiedTerm: Term = NarWorkingCycle::unifySubst(&*(iv.borrow().sentence).term, &asgnmts.unwrap()); // unify because we need term with less or no variables!
                        res = (exp, Some((Rc::clone(&iv), unifiedTerm)));
                    }
                }
            },
            _ => {}
        }

    }

    res
}

// /param t is the procedural reasoner NAR time
//pub fn sampleAndInference(goalSystem: &mut GoalSystem, t:i64, procNar:&NarProc::ProcNar, rng: &mut rand::rngs::ThreadRng) {
pub fn sampleAndInference(goalSystem: &mut GoalSystem, t:i64, procMem:&NarMem::Mem, rng: &mut rand::rngs::ThreadRng) {
    // * sample goal
    let sampledGoalOpt: Option<(Arc<SentenceDummy>, i64)> = sample(&goalSystem, rng);

    if !sampledGoalOpt.is_some() {
        return; // no goal was sampled -> give up
    }
    let (sampledGoal, sampledDepth): (Arc<SentenceDummy>, i64) = sampledGoalOpt.unwrap();

    let mut concls:Vec<(Arc<SentenceDummy>, Option<Arc<Mutex<SentenceDummy>>>, i64)> = Vec::new(); // conclusions are tuple (goal, evidence, depth)
    
    // * try to do goal detachment
    match &*sampledGoal.term {
        Term::Seq(seq) if seq.len() >= 1 => {
            let detachedGoal:SentenceDummy = newEternalSentenceByTv(&seq[0],EnumPunctation::GOAL,&retTv(&sampledGoal).unwrap(),sampledGoal.stamp.clone());
            //dbg(format!("dbg: detached goal {}", &NarSentence::convSentenceTermPunctToStr(&detachedGoal, true)));
            concls.push((Arc::new(detachedGoal), None, sampledDepth));
        },
        _ => {
            // * try to find candidates for inference
            let envidenceCandidates: Vec<Arc<Mutex<SentenceDummy>>> = retBeliefCandidates(&sampledGoal, procMem);

            // * try to do inference
            for iBelief in &envidenceCandidates {
                let conclOpt:Option<SentenceDummy> = infer(&sampledGoal, &iBelief.lock().unwrap());
                if conclOpt.is_some() {
                    concls.push((Arc::new(conclOpt.unwrap()), Some(Arc::clone(iBelief)), sampledDepth+1));
                }
            }
        }
    }

    // * try to add goals
    for (iGoal, iEvidence, iDepth) in &concls {
        let iEvidence2 = match iEvidence { // clone evidence
            Some(e) => {Some(Arc::clone(e))}
            None => None
        };
        addEntry(goalSystem, t, Arc::clone(iGoal), iEvidence2, *iDepth);
    }
}

/// called from outside when event happened
pub fn event_occurred(goalSystem: &mut GoalSystem, eventTerm:&Term) {
    for iEntityRc in retEntries(goalSystem) {
        let mut iEntity = iEntityRc.borrow_mut();
        if checkEqTerm(&iEntity.sentence.term, eventTerm) {
            iEntity.desirability = 0.0; // set desirability to 0.0 because it happened
        }
    }
}




// helper
pub fn retDesire(goal: &SentenceDummy) -> Tv::Tv {
    retTv(&goal).unwrap() // interpret tv as desire
}

// helper for debugging: return all goals as text
pub fn dbgRetGoalsAsText(goalSystem: &GoalSystem) -> String {
    let mut res:String = String::new();

    for iv in &retEntries(goalSystem) {
        let sentenceAsStr = NarSentence::convSentenceTermPunctToStr(&(*iv).borrow().sentence, true);
        res += &format!("{}   util={}\n", &sentenceAsStr, &(*iv).borrow().utility);
    }

    res
}