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
use crate::NarInfProcedural;
use crate::Utils::{enforce};

pub struct ActiveSet {
	pub set: Vec< Rc<RefCell<Entry>> >,
}


/// structure for the goal system
pub struct GoalSystem {
    /// we store op goals seperatly as a optimization
    pub activeSet: ActiveSet,

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

    /// how many times are goals sample in the sub-working-cycle?
    pub cfg__subworkingCycle_rounds:i64,
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


    pub utility: f64,
    /// helper used to keep track of the accumulated selection criteria up to this item in the array
    pub accDesirability: f64,
}

/// returns if goal is desired
pub fn is_desired(e:&Entry) -> bool {
    e.desirability > 0.0001 // is bigger than theshold, where threshold is epsilon
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
        activeSet: ActiveSet{set: vec![]},

        batchesByDepth: batchesByDepth,
        nMaxEntries: nMaxEntries,
        nMaxDepth: nMaxDepth,

        cfg__enGoalSatisfaction: true, // enable for natural environments
        cfg__dbg_enAddEntry: true, // for debugging

        cfg__subworkingCycle_rounds:15,
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

    for iVal in &goalSystem.activeSet.set {
        res.push(Rc::clone(&iVal));
    }

    res
}

/// /param t is the procedural reasoner NAR time
pub fn addEntry(goalSystem: &mut GoalSystem, t:i64, goal: Arc<SentenceDummy>, evidence: Option<Arc<RwLock<SentenceDummy>>>, depth:i64) {
    enforce(goal.punct == EnumPunctation::GOAL); // must be a goal!
    
    if goalSystem.cfg__dbg_enAddEntry { // print goal which is tried to put into system
        if depth > 2 {
            //println!("goal system: addEntry depth={} {}", depth, &NarSentence::convSentenceTermPunctToStr(&goal, true));
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

    addEntry2(goalSystem, Rc::new(RefCell::new(Entry{sentence:Arc::clone(&goal), utility:1.0, evidence:evidence, createTime:t, depth:depth, desirability:1.0, accDesirability:0.0})));
}

/// helper to add goal
// private because it is a helper
fn addEntry2(goalSystem: &mut GoalSystem, e: Rc<RefCell<Entry>>) {
    
    // decide if we add the goal to the active set
    use crate::TermUtils::decodeOp;
    let isOp = decodeOp(&e.borrow().sentence.term).is_some();
	if isOp {
		if checkSetContains(&goalSystem.activeSet.set, &e.borrow().sentence) {
			return;
		}

		goalSystem.activeSet.set.push(Rc::clone(&e));
	}
	else {
        // usual code for adding it to the "big" set


        let chosenDepthIdx:usize = e.borrow().depth.min(goalSystem.nMaxDepth-1) as usize;
        //dbg(&format!("addEntry depth = {} chosenDepthIdx = {}", e.borrow().depth, chosenDepthIdx));
    
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
                    let newAcc:f64 = iGroup.entries[iGroup.entries.len()-1].borrow().accDesirability+iGroup.entries[iGroup.entries.len()-1].borrow().desirability.max(0.0) as f64; // compute acc utility for this item
                    e.borrow_mut().accDesirability = newAcc; // update
    
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
}

// privte because helper
fn checkSetContains(set: &Vec< Rc<RefCell<Entry>> >, s: &SentenceDummy) -> bool {
    // we check for same stamp - ignore it if the goal is exactly the same, because we don't need to store same goals
    for iv in set {
        if 
            // optimization< checking term first is faster! >
            checkEqTerm(&iv.borrow().sentence.term, &s.term) && // is necessary, else we don't accept detached goals!
            NarStamp::checkSame(&iv.borrow().sentence.stamp, &s.stamp)
        {
            return true;
        }
    }
    false
}

/// called when it has to stay under AIKR
/// /param t is the procedural reasoner NAR time
pub fn limitMemory(goalSystem: &mut GoalSystem, t: i64) {
    let mut arr:Vec<Rc<RefCell<Entry>>> = retEntries(goalSystem); // working array with all entries
    
    for iv in &goalSystem.activeSet.set {
        arr.push(Rc::clone(&iv));
    }

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

        //dbg(&format!("sample sum prio={} selPrio={}", sumPriorities, selPriority));
    
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

        //dbg(&format!("sel byDepthIdx={}", idx));

        selBatch
    };
    let selBatch = selBatchRef.borrow();


    
    
    
    if selBatch.groups.len() == 0 {
        return None;
    }
    
    let entriesOfSelBatch: &Vec<EntryFoldedByTerm> = &selBatch.groups;
    
    let sumPriorities:f64 = entriesOfSelBatch.iter()
        .map(|iEntriesByTerm| { // map over entries-by-term
            let lastEntry = &iEntriesByTerm.entries[iEntriesByTerm.entries.len()-1].borrow();
            let des2:f64 = lastEntry.accDesirability+lastEntry.desirability as f64;
            des2 // return inner sum
        }).sum();
    
    let selPriority:f64 = rng.gen_range(0.0, 1.0) * sumPriorities;

    // select
    let mut sum:f64 = 0.0;
    let mut selEntry = None;
    for iv in &selBatch.groups {
        assert!(sum <= sumPriorities); // priorities are summed in the wrong way in this loop if this invariant is violated
        
        let lastEntry = &iv.entries[iv.entries.len()-1].borrow();
        let des2:f64 = lastEntry.accDesirability+lastEntry.desirability as f64;

        if des2+sum > selPriority { // is the range of the selection inside this "group"?

            use crate::BinSearch::binSearch;
            let selEntry3: Rc<RefCell<Entry>> = binSearch(&iv.entries, selPriority);
            let selEntry2 = selEntry3.borrow();
            selEntry = Some((Arc::clone(&selEntry2.sentence), selEntry2.depth));
            break;

            /* old slow loop
            for iEntry in &iv.entries {
                sum += (iEntry.borrow().desirability as f64).max(0.0); // desired goals should be favored to get sampled
                let selEntry2 = iEntry.borrow();
                
                if sum >= selPriority {
                    break;
                }
            }
            */

        }
        else {
            sum+=des2;
        }
    }

    selEntry
}



/// filters belief candidates by antecedent
/// ex:
/// queryterm: a
/// belief (a, x) =/> b
/// belief (b, x) =/> b
/// returns
/// belief (a, x) =/> b
pub fn query_by_antecedent(queryTerm: &Term, procMem:&NarMem::Mem) -> Vec<Arc<RwLock<SentenceDummy>>> {
    let mut res = Vec::new();

    // query memory for potential evidence which we can use
    let potentialEvidence = NarMem::ret_beliefs_by_terms_nonunique(procMem, &[(*queryTerm).clone()]);
    
    // filter
    for iBelief in &potentialEvidence {
        match &*(iBelief.read()).term {
            Term::Stmt(Copula::PREDIMPL, subj, _pred) => {
                match &**subj {
                    Term::Seq(seq) if checkEqTerm(&queryTerm, &seq[0]) => {
                        res.push(Arc::clone(iBelief));
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    res
}

/// filters belief candidates which can be used for inference with the goal
pub fn query_by_consequence(queryTerm: &Term, procMem:&NarMem::Mem) -> Vec<Arc<RwLock<SentenceDummy>>> {
    let mut res = Vec::new();

    // query memory for potential evidence which we can use
    let potentialEvidence = NarMem::ret_beliefs_by_terms_nonunique(procMem, &[(*queryTerm).clone()]);
    
    // filter
    for iBelief in &potentialEvidence {
        match &*(iBelief.read()).term {
            Term::Stmt(Copula::PREDIMPL, _subj, pred) => {
                if checkEqTerm(&queryTerm, &pred) {
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

/// helper struct for deriver
/// carries goal with evidence and the depth of the goal
struct H {
    goal: Arc<SentenceDummy>,
    evidence: Option<Arc<RwLock<SentenceDummy>>>,
    depth: i64,
}

/// enum to tell the strategy of deriving goals
enum EnumGoalDerivationStrategy {
    /// derive from all beliefs
    ALL_BELIEFS,

    /// sample two random beliefs
    SAMPLE_2,
}

// private because helper for sampleAndInference()
/// sampledDepth: depth of sampled goal
fn deriveGoalsHelper(sampledGoal: &SentenceDummy, sampledDepth:i64, strategy:EnumGoalDerivationStrategy, procMem:&NarMem::Mem, rng: &mut rand::rngs::ThreadRng)->Vec<H> {
    let mut concls:Vec<H> = Vec::new(); // conclusions

    match NarInfProcedural::infGoalDetach(&sampledGoal) {
        Some(concl) => {
            concls.push(H{goal:Arc::new(concl), evidence:None, depth:sampledDepth+1});
        },
        _ => {
            // * try to find candidates for inference
            let evidenceCandidates: Vec<Arc<RwLock<SentenceDummy>>> = query_by_consequence(&sampledGoal.term, procMem);

            //dbg(&format!("sampleAndInference() found belief candidates #={}", evidenceCandidates.len()));

            // * try to do inference
            match strategy {
                ALL_BELIEFS => {
                    for iBelief in &evidenceCandidates {
                        let conclOpt:Option<SentenceDummy> = NarInfProcedural::infGoalBelief(&sampledGoal, &iBelief.read());
                        if conclOpt.is_some() {
                            concls.push(H{goal:Arc::new(conclOpt.unwrap()), evidence:Some(Arc::clone(iBelief)), depth:sampledDepth+1});
                        }
                    }
                },
                SAMPLE_2 => {
                    for _it in 0..2 {
                        if evidenceCandidates.len() > 0 {
                            let idx = rng.gen_range(0, evidenceCandidates.len());
                            let conclOpt:Option<SentenceDummy> = NarInfProcedural::infGoalBelief(&sampledGoal, &evidenceCandidates[idx].read());
                            if conclOpt.is_some() {
                                concls.push(H{goal:Arc::new(conclOpt.unwrap()), evidence:Some(Arc::clone(&evidenceCandidates[idx])), depth:sampledDepth+1});
                            }
                        }
                    }
                }
            }

        }
    }

    concls
}


/// /param t is the procedural reasoner NAR time
pub fn sampleAndInference(goalSystem: &mut GoalSystem, t:i64, procMem:&NarMem::Mem, rng: &mut rand::rngs::ThreadRng) {
    // * sample goal from set of goals
    let sampledGoalOpt: Option<(Arc<SentenceDummy>, i64)> = sample(&goalSystem, rng);

    if !sampledGoalOpt.is_some() {
        return; // no goal was sampled -> give up
    }
    let (sampledGoal, sampledDepth): (Arc<SentenceDummy>, i64) = sampledGoalOpt.unwrap();

    let mut concls:Vec<(Arc<SentenceDummy>, Option<Arc<RwLock<SentenceDummy>>>, i64)> = Vec::new(); // conclusions are tuple (goal, evidence, depth)
    
    //dbg(&format!("sampleAndInference() sampled goal = {}", &NarSentence::convSentenceTermPunctToStr(&sampledGoal, true)));

    {
        // we need a structure to store goals in the working set with some meta-information
        struct WsEntry { // working set entry
            pub goal: Arc<SentenceDummy>, // goal
            pub sampledDepth: i64, // the depth of the goal
        }

        // put sampled goal into working set
        let mut workingSet: Vec<Rc<WsEntry>> = vec![];
        workingSet.push(Rc::new(WsEntry {
            goal: Arc::clone(&sampledGoal),
            sampledDepth: sampledDepth,
        }));

        // MECHANISM "sub working cycle"<
        //    we sample a goal from the global set of goals and put it as the "entry" goal into the working set
        //    
        //    we then derive goals in the working set until we run out of (compute) time.
        //    this way it's possible to deeply examine some sub-goals of goals
        //
        //    all conclusions are immediatly put back into the global set of goals!
        // >
        for _iRound in 0..goalSystem.cfg__subworkingCycle_rounds {
            if workingSet.len() == 0 {
                break; // no more goals -> we are done
            }

            // sample goal from workingSet

            let sampledWsEntry: Rc<WsEntry> = {
                let selIdx = rng.gen_range(0, workingSet.len());
                workingSet.swap_remove(selIdx) // select and remove element
            };

            // do actual derivations!
            for iconcl in deriveGoalsHelper(&sampledWsEntry.goal, sampledWsEntry.sampledDepth, /* pick in deriveGoalsHelper() only two beliefs */ EnumGoalDerivationStrategy::SAMPLE_2, &procMem, rng) {
                workingSet.push(Rc::new(WsEntry{goal:iconcl.goal.clone(), sampledDepth:iconcl.depth})); // add to working set for processing
                concls.push((iconcl.goal, iconcl.evidence, iconcl.depth)); // add to conclusions
            }
        }
    }





    
    { // old mechanism
      // process only sampled goal
      // we need to do this additional to the other mechanism, because the other mechanism doesn't process all belief candidates!
        for iconcl in deriveGoalsHelper(&sampledGoal, sampledDepth, EnumGoalDerivationStrategy::ALL_BELIEFS, procMem, rng) {
            concls.push((iconcl.goal, iconcl.evidence, iconcl.depth));
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

    for iEntityRc in &goalSystem.activeSet.set {
        let mut iEntity = iEntityRc.borrow_mut();
        if goalSystem.cfg__enGoalSatisfaction && // do we want to satisfy goals?
           checkEqTerm(&iEntity.sentence.term, eventTerm) // terms must of course to match up that a event can satify the goal
        {
            iEntity.desirability = 0.0; // set desirability to 0.0 because it happened
        }
    }
}

/// returns the goal which matches with the term
pub fn query(goalSystem: &GoalSystem, eventTerm:&Term) -> Option<Rc<RefCell<Entry>>> {
    // TODO< select goal with highest exp! >

    for iEntityRc in retEntries(goalSystem) {
        let iEntity = iEntityRc.borrow_mut();
        if checkEqTerm(&iEntity.sentence.term, eventTerm) { // terms must of course to match up
            return Some(Rc::clone(&iEntityRc));
        }
    }

    for iEntityRc in &goalSystem.activeSet.set {
        let iEntity = iEntityRc.borrow_mut();
        if checkEqTerm(&iEntity.sentence.term, eventTerm) { // terms must of course to match up
            return Some(Rc::clone(&iEntityRc));
        }
    }

    None
}



/// helper for debugging: return all goals as text
pub fn dbgRetGoalsAsText(goalSystem: &GoalSystem) -> String {
    let mut res:String = String::new();

    for iv in &retEntries(goalSystem) {
        let sentenceAsStr = NarSentence::convSentenceTermPunctToStr(&(*iv).borrow().sentence, true);
        res += &format!("{}   util={} depth={}\n", &sentenceAsStr, &(*iv).borrow().utility, iv.borrow().depth);
    }

    res
}










/// helper for debugging
pub fn dbg(str2:&String) {
    if false { // don't we want to debug?
        return;
    }

    println!("DBG {}", str2);
}
