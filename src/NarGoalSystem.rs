//! goal system
//!
//! #Mechanism
//! Stores a set of goals which are considered active.
//! #Mechanism sampling strategy
//! Samples goals currently by a uniform distribution.
//! Sampling is made more fair by biasing the sampling by the depth of the goal (which is stored in the goal).
use std::rc::Rc;
use std::sync::{Arc};
use std::collections::HashMap;
use rand::Rng;
use parking_lot::RwLock;

use crate::Term::*;

//use crate::Tv::calcExp;
//use crate::Tv::ded;
use crate::Tv;

use crate::NarStamp;

use crate::NarSentence;
use crate::NarSentence::EnumPunctation;
use crate::NarSentence::Sentence;
use crate::NarSentence::retTv;
use crate::NarSentence::newEternalSentenceByTv;
use crate::NarSentence::convSentenceTermPunctToStr;
use crate::NarSentence::shallowCopySentence;
use crate::NarMem;
use crate::NarUnify;
use crate::NarInfProcedural;
use crate::Utils::{enforce};
use crate::NarStamp::newStamp;
use crate::NarWorkingCycle::Task2;
use crate::NarWorkingCycle::QHandler;
use crate::NarWorkingCycle::Mem2;

pub struct ActiveSet {
	pub set: Vec< Arc<RwLock<Entry>> >,
}


/// structure for the goal system
pub struct GoalSystem {
    pub queuedProcQaBridgeAnswers: Vec<QueuedProcQaBridgeAnswer>,

    /// we store op goals seperatly as a optimization
    pub activeSet: ActiveSet,

    /// we are storing the entries batched by depth
    pub batchesByDepth: Vec<Arc<RwLock<BatchByDepth>>>,
    /// max number of entries
    pub nMaxEntries: i64,
    /// soft limit of depth
    pub nMaxDepth: i64,

    /// used to speed up queries
    pub entriesByTerm: HashMap<Term, RwLock<Vec< Arc<RwLock<Entry>> > > >,

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
    pub entries:Vec<Arc<RwLock<Entry>>>,
}

/// entry for goal system
pub struct Entry {
    pub sentence: Arc<Sentence>,
    
