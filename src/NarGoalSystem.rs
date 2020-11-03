//! goal system
//!
//! #Mechanism
//! Stores a set of goals which are considered active.
//! #Mechanism sampling strategy
//! Samples goals currently by a uniform distribution.
//! Sampling is made more fair by biasing the sampling by the depth of the goal (which is stored in the goal).
use std::rc::Rc;
use core::cell::RefCell;
use std::sync::{Arc, Mutex};
use rand::Rng;
use parking_lot::RwLock;

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
use crate::NarUnify;

/// structure for the goal system
pub struct GoalSystem {
    /// we are storing the entries batched by depth
    pub batchesByDepth: Vec<Rc<RefCell<BatchByDepth>>>,
    /// max number of entries
    pub nMaxEntries: i64,
    /// soft limit of depth
    pub nMaxDepth: i64,

    /// is a goal satisfied if a event happens when the terms match up?
    ///
    /// is "true" for natural environments, but can be set to "false" to specialize the reasoner for "crisp" tasks such as playing board games or solving logic problems(?)
    pub cfg__enGoalSatisfaction: bool,

    /// is debugging of adding of goal entry enabled?
    pub cfg__dbg_enAddEntry: bool,
}

/// we need to fold entries by term to get deeper plans
///
/// this is because sampling has to be done by term, not by dt
pub struct EntryFoldedByTerm {
    /// folded term
    pub term:Term,
    /// entries which all have the same term
    pub entries:Vec<Rc<RefCell<Entry>>>,
}

/// entry for goal system
pub struct Entry {
    pub sentence: Arc<SentenceDummy>,
    pub utility: f64,
    /// evidence which was used to derive this sentence. This is used to create the anticipations
    /// sentence: (a, ^b)!
    /// evidence: (a, ^b) =/> c.  (actual impl seq was this)
    pub evidence: Option<Arc<RwLock<SentenceDummy>>>,
    /// time of the creation of this entry
    pub createTime: i64,
    /// depth of the goal
    pub depth: i64,
    /// from -1.0 to 1.0
    pub desirability: f32,
}

/// used to group entries by depth
pub struct BatchByDepth {
    pub groups: Vec<EntryFoldedByTerm>,
    pub depth: i64,
}

pub fn makeGoalSystem(nMaxEntries:i64, nMaxDepth: i64) -> GoalSystem {
    let mut batchesByDepth: Vec<Rc<RefCell<BatchByDepth>>> = vec![];
    for iDepth in 0..nMaxDepth {
        batchesByDepth.push(Rc::new(RefCell::new(BatchByDepth{groups: vec![], depth:iDepth,})));
    }
    
    GoalSystem {
        batchesByDepth: batchesByDepth,
        nMaxEntries: nMaxEntries,
        nMaxDepth: nMaxDepth,

        cfg__enGoalSatisfaction: true, // enable for natural environments
        cfg__dbg_enAddEntry: true, // for debugging
    }
}

/// return array of all entries, is a helper and shouldn't get called to often
pub fn retEntries(goalSystem: &GoalSystem) -> Vec<Rc<RefCell<Entry>>> {
    let mut res: Vec<Rc<RefCell<Entry>>> = vec![];
    for iEntry in &goalSystem.batchesByDepth {
        for iGroup in &iEntry.borrow().groups {
            for iVal in &iGroup.entries {
                res.push(Rc::clone(&iVal));
            }
        }
    }
    res
}

/// /param t is the procedural reasoner NAR time
pub fn addEntry(goalSystem: &mut GoalSystem, t:i64, goal: Arc<SentenceDummy>, evidence: Option<Arc<RwLock<SentenceDummy>>>, depth:i64) {
    if goalSystem.cfg__dbg_enAddEntry { // print goal which is tried to put into system
        if depth > 2 {
            println!("goal system: addEntry depth={} {}", depth, &NarSentence::convSentenceTermPunctToStr(&goal, true));
            //panic!("DEBUGGING SHOULD BE FINISHED BECAUSE WE HIT DEVGOAL");
        }
    };
    
    // we check for same stamp - ignore it if the goal is exactly the same, because we don't need to store same goals
    for iv in &retEntries(goalSystem) {
        if 
            // optimization< checking term first is faster! >
            checkEqTerm(&iv.borrow().sentence.term, &goal.term) && // is necessary, else we don't accept detached goals!
            NarStamp::checkSame(&iv.borrow().sentence.stamp, &goal.stamp)
        {
            return;
        }
    }

    addEntry2(goalSystem, Rc::new(RefCell::new(Entry{sentence:Arc::clone(&goal), utility:1.0, evidence:evidence, createTime:t, depth:depth, desirability:1.0})));
}

pub fn addEntry2(goalSystem: &mut GoalSystem, e: Rc<RefCell<Entry>>) {
    let chosenDepthIdx:usize = e.borrow().depth.min(goalSystem.nMaxDepth-1) as usize;
    dbg(&format!("addEntry depth = {} chosenDepthIdx = {}", e.borrow().depth, chosenDepthIdx));

    let chosenBatchRc:Rc<RefCell<BatchByDepth>> = Rc::clone(&goalSystem.batchesByDepth[chosenDepthIdx]);
    let mut chosenBatch = chosenBatchRc.borrow_mut();
    
    // now we need to add the entry to the batch
    //
    // a small problem is that the actual entries are stored by term of the sentence
    // * so we have to find the group with the same term if the group exists and add it there or
    // * add it as a new group
    
    { // try to search for group by e.sentence.term
        for iGroup in &mut chosenBatch.groups { // iterate over groups by term
            if checkEqTerm(&iGroup.term, &e.borrow().sentence.term) { // found entry?
                iGroup.entries.push(Rc::clone(&e)); // add entry
                return;
            }
        }
    }

    { // case to add it as a new group
        chosenBatch.groups.push(EntryFoldedByTerm {
                term:(*(e.borrow().sentence.term)).clone(),
                entries:vec![Rc::clone(&e)],
            }
        );
    }
}

/// called when it has to stay under AIKR
/// /param t is the procedural reasoner NAR time
pub fn limitMemory(goalSystem: &mut GoalSystem, t: i64) {
    let mut arr:Vec<Rc<RefCell<Entry>>> = retEntries(goalSystem); // working array with all entries
    
    dbg(&format!("nEntries={}", arr.len()));

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
        goalSystem.batchesByDepth.push(Rc::new(RefCell::new(BatchByDepth{groups: vec![], depth:iDepth,})));
    }

    // fill
    for iVal in arr {
        addEntry2(goalSystem, iVal);
    }
}

/// sample a goal from the goal table of the goal system
/// returns (sentence, depth)
pub fn sample(goalSystem: &GoalSystem, rng: &mut rand::rngs::ThreadRng) -> Option<(Arc<SentenceDummy>, i64)> {
    // select batch (or return)
    let selBatchRef = {

        let sumPriorities:f64 = goalSystem.batchesByDepth.iter().map(|iv| 
            if iv.borrow().groups.len() > 0 {1.0} else {0.0} // only consider batches which have groups
        ).sum();
    
        let selPriority:f64 = rng.gen_range(0.0, 1.0) * sumPriorities;

        dbg(&format!("sample sum prio={} selPrio={}", sumPriorities, selPriority));
    
        // select
        /* commented because it is old buggy code
        let mut selBatch:Rc<RefCell<BatchByDepth>> = Rc::clone(&goalSystem.batchesByDepth[goalSystem.batchesByDepth.len()-1]); // default, should never be used
        let mut sum:f64 = 0.0;
        for iv in &goalSystem.batchesByDepth {
            sum += iv.groups.len() > 0 {1.0} else {0.0};
            if sum >= selPriority {
                selBatch = Rc::clone(iv);
                break;
            }
        }
        */
        let mut selBatch:Rc<RefCell<BatchByDepth>> = Rc::clone(&goalSystem.batchesByDepth[0]); // default, should never be used
        let mut sum:f64 = 0.0;
        let mut idx=0;
        for iv in &goalSystem.batchesByDepth {
            if sum >= selPriority {
                break;
            }
            if iv.borrow().groups.len() > 0 {
                sum+=1.0;
                selBatch = Rc::clone(iv);
            };
            
            idx+=1;
        }

        dbg(&format!("sel byDepthIdx={}", idx));

        selBatch
    };
    let selBatch = selBatchRef.borrow();


    
    
    
    if selBatch.groups.len() == 0 {
        return None;
    }
    
    let entriesOfSelBatch: &Vec<EntryFoldedByTerm> = &selBatch.groups;
    let sumPriorities:f64 = entriesOfSelBatch.iter()
        .map(|iEntriesByTerm| { // map over entries-by-term
            let entries:&Vec<Rc<RefCell<Entry>>> = &iEntriesByTerm.entries;
            entries.iter().map(|iv| (iv.borrow().desirability as f64).max(0.0)).sum::<f64>() // compute inner sum
        }).sum();
    
    let selPriority:f64 = rng.gen_range(0.0, 1.0) * sumPriorities;

    // select
    let mut sum:f64 = 0.0;
    let mut selEntry = None;
    for iv in &selBatch.groups {
        assert!(sum <= sumPriorities); // priorities are summed in the wrong way in this loop if this invariant is violated
        
        for iEntry in &iv.entries {
            sum += (iEntry.borrow().desirability as f64).max(0.0); // desired goals should be favored to get sampled
            let selEntry2 = iEntry.borrow();
            selEntry = Some((Arc::clone(&selEntry2.sentence), selEntry2.depth));
            if sum >= selPriority {
                break;
            }
        }
    }

    selEntry
}