    /// evidence which was used to derive this sentence. This is used to create the anticipations
    /// sentence: (a, ^b)!
    /// evidence: (a, ^b) =/> c.  (actual impl seq was this)
    pub evidence: Option<Arc<RwLock<Sentence>>>,
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

/// answer of procedural / Q&A bridge - which is queued for processing in NarProc.rs
pub struct QueuedProcQaBridgeAnswer {
    pub entry: Arc<RwLock<Entry>>,
    pub answer: Sentence,
}

pub fn makeGoalSystem(nMaxEntries:i64, nMaxDepth: i64) -> GoalSystem {
    let mut batchesByDepth: Vec<Arc<RwLock<BatchByDepth>>> = vec![];
    for iDepth in 0..nMaxDepth {
        batchesByDepth.push(Arc::new(RwLock::new(BatchByDepth{groups: vec![], depth:iDepth,})));
    }
    
    GoalSystem {
        queuedProcQaBridgeAnswers: vec![],

        activeSet: ActiveSet{set: vec![]},

        batchesByDepth: batchesByDepth,
        nMaxEntries: nMaxEntries,
        nMaxDepth: nMaxDepth,

        entriesByTerm:HashMap::new(),

        cfg__enGoalSatisfaction: true, // enable for natural environments
        cfg__dbg_enAddEntry: true, // for debugging

        cfg__subworkingCycle_rounds:15,
    }
}

/// return array of all entries, is a helper and shouldn't get called to often
pub fn retEntries(goalSystem: &GoalSystem) -> Vec<Arc<RwLock<Entry>>> {
    let mut res: Vec<Arc<RwLock<Entry>>> = vec![];
    for iEntry in &goalSystem.batchesByDepth {
        for iGroup in &iEntry.read().groups {
            for iVal in &iGroup.entries {
                res.push(Arc::clone(&iVal));
            }
        }
    }

    for iVal in &goalSystem.activeSet.set {
        res.push(Arc::clone(&iVal));
    }

    res
}

/// /param t is the procedural reasoner NAR time
pub fn addEntry(goalSystem: &Arc<RwLock<GoalSystem>>, mem2: &Mem2, t:i64, goal: Arc<Sentence>, evidence: Option<Arc<RwLock<Sentence>>>, depth:i64) {
    enforce(goal.punct == EnumPunctation::GOAL); // must be a goal!
    
    if goalSystem.read().cfg__dbg_enAddEntry { // print goal which is tried to put into system
        if depth > 2 {
            //println!("goal system: addEntry depth={} {}", depth, &NarSentence::convSentenceTermPunctToStr(&goal, true));
        }
    };
    
    // we check for same stamp - ignore it if the goal is exactly the same, because we don't need to store same goals
    {
        match goalSystem.write().entriesByTerm.get_mut(&goal.term.clone()) {
            Some(arr) => {
                let arr2 = &*arr.read();
                {
                    {
                        let mut exists = false;
                        for iv in arr2 {
                            if 
                                // optimization< checking term first is faster! >
                                checkEqTerm(&iv.read().sentence.term, &goal.term) && // is necessary, else we don't accept detached goals!
                                NarStamp::checkSame(&iv.read().sentence.stamp, &goal.stamp)
                            {
                                exists = true;
                                break; // OPT
                            }
                        }
                        
                        if exists {
                            return;
                        }
                    }
                    //None => {
                    //    println!("INTERNAL ERROR");
                    //}
                }
            },
            None => {} // term doesn't exist
        }
    }

    addEntry2(goalSystem, mem2, Arc::new(RwLock::new(Entry{sentence:Arc::clone(&goal), utility:1.0, evidence:evidence, createTime:t, depth:depth, desirability:1.0, accDesirability:0.0})));
}

/// helper to add goal
// private because it is a helper
fn addEntry2(goalSystem: &Arc<RwLock<GoalSystem>>, mem2: &Mem2, e: Arc<RwLock<Entry>>) {
    
    { // procedural / Q&A bridge
        match &*e.read().sentence.term {
            Term::Seq(seq) if seq.len() > 0 => {
                let seqCondTerm: Term = (*seq[0]).clone();

                let seqCondQaTerm: Term = convProcTermToQaTerm(&seqCondTerm); // translate variables to Question variables

                // add question to Q&A system
                let sharedGuard = mem2.shared.read();
                let questionTasksGuard = sharedGuard.questionTasks.write();
                
                /*
                24.04.2021 commented this because
                           * it's not useful
                           * it leaks memory because it's not kept under AIKR
                           * the functionality to reason about declarative stuff is better solved by a more unified reasoner where we reason on events directly like in ONA

                questionTasksGuard.push(Box::new(Task2 {
                    sentence:newEternalSentenceByTv(&seqCondQaTerm,EnumPunctation::QUESTION,&Tv::Tv{f:1.0,c:0.0},newStamp(&vec![])),
                    handler:Some(Arc::new(RwLock::new(QaProcHandlerImpl{entry:Arc::clone(&e), goalSystem:Arc::clone(goalSystem)}))),
                    bestAnswerExp:0.0, // because has no answer yet
                    prio:1.0,
                }));

                // keep under AIKR (hard)
                questionTasksGuard = questionTasksGuard[..questionTasksGuard.len().min(50)].to_vec();
                */
            }
            _ => {}
        }
    }

    // decide if we add the goal to the active set
    use crate::TermUtils::decodeOp;
    let isOp = decodeOp(&e.read().sentence.term).is_some();
	if isOp {
		if checkSetContains(&goalSystem.read().activeSet.set, &e.read().sentence) {
			return;
		}

		goalSystem.write().activeSet.set.push(Arc::clone(&e));
	}
	else {
        // usual code for adding it to the "big" set


        let chosenDepthIdx:usize = e.read().depth.min(goalSystem.read().nMaxDepth-1) as usize;
        //dbg(&format!("addEntry depth = {} chosenDepthIdx = {}", e.borrow().depth, chosenDepthIdx));
    
        let chosenBatchArc:Arc<RwLock<BatchByDepth>> = Arc::clone(&goalSystem.read().batchesByDepth[chosenDepthIdx]);
        let mut chosenBatch = chosenBatchArc.write();
        
        // now we need to add the entry to the batch
        //
        // a small problem is that the actual entries are stored by term of the sentence
        // * so we have to find the group with the same term if the group exists and add it there or
        // * add it as a new group
        
        { // try to search for group by e.sentence.term
            for iGroup in &mut chosenBatch.groups { // iterate over groups by term
                if checkEqTerm(&iGroup.term, &e.read().sentence.term) { // found entry?
                    let newAcc:f64 = iGroup.entries[iGroup.entries.len()-1].read().accDesirability+iGroup.entries[iGroup.entries.len()-1].read().desirability.max(0.0) as f64; // compute acc utility for this item
                    e.write().accDesirability = newAcc; // update
    
                    iGroup.entries.push(Arc::clone(&e)); // add entry
                    return;
                }
            }
        }
    
        { // case to add it as a new group
            chosenBatch.groups.push(EntryFoldedByTerm {
                    term:(*(e.read().sentence.term)).clone(),
                    entries:vec![Arc::clone(&e)],
                }
            );

            match goalSystem.write().entriesByTerm.get_mut(&e.read().sentence.term) {
                Some(arr) => {
                    
                    let mut exists = false;
                    for iItem in arr.read().iter() {
                        let borrowedItem = iItem.read();
                        if checkEqTerm(&borrowedItem.sentence.term, &e.read().sentence.term) && 
                           NarStamp::checkOverlap(&borrowedItem.sentence.stamp, &e.read().sentence.stamp) {
                            exists = true;
                            break; // OPT
                        }
                    }
                    
                    if !exists {
                        arr.write().push(Arc::clone(&e));
                    }
                    return;
                },
                None => {} // by term doesn't exist - fall through
            }

            // by term doesn't exist
            // * insert new item if we are here
            goalSystem.write().entriesByTerm.insert((*(e.read().sentence.term)).clone(), RwLock::new(vec![Arc::clone(&e)])); // add to memory
        }
	}
}


/// handler to build a procedural pseudo-event for proc / Q&A bridge
pub struct QaProcHandlerImpl {
    pub entry: Arc<RwLock<Entry>>,
    pub goalSystem: Arc<RwLock<GoalSystem>>,
}

impl QHandler for QaProcHandlerImpl {
    fn answer(&mut self, question:&Term, answer:&Sentence) {
        // print question and send answer
        let msg = "TRACE proc/Q&A bridge answer: ".to_owned() + &convTermToStr(&question) + "? " + &convSentenceTermPunctToStr(&answer, true);
        println!("{}", &msg);

        // queue up answer to be processed in NarProc.rs
        self.goalSystem.write().queuedProcQaBridgeAnswers.push(QueuedProcQaBridgeAnswer{
            entry: Arc::clone(&self.entry),
            answer: shallowCopySentence(&answer),
        });

    }
}




fn convProcTermToQaTerm(term: &Term) -> Term {
    match term {
        Term::QVar(name) => Term::QVar("Q_".to_owned()+name),
        Term::DepVar(name)=> Term::QVar("D_".to_owned()+name),
        Term::IndepVar(name)=> Term::QVar("I_".to_owned()+name),


        Term::Stmt(cop, subj, pred) => 
            Term::Stmt(*cop, Box::new(convProcTermToQaTerm(subj)), Box::new(convProcTermToQaTerm(pred)))
        ,
        Term::Name(name) => term.clone(),
        Term::Seq(arr) => {
            let mut arr2 = vec![];
            for i in arr {
                arr2.push(Box::new(convProcTermToQaTerm(i)));
            }
            Term::Seq(arr2)
        }, // sequence
        Term::SetInt(arr) => {
            let mut arr2 = vec![];
            for i in arr {
                arr2.push(Box::new(convProcTermToQaTerm(i)));
            }
            Term::SetInt(arr2)
        },
        Term::SetExt(arr) => {
            let mut arr2 = vec![];
            for i in arr {
                arr2.push(Box::new(convProcTermToQaTerm(i)));
            }
            Term::SetExt(arr2)
        },
        Term::Conj(arr) => {
            let mut arr2 = vec![];
            for i in arr {
                arr2.push(Box::new(convProcTermToQaTerm(i)));
            }
            Term::Conj(arr2)
        },
        Term::Prod(arr) => {
            let mut arr2 = vec![];
            for i in arr {
                arr2.push(Box::new(convProcTermToQaTerm(i)));
            }
            Term::Prod(arr2)
        },
        Term::Img(rel, idx, arr) => {
            let rel2:Term = convProcTermToQaTerm(rel);

            let mut arr2 = vec![];
            for i in arr {
                arr2.push(Box::new(convProcTermToQaTerm(i)));
            }

            Term::Img(Box::new(rel2), *idx, arr2)
        },
        Term::IntInt(arr) => {
            let mut arr2 = vec![];
            for i in arr {
                arr2.push(Box::new(convProcTermToQaTerm(i)));
            }
            Term::IntInt(arr2)
        },
        Term::ExtInt(arr) => {
            let mut arr2 = vec![];
            for i in arr {
                arr2.push(Box::new(convProcTermToQaTerm(i)));
            }
            Term::ExtInt(arr2)
        },
        Term::Par(arr) => {
            let mut arr2 = vec![];
            for i in arr {
                arr2.push(Box::new(convProcTermToQaTerm(i)));
            }
            Term::Par(arr2)
        },
        Term::Neg(t) => Term::Neg(Box::new(convProcTermToQaTerm(t))),
    }
}

// privte because helper
fn checkSetContains(set: &Vec< Arc<RwLock<Entry>> >, s: &Sentence) -> bool {
    // we check for same stamp - ignore it if the goal is exactly the same, because we don't need to store same goals
    for iv in set {
        if 
            // optimization< checking term first is faster! >
            checkEqTerm(&iv.read().sentence.term, &s.term) && // is necessary, else we don't accept detached goals!
            NarStamp::checkSame(&iv.read().sentence.stamp, &s.stamp)
        {
            return true;
        }
    }
    false
}

/// called when it has to stay under AIKR
/// /param t is the procedural reasoner NAR time
pub fn limitMemory(goalSystem: &Arc<RwLock<GoalSystem>>, mem2: &Mem2, t: i64) {
    let mut arr:Vec<Arc<RwLock<Entry>>> = retEntries(&goalSystem.read()); // working array with all entries
    
    for iv in &goalSystem.read().activeSet.set {
        arr.push(Arc::clone(&iv));
    }

    dbg(&format!("nEntries={}", arr.len()));

    // * recalc utility
    for iv in &arr {
        let mut iv2 = iv.write();
        // consider age
        let age: i64 = t-iv2.createTime;
        let decay = ((age as f64)*-0.01).exp(); // compute decay by age

        iv2.utility = 
            Tv::calcExp(&retTv(&iv2.sentence).unwrap())*
            (iv2.desirability as f64).abs()* // times the desirability because not so desired goals should get forgotten
            decay;
    }

    // * sort by utility
    arr.sort_by(|a, b| b.read().utility.partial_cmp(&a.read().utility).unwrap());

    // * limit (AIKR)
    while arr.len() as i64 > goalSystem.read().nMaxEntries {
        arr.remove(goalSystem.read().nMaxEntries as usize);
    }

    goalSystem.write().batchesByDepth = vec![]; // flush
    // rebuild
    let nMaxDepth = goalSystem.read().nMaxDepth;
    for iDepth in 0..nMaxDepth {
        goalSystem.write().batchesByDepth.push(Arc::new(RwLock::new(BatchByDepth{groups: vec![], depth:iDepth,})));
    }

    goalSystem.write().entriesByTerm = HashMap::new(); // flush, need to do this before calling addEntry2()
    goalSystem.write().activeSet.set = vec![];

    // fill
    for iVal in arr {
        addEntry2(goalSystem, mem2, iVal);
    }
}

/// sample a goal from the goal table of the goal system
/// returns (sentence, depth)
pub fn sample(goalSystem: &GoalSystem, rng: &mut rand::rngs::ThreadRng) -> Option<(Arc<Sentence>, i64)> {
    // select batch (or return)
    let selBatchRef = {

        let sumPriorities:f64 = goalSystem.batchesByDepth.iter().map(|iv| 
            if iv.read().groups.len() > 0 {1.0} else {0.0} // only consider batches which have groups
        ).sum();
    
        let selPriority:f64 = rng.gen_range(0.0..1.0) * sumPriorities;

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
        let mut selBatch:Arc<RwLock<BatchByDepth>> = Arc::clone(&goalSystem.batchesByDepth[0]); // default, should never be used
        let mut sum:f64 = 0.0;
        for iv in &goalSystem.batchesByDepth {
            if sum >= selPriority {
                break;
            }
            if iv.read().groups.len() > 0 {
                sum+=1.0;
                selBatch = Arc::clone(iv);
            };
        }

        //dbg(&format!("sel byDepthIdx={}", idx));

        selBatch
    };
    let selBatch = selBatchRef.read();


    
    
    
    if selBatch.groups.len() == 0 {
        return None;
    }
    
    let entriesOfSelBatch: &Vec<EntryFoldedByTerm> = &selBatch.groups;
    
    let sumPriorities:f64 = entriesOfSelBatch.iter()
        .map(|iEntriesByTerm| { // map over entries-by-term
            let lastEntry = &iEntriesByTerm.entries[iEntriesByTerm.entries.len()-1].read();
            let des2:f64 = lastEntry.accDesirability+lastEntry.desirability as f64;
            des2 // return inner sum
        }).sum();
    
    let selPriority:f64 = rng.gen_range(0.0..1.0) * sumPriorities;

    // select
    let mut sum:f64 = 0.0;
    let mut selEntry = None;
    for iv in &selBatch.groups {
        assert!(sum <= sumPriorities); // priorities are summed in the wrong way in this loop if this invariant is violated
        
        let lastEntry = &iv.entries[iv.entries.len()-1].read();
        let des2:f64 = lastEntry.accDesirability+lastEntry.desirability as f64;

        if des2+sum > selPriority { // is the range of the selection inside this "group"?

            use crate::BinSearch::binSearch;
            let selEntry3: Arc<RwLock<Entry>> = binSearch(&iv.entries, selPriority);
            let selEntry2 = selEntry3.read();
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
pub fn query_by_antecedent(queryTerm: &Term, procMem:&NarMem::Mem) -> Vec<Arc<RwLock<Sentence>>> {
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
pub fn query_by_consequence(queryTerm: &Term, procMem:&NarMem::Mem) -> Vec<Arc<RwLock<Sentence>>> {
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

/// returns sub sequence as term if it has a single sub-term or as seq if it has multiple
//private because helper
fn retSubSeqAsSeqOrTerm(subseq:&[Box<Term>])->Term {
    return if subseq.len() == 1 {
        (*subseq[0]).clone()
    }
    else {
        Term::Seq(subseq.to_vec())
    }
}

/// select highest ranked goal for state
/// returns entity and unified result
pub fn selHighestExpGoalByState(goalSystem: &GoalSystem, state:&Term) -> (f64, Option<(Arc<RwLock<Entry>>, Term)>) {
    let mut res:(f64, Option<(Arc<RwLock<Entry>>, Term)>) = (0.0, None);

    for iv in &retEntries(goalSystem) {
        match &(*(iv.read().sentence).term) {
            
            Term::Seq(seq) if seq.len() >= 1 => {
                // try to unify first part of sequence with observed state
                // we can only consider if it matches!
                let asgnmts:Option<Vec<NarUnify::Asgnment>> = NarUnify::unify(&retSubSeqAsSeqOrTerm(&seq[..seq.len()-1]), &state);

                if asgnmts.is_some() { // does first event of seq match to state with unification?, must unify!
                    let exp = Tv::calcExp(&retTv(&iv.read().sentence).unwrap());
                    let (resExp, _) = res;
                    if exp > resExp {
                        let unifiedTerm: Term = NarUnify::unifySubst(&*(iv.read().sentence).term, &asgnmts.unwrap()); // unify because we need term with less or no variables!
                        res = (exp, Some((Arc::clone(&iv), unifiedTerm)));
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
    goal: Arc<Sentence>,
    evidence: Option<Arc<RwLock<Sentence>>>,
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
fn deriveGoalsHelper(sampledGoal: &Sentence, sampledDepth:i64, strategy:EnumGoalDerivationStrategy, procMem:&NarMem::Mem, rng: &mut rand::rngs::ThreadRng)->Vec<H> {
    let mut concls:Vec<H> = Vec::new(); // conclusions

    match NarInfProcedural::infGoalDetach(&sampledGoal) {
        Some(concl) => {
            concls.push(H{goal:Arc::new(concl), evidence:None, depth:sampledDepth+1});
        },
        _ => {
            // * try to find candidates for inference
            let evidenceCandidates: Vec<Arc<RwLock<Sentence>>> = query_by_consequence(&sampledGoal.term, procMem);

            //dbg(&format!("sampleAndInference() found belief candidates #={}", evidenceCandidates.len()));

            // * try to do inference
            match strategy {
                ALL_BELIEFS => {
                    for iBelief in &evidenceCandidates {
                        let conclOpt:Option<Sentence> = NarInfProcedural::infGoalBelief(&sampledGoal, &iBelief.read());
                        if conclOpt.is_some() {
                            concls.push(H{goal:Arc::new(conclOpt.unwrap()), evidence:Some(Arc::clone(iBelief)), depth:sampledDepth+1});
                        }
                    }
                },
                SAMPLE_2 => {
                    for _it in 0..2 {
                        if evidenceCandidates.len() > 0 {
                            let idx = rng.gen_range(0..evidenceCandidates.len());
                            let conclOpt:Option<Sentence> = NarInfProcedural::infGoalBelief(&sampledGoal, &evidenceCandidates[idx].read());
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
pub fn sampleAndInference(goalSystem: &Arc<RwLock<GoalSystem>>, mem2: &Mem2, t:i64, procMem:&NarMem::Mem, rng: &mut rand::rngs::ThreadRng) {
    // * sample goal from set of goals
    let sampledGoalOpt: Option<(Arc<Sentence>, i64)> = sample(&goalSystem.write(), rng);

    if !sampledGoalOpt.is_some() {
        return; // no goal was sampled -> give up
    }
    let (sampledGoal, sampledDepth): (Arc<Sentence>, i64) = sampledGoalOpt.unwrap();

    let mut concls:Vec<(Arc<Sentence>, Option<Arc<RwLock<Sentence>>>, i64)> = Vec::new(); // conclusions are tuple (goal, evidence, depth)
    
    //dbg(&format!("sampleAndInference() sampled goal = {}", &NarSentence::convSentenceTermPunctToStr(&sampledGoal, true)));

    {
        // we need a structure to store goals in the working set with some meta-information
        struct WsEntry { // working set entry
            pub goal: Arc<Sentence>, // goal
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
        for _iRound in 0..goalSystem.read().cfg__subworkingCycle_rounds {
            if workingSet.len() == 0 {
                break; // no more goals -> we are done
            }

            // sample goal from workingSet

            let sampledWsEntry: Rc<WsEntry> = {
                let selIdx = rng.gen_range(0..workingSet.len());
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
        addEntry(goalSystem, mem2, t, Arc::clone(iGoal), iEvidence2, *iDepth);
    }
}

/// called from outside when event happened
pub fn event_occurred(goalSystem: &mut GoalSystem, eventTerm:&Term) {
    for iEntityRc in retEntries(goalSystem) {
        let mut iEntity = iEntityRc.write();
        if goalSystem.cfg__enGoalSatisfaction && // do we want to satisfy goals?
           checkEqTerm(&iEntity.sentence.term, eventTerm) // terms must of course to match up that a event can satify the goal
        {
            iEntity.desirability = 0.0; // set desirability to 0.0 because it happened
        }
    }

    for iEntityRc in &goalSystem.activeSet.set {
        let mut iEntity = iEntityRc.write();
        if goalSystem.cfg__enGoalSatisfaction && // do we want to satisfy goals?
           checkEqTerm(&iEntity.sentence.term, eventTerm) // terms must of course to match up that a event can satify the goal
        {
            iEntity.desirability = 0.0; // set desirability to 0.0 because it happened
        }
    }
}

/// returns the goal which matches with the term
pub fn query(goalSystem: &GoalSystem, eventTerm:&Term) -> Option<Arc<RwLock<Entry>>> {
    // TODO< select goal with highest exp! >

    for iEntityRc in retEntries(goalSystem) {
        let iEntity = iEntityRc.read();
        if checkEqTerm(&iEntity.sentence.term, eventTerm) { // terms must of course to match up
            return Some(Arc::clone(&iEntityRc));
        }
    }

    for iEntityRc in &goalSystem.activeSet.set {
        let iEntity = iEntityRc.read();
        if checkEqTerm(&iEntity.sentence.term, eventTerm) { // terms must of course to match up
            return Some(Arc::clone(&iEntityRc));
        }
    }

    None
}



/// helper for debugging: return all goals as text
pub fn dbgRetGoalsAsText(goalSystem: &GoalSystem) -> String {
    let mut res:String = String::new();

    for iv in &retEntries(goalSystem) {
        let sentenceAsStr = NarSentence::convSentenceTermPunctToStr(&(*iv).read().sentence, true);
        res += &format!("{}   util={} depth={}\n", &sentenceAsStr, &(*iv).read().utility, iv.read().depth);
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