/// does inference of goal with a belief
/// returns derivation
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



/// filters belief candidates which can be used for inference with the goal
pub fn retBeliefCandidates(goal: &SentenceDummy, procMem:&NarMem::Mem) -> Vec<Arc<RwLock<SentenceDummy>>> {
    let mut res = Vec::new();

    // query memory for potential evidence which we can use
    let potentialEvidence = NarMem::ret_beliefs_by_terms_nonunique(procMem, &[(*goal.term).clone()]);
    
    // filter
    for iBelief in &potentialEvidence {
        match &*(iBelief.read()).term {
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


/// select highest ranked goal for state
/// returns entity and unified result
pub fn selHighestExpGoalByState(goalSystem: &GoalSystem, state:&Term) -> (f64, Option<(Rc<RefCell<Entry>>, Term)>) {
    let mut res:(f64, Option<(Rc<RefCell<Entry>>, Term)>) = (0.0, None);

    for iv in &retEntries(goalSystem) {
        match &(*(iv.borrow().sentence).term) {
            
            Term::Seq(seq) if seq.len() >= 1 => {
                // try to unify first part of sequence with observed state
                // we can only consider if it matches!
                let asgnmts:Option<Vec<NarUnify::Asgnment>> = NarUnify::unify(&seq[0], &state);

                if asgnmts.is_some() { // does first event of seq match to state with unification?, must unify!
                    let exp = Tv::calcExp(&retTv(&iv.borrow().sentence).unwrap());
                    let (resExp, _) = res;
                    if exp > resExp {
                        let unifiedTerm: Term = NarUnify::unifySubst(&*(iv.borrow().sentence).term, &asgnmts.unwrap()); // unify because we need term with less or no variables!
                        res = (exp, Some((Rc::clone(&iv), unifiedTerm)));
                    }
                }
            },
            _ => {}
        }

    }

    res
}

/// /param t is the procedural reasoner NAR time
pub fn sampleAndInference(goalSystem: &mut GoalSystem, t:i64, procMem:&NarMem::Mem, rng: &mut rand::rngs::ThreadRng) {
    // * sample goal
    let sampledGoalOpt: Option<(Arc<SentenceDummy>, i64)> = sample(&goalSystem, rng);

    if !sampledGoalOpt.is_some() {
        return; // no goal was sampled -> give up
    }
    let (sampledGoal, sampledDepth): (Arc<SentenceDummy>, i64) = sampledGoalOpt.unwrap();

    let mut concls:Vec<(Arc<SentenceDummy>, Option<Arc<RwLock<SentenceDummy>>>, i64)> = Vec::new(); // conclusions are tuple (goal, evidence, depth)
    
    // * try to do goal detachment
    match &*sampledGoal.term {
        Term::Seq(seq) if seq.len() >= 1 => {
            let detachedGoal:SentenceDummy = newEternalSentenceByTv(&seq[0],EnumPunctation::GOAL,&retTv(&sampledGoal).unwrap(),sampledGoal.stamp.clone());
            dbg(&format!("detached goal {}", &NarSentence::convSentenceTermPunctToStr(&detachedGoal, true)));
            concls.push((Arc::new(detachedGoal), None, sampledDepth+1));
        },
        _ => {
            // * try to find candidates for inference
            let envidenceCandidates: Vec<Arc<RwLock<SentenceDummy>>> = retBeliefCandidates(&sampledGoal, procMem);

            // * try to do inference
            for iBelief in &envidenceCandidates {
                let conclOpt:Option<SentenceDummy> = infer(&sampledGoal, &iBelief.read());
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
        if goalSystem.cfg__enGoalSatisfaction && // do we want to satisfy goals?
           checkEqTerm(&iEntity.sentence.term, eventTerm) // terms must of course to match up that a event can satify the goal
        {
            iEntity.desirability = 0.0; // set desirability to 0.0 because it happened
        }
    }
}




// helper
pub fn retDesire(goal: &SentenceDummy) -> Tv::Tv {
    retTv(&goal).unwrap() // interpret tv as desire
}

/// helper for debugging: return all goals as text
pub fn dbgRetGoalsAsText(goalSystem: &GoalSystem) -> String {
    let mut res:String = String::new();

    for iv in &retEntries(goalSystem) {
        let sentenceAsStr = NarSentence::convSentenceTermPunctToStr(&(*iv).borrow().sentence, true);
        res += &format!("{}   util={}\n", &sentenceAsStr, &(*iv).borrow().utility);
    }

    res
}



/// helper for debugging
pub fn dbg(str2:&String) {
    if true { // don't we want to debug?
        return;
    }

    println!("DBG {}", str2);
}
