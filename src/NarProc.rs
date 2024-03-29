use rand::Rng;
use std::time::{Instant};
use std::rc::Rc;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;
use std::thread::JoinHandle;
use parking_lot::RwLock;

use crate::NarStamp::*;
use crate::NarSentence::*;
use crate::Term::*;
use crate::TermUtils::*;
use crate::TermApi::*;
use crate::NarGoalSystem;
use crate::NarMem;
use crate::Tv;
use crate::NarInfProcedural;
use crate::Utils::{enforce};
use crate::NarWorkingCycle::Mem2;

/// contains all necessary variables of a procedural NAR
pub struct ProcNar {
    /// base for the exponential intervals
    pub cfgIntervalExpBase:f64,
    /// maximal interval time
    pub cfgIntervalMax:i64,

    /// perception window for current events
    pub cfgPerceptWindow:i64,
    /// decision threshold for decision making
    pub cfgDescnThreshold:f64,

    /// maximal number of evidence
    pub cfgNMaxEvidence:i64,
    
    /// enable motor babbling?
    pub cfgEnBabbling:bool,

    /// how many ops can a impl seq maximally contain, values above 1 are considered as EXPERIMENTAL
    pub cfg__nOpsMax:i64,
    /// how high is the proability to select multiple ops for seq impl candidates
    pub cfg__multiOpProbability:f64,

    // how many pieces of evidence are assigned to an observation, usually low numbers, high numbers make the observation more axiomatic
    //pub cfg__eviCnt:i64,

    /// is anticipation enabled? disable for specialized functionality
    pub cfg__enAnticipation: bool,


    /// how many concepts does it store at max (soft limit)
    pub cfg__nConcepts:i64,

    /// how many beliefs are stored in a concept
    pub cfg__nConceptBeliefs:usize,

    // how many samples are done for goal derivation (per timed cycle)
    //pub cfg__nGoalDeriverSamples:i64,

    // resource distribution (AIKR)
    /// expected time for one derivation step, is the smallest time unit from the outside
    pub cfg__resourceExpectedTime: f32,
    /// priority of (temporal) goal derivation
    pub cfg__resourceGoalDeriverPrio: f32,
    /// how ofter should event-FIFO get sampled for perception in cycle?
    pub cfg__perceptionSamplesPerStep:i64,


    /// how verbose is the reasoner, mainly used for debugging
    pub cfgVerbosity:i64,

    /// memory with the (procedural) evidence
    pub evidenceMem: Arc<RwLock<NarMem::Mem>>,

    /// trace of some past events under AIKR
    pub trace: Box<dyn Trace>,
    
    /// all anticipated events "in flight"
    pub anticipatedEvents: Vec<AnticipationEvent>,

    /// all registered ops
    pub ops: Vec<Rc<Box<dyn Op>>>,

    /// NAR time
    pub t:i64,


    pub rng: rand::rngs::ThreadRng,

    /// table with exponential intervals
    pub expIntervalsTable:Vec<i64>,

    /// "goal system" - manages goals of the procedural reasoner
    pub goalSystem: Arc<RwLock<NarGoalSystem::GoalSystem>>,



    // internals which are still public

    /// array of workers
    pub storeWorkers: Vec<JoinHandle<()>>,
    /// sender to worker
    pub storeWorkersTx: Vec<SyncSender<(Sentence,i64)>>,
}

/// trait for trace implementations
/// 
/// allows to select the implementation of the trace at runtime and overwrite it with custom one
pub trait Trace {
    /// return name of implementation of trait
    fn retImplName(&self) -> String;

    // return last events which happened in parallel
    //fn retLastEvents(&self) -> Vec<SimpleSentence>;

    fn step(&self, nar:&ProcNar);

    fn limit(&self);

    /// adds a event which happened at the current moment
    fn event_happened(&self, event: Rc<SimpleSentence>);

    /// flushes all events in the datastructure
    fn force_flush(&self);

    /// return concurrent last events
    ///
    /// result can be None if no event happened yet
    fn ret_last_events(&self) -> Option<Vec<Rc<SimpleSentence>>>;

    /// returns a view of all events
    ///
    /// can be a expensive operation
    fn ret_view(&self) -> Vec<Rc<SimpleSentence>>;
}

pub struct TraceDefault {
    /// trace of some past events under AIKR
    pub trace: Arc<RwLock<Vec<Rc<SimpleSentence>>>>,

    pub rng: Arc<RwLock<rand::rngs::ThreadRng>>,
}

/// standard implementation of "Trace"
impl Trace for TraceDefault {
    fn retImplName(&self) -> String {
        "default".to_string()
    }

    fn step(&self, nar:&ProcNar) {
        {
            // neutralize goals which are fullfilled by current event
            let mut traceGuard = self.trace.write();
            if traceGuard.len() > 0 {
                let last_events: Vec<Rc<SimpleSentence>> = {
                    let mut last_events: Vec<Rc<SimpleSentence>> = vec![]; // concurrently happened last events
                    last_events.push(Rc::clone(&traceGuard[traceGuard.len()-1]));
                    last_events
                };
                for i_last_event in last_events {
                    NarGoalSystem::event_occurred(&mut nar.goalSystem.write(), &i_last_event.name);
                }
            }
        }

        let traceGuard = self.trace.read();
        if traceGuard.len() >= 3 { // add evidence
            for _sampleIt in 0..nar.cfg__perceptionSamplesPerStep {
                // filter middle by ops and select random first event before that!
                let idxsOfOps:Vec<i64> = calcIdxsOfOps(&nar, &traceGuard);
                if idxsOfOps.len() > 0 { // there must be at least one op to sample


                    let selIdxOfOps: Vec<usize> = {// indices of selected ops
                        
                        let nSelOps = // how many ops do we try to select
                            if nar.cfg__nOpsMax == 1 {1}
                            else if self.rng.write().gen_range(0.0..1.0) < nar.cfg__multiOpProbability {
                                idxsOfOps.len().min(nar.cfg__nOpsMax as usize)
                            }
                            else {1};

                        // select ops
                        let mut selIdxOfOps = vec![];
                        loop {
                            let idx1Idx = self.rng.write().gen_range(0..idxsOfOps.len());
                            let selIdx = idxsOfOps[idx1Idx] as usize;
                            if !selIdxOfOps.contains(&selIdx) {
                                selIdxOfOps.push(selIdx);
                            }

                            if selIdxOfOps.len() == nSelOps { // do we have selected the ops?
                                break;
                            }
                        }

                        selIdxOfOps
                    };
                    
                    if selIdxOfOps.len() > 0 && *selIdxOfOps.iter().min().unwrap() > 0 { // is there a valid index for a op which is not the last item in the trace?
                        
                        let selTraceItems: Vec<Rc<SimpleSentence>> = {
                            let idxFirst = self.rng.write().gen_range(0..*selIdxOfOps.iter().min().unwrap()); // select index of event before first selected op
                            let mut idxLast = traceGuard.len()-1; // last event is last

                            // TODO< rewrite to logic which scans for the first op between idxLast and selIdxOfOps, select random event as idxLast between these!
                            
                            // check if we can select previous event
                            {
                                let sel = traceGuard[traceGuard.len()-1-1].clone();
                                let rng0:i64 = self.rng.write().gen_range(0..2);
                                if rng0 == 1 && traceGuard.len()-1-1 > *selIdxOfOps.iter().max().unwrap() && !checkIsCallableOp(&nar, &sel.name) {
                                    idxLast = traceGuard.len()-1-1;
                                }
                            }
                            let idxs = { // compose indices of selected events
                                let mut idxs = vec![idxFirst];
                                idxs.extend(selIdxOfOps);
                                idxs.push(idxLast);
                                idxs.sort();
                                idxs
                            };

                            idxs.iter().map(|idx| Rc::clone(&traceGuard[*idx])).collect() // select trace items
                        };
                        
                        let termsOfSelVecItems:Vec<Term> = selTraceItems.iter().map(|iv| iv.name.clone()).collect();
                        let implSeqOpt: Option<Term> = try_build_implSeq(nar, &termsOfSelVecItems); // try to build impl seq from selected trace items
                        if implSeqOpt.is_some() { // was building of impl seq successful?
                            let candidateTerm:Term = implSeqOpt.unwrap().clone();
                            
                            if nar.cfgVerbosity > 0 {println!("perceive {}", convTermToStr(&candidateTerm));};
                            
                            // compute time between last event and the element before the last event
                            let dt:i64 = selTraceItems[selTraceItems.len()-1].occT - selTraceItems[selTraceItems.len()-2].occT;
                            // compute exponential delta time
                            let expDt:i64 = findMinTableIdx(dt, &nar.expIntervalsTable);
                            


                            
                            let stamp:Stamp = {
                                // compute merged stamp from evidence of all events
                                let stampEvi = selTraceItems.iter().map(|iv| iv.evi).collect();
                                newStamp(&stampEvi)
                            };

                            let mut cfg__eviCnt = 3; // how many pieces of evidence are added?


                            { // try to lookup eviCnt of op by op-name
                                let implSeqOpTerm: Term = retImplSeqOp(&candidateTerm);

                                let (opArgs, opName) = decodeOp(&implSeqOpTerm).unwrap();

                                match decodeOp(&implSeqOpTerm) {
                                    Some((opArgs, opName)) => {

                                        // search for action with name
                                        cfg__eviCnt = match ret_op_by_name(nar, &opName) {
                                            Some(op) => {op.ret_evi_cnt()},
                                            None => {cfg__eviCnt}
                                        }
                                    },
                                    None => {}
                                }
                            }

                            let evidenceSentence: Sentence = Sentence {
                                punct:EnumPunctation::JUGEMENT,
                                t:None,
                                stamp:stamp,
                                expDt:Some(expDt),
                                term:Arc::new(candidateTerm.clone()), // ex: (e0 &/ e1) =/> e2
                                evi:Some(Evidence::CNT{pos:cfg__eviCnt,cnt:cfg__eviCnt}),
                                usage:Arc::new(RwLock::new(Usage{lastUsed: 0, useCount: 0})),
                            };
                            
                            let workerIdx = self.rng.write().gen_range(0..nar.storeWorkersTx.len());
                            nar.storeWorkersTx[workerIdx].send((evidenceSentence, nar.t)).unwrap(); // defer actual storage to worker
                        }
                    }
                }
            }
        }
    }

    fn limit(&self) {
        let mut writeGuard = self.trace.write();
        // limit trace (AIKR)
        if writeGuard.len() > 20 {
            *writeGuard = (&writeGuard[(*writeGuard).len()-20..]).to_vec();
        }
    }

    
    fn event_happened(&self, event: Rc<SimpleSentence>) {
        self.trace.write().push(Rc::clone(&event));
    }
    
    fn force_flush(&self) {
        self.trace.write().clear();
    }

    fn ret_last_events(&self) -> Option<Vec<Rc<SimpleSentence>>> {
        let readGuard = self.trace.read();
        match readGuard.len() {
            0 => {None}
            _ => {
                // TODO< scan for events with the same occurence time from the back!? >

                let last_event: Rc<SimpleSentence> = Rc::clone(&readGuard[readGuard.len()-1]);
                Some(vec![last_event])
            }
        }
    }

    fn ret_view(&self) -> Vec<Rc<SimpleSentence>> {
        let mut res = vec![];
        let read_guard = self.trace.read();
        for i_event in &*read_guard {
            res.push(Rc::clone(&i_event));
        }
        res
    }
}

/// init and set to default values
pub fn narInit() -> ProcNar {
    let mut nar = ProcNar {
        cfgIntervalExpBase: 1.3,
        cfgIntervalMax: 20,
        cfgPerceptWindow: 2,
        cfgDescnThreshold: 0.58,
        cfgNMaxEvidence: 5000,
        cfg__perceptionSamplesPerStep:4,
        cfgEnBabbling: true,
        cfg__nOpsMax: 1,
        cfg__multiOpProbability: 0.2,
        //cfg__eviCnt: 3, // non-axiomatic
        cfg__enAnticipation: true, // by default

        cfg__nConcepts: 1000,
        cfg__nConceptBeliefs: 100,

        // compute resources
        //cfg__nGoalDeriverSamples: 1, // 3 is enough for pong
        cfg__resourceExpectedTime: 0.08, // expected time for one inference step
        cfg__resourceGoalDeriverPrio: 1.05, // priority of (temporal) goal derivation
        
        cfgVerbosity: 0, // be silent

        //evidence: Vec::new(),
        evidenceMem: Arc::new(RwLock::new(NarMem::make())),

        //trace: Vec::new(),
        trace: Box::new(TraceDefault{trace: Arc::new(RwLock::new(vec![])),rng: Arc::new(RwLock::new(rand::thread_rng())),}),
        anticipatedEvents: Vec::new(),
        ops: Vec::new(),
        t: 0,

        rng: rand::thread_rng(),

        expIntervalsTable: Vec::new(),

        goalSystem: Arc::new(RwLock::new(NarGoalSystem::makeGoalSystem(20, 8))),

        storeWorkers: vec![],
        storeWorkersTx: vec![],
    };


    // create worker which stores evidence with revision
    for _iWorker in 0..1 {
        let (tx, rx) = sync_channel(4); // create channel with fixed size, reason is that we want to limit backlog!
        nar.storeWorkersTx.push(tx);

        let evidenceMem = Arc::clone(&nar.evidenceMem);
        let cfg__nConceptBeliefs = nar.cfg__nConceptBeliefs;
        nar.storeWorkers.push(thread::spawn(move|| {
            let mut rng = rand::thread_rng();
            
            loop {
                let msgRes = rx.recv();
                if !msgRes.is_ok() {
                    break; // other side has hung up, terminate this worker
                }
                // currentTime: is the time when the sentence was added
                let (evidenceSentence, currentTime) = msgRes.unwrap(); // receive message
                //println!("[STORAGE WORKER] received MSG!");

                /////////
                // STORE
                /////////
                let addEvidenceFlag: Arc<AtomicBool> = Arc::new(AtomicBool::new(true)); // do we need to add new evidence?
                
                { // scope for guard
                    let evidenceMemGuard = evidenceMem.read();
                    for iEEArc in &NarMem::ret_beliefs_by_terms_nonunique(&evidenceMemGuard, &[retSeqCond(&evidenceSentence.term).clone(), retPred(&evidenceSentence.term).clone()]) { // iterate over evidence where seqCond and/or pred appear
                        let mut iEE = iEEArc.write();
                        
                        if !checkOverlap(&iEE.stamp, &evidenceSentence.stamp) { // evidence must no overlap!
                            if
                                iEE.expDt.unwrap() >= evidenceSentence.expDt.unwrap() && // check for greater because we want to count evidence for longer intervals too, because longer ones are "included"
                                
                                // does impl seq match?
                                checkEqTerm(&iEE.term, &evidenceSentence.term)
                            {
                                iEE.stamp = merge(&iEE.stamp, &evidenceSentence.stamp);

                                let (otherCntPos, otherCnt) = match evidenceSentence.evi.as_ref().unwrap() {
                                    Evidence::CNT{pos,cnt} => {
                                        (pos, cnt)
                                    },
                                    _ => {panic!("expected CNT!");}
                                };

                                match iEE.evi.as_ref().unwrap() {
                                    Evidence::CNT{pos,cnt} => {
                                        iEE.evi = Some(Evidence::CNT{pos:pos+otherCntPos,cnt:cnt+otherCnt}); // bump positive counter
                                    },
                                    _ => {panic!("expected CNT!");}
                                }
                                
                                if false {println!("dbg - REV")};
                                
                                addEvidenceFlag.store(false, Ordering::Relaxed); // because we revised
                            }                                
                        }
                    }    
                }
                
                if addEvidenceFlag.load(Ordering::Relaxed) {
                    // add evidence
                    mem_add_evidence(Arc::clone(&evidenceMem), &evidenceSentence, cfg__nConceptBeliefs, currentTime, &mut rng);
                }
            }
        }));
    }



    // build table with exponential intervals
    {
        let mut lastExpInterval:i64 = 0;
        let mut i:i64 = 0;
        loop {
            let thisInterval = nar.cfgIntervalExpBase.powf(i as f64) as i64;
            if thisInterval > nar.cfgIntervalMax {
                break; // we collected all intervals
            }
            if thisInterval > lastExpInterval {
                lastExpInterval = thisInterval;
                nar.expIntervalsTable.push(thisInterval); // store
            }
            
            i+=1;
        }
    }
    
    nar
}

// INGATE< gate >
/// add procedural evidence to memory
pub fn mem_add_evidence(evidenceMem: Arc<RwLock<NarMem::Mem>>, evidenceSentence: &Sentence, nBeliefs:usize, currentTime: i64,  rng: &mut rand::rngs::ThreadRng) {
    mem_add_evidence_internal(Arc::clone(&evidenceMem), evidenceSentence, nBeliefs, currentTime);

    // TODO< how to manage evidence of vars? and revise? >
    // intro vars and add to memory
    let sentences_with_vars: Vec<Sentence> = NarInfProcedural::matchandintro_var1_sentence(evidenceSentence, rng);
    for i_sentence in &sentences_with_vars {
        mem_add_evidence_internal(Arc::clone(&evidenceMem), i_sentence, nBeliefs, currentTime);
    }
}

/// internal helper to add procedural evidence to memory
fn mem_add_evidence_internal(evidenceMem: Arc<RwLock<NarMem::Mem>>, evidenceSentence: &Sentence, nBeliefs:usize, currentTime: i64) {
    // enumerate subterms to decide concept names where we store the belief
    let subterms = {
        let mut subterms = vec![];
        // enumerate subterms from first seq element of predimpl and predicate of predimpl
        subterms.extend(retSubterms(&retSeqCond(&evidenceSentence.term)));
        subterms.extend(retSubterms(&retPred(&evidenceSentence.term)));
        subterms
    };

    NarMem::storeInConcepts2(&mut evidenceMem.write(), &evidenceSentence, &subterms, nBeliefs, currentTime);
}

/// returns all evidence, can be overlapping!
pub fn mem_ret_evidence_all_nonunique(procNar:&ProcNar) -> Vec<Arc<RwLock<Sentence>>> {
    let mut res = vec![];
    let evidenceMemGuard = procNar.evidenceMem.read();
    for (ikey, _iConcept) in &evidenceMemGuard.concepts {
        let beliefsOfConceptOpt = NarMem::ret_beliefs_of_concept(&evidenceMemGuard, &ikey);

        match beliefsOfConceptOpt {
            Some(beliefsOfConcept) => {
                // add to result
                for iBelief in beliefsOfConcept {
                    res.push(Arc::clone(iBelief));
                }
            },
            None => {}
        }
    }
    res
}

/// helper which tries to build a impl seq out of events
/// ex: [a, b, c]  returns (a, b) =/> c
pub fn try_build_implSeq(nar:&ProcNar, events:&[Term]) -> Option<Term> {
    let middle = &events[1..events.len()-1];
    assert!(middle.len() >= 1, ""); // check if the code to select middle is right
    if
        middle.iter().map(|iItem| checkIsCallableOp(&nar, &iItem)).all(|x| x == true) && // middle must be callable ops
        // first and last must not be op
        !checkIsCallableOp(&nar, &events[0])  &&
        !checkIsCallableOp(&nar, &events[events.len()-1]) && 
        !checkEqTerm(&events[0], &events[events.len()-1]) // first and last event must not be the same
    {
        let mut seq2:Vec<Term> = vec![events[0].clone()];
        for i in middle {
            seq2.push(i.clone());
        }

        let implSeq = s(Copula::PREDIMPL, &seq(&seq2), &events[events.len()-1]);
        return Some(implSeq);
    };
    None
}

/// does first work of one reasoner step
pub fn narStep0(nar:&mut ProcNar) {
    if nar.cfgVerbosity > 0 {println!("ae# = {}", nar.anticipatedEvents.len());}; // debug number of anticipated events
    
    // remove confirmed anticipations
    {
        let trace_view = nar.trace.ret_view();
        for perceptIdx in 0..nar.cfgPerceptWindow as usize {
            if trace_view.len() > perceptIdx {
                let curEvent:&Term = &trace_view[trace_view.len()-1-perceptIdx].name;
                
                let mut newanticipatedEvents = Vec::new();
                for iDeadline in &nar.anticipatedEvents {
                    let evi = iDeadline.evi.read();
                    if !checkEqTerm( &retPred(& evi.term), &curEvent) { // is predicted event not current event?
                        newanticipatedEvents.push(iDeadline.clone());
                    }
                }
                nar.anticipatedEvents = newanticipatedEvents;
            }
        }
    }
    

    
    { // neg confirm for anticipated events
        {
            for iDeadlineViolated in nar.anticipatedEvents.iter().filter(|v| v.deadline <= nar.t) {
                let mut mutEviGuard = iDeadlineViolated.evi.write();
                
                // KEYWORD< neg-confirm >
                if false {match mutEviGuard.evi.as_ref().unwrap() {
                    Evidence::CNT{pos,cnt} => {
                        println!("TRACE anticipation: before neg conf   evidence: +/n {}/{}", pos, cnt); // evidence before neg-confirm
                    },
                    _ => {panic!("expected CNT!");}
                }}

                match mutEviGuard.evi.as_ref().unwrap() {
                    Evidence::CNT{pos,cnt} => {
                        mutEviGuard.evi = Some(Evidence::CNT{pos:*pos,cnt:cnt+1}); // add negative evidence
                    },
                    _ => {panic!("expected CNT!");}
                }

                if false {match mutEviGuard.evi.as_ref().unwrap() {
                    Evidence::CNT{pos,cnt} => {
                        println!("TRACE anticipation: after neg conf   evidence: +/n {}/{}", pos, cnt); // evidence before neg-confirm
                    },
                    _ => {panic!("expected CNT!");}
                }}
            }
        }
        
        nar.anticipatedEvents = nar.anticipatedEvents.iter().filter(|&iDeadline| iDeadline.deadline > nar.t).map(|v| v.clone()).collect();
    }

    // do stuff of the trace
    nar.trace.step(nar);
}

/// does second part of reasoner step
///
/// usually after events were put into the FIFO
pub fn narStep1(nar:&mut ProcNar, declMem:&Option<Arc<RwLock<Mem2>>>) {    
    let timeStart = Instant::now();
    
    let mut pickedAction:Option<Term> = None; // complete term of op
    {
        struct BestEntry {
            unifiedSeq: Term, // unified sequence used for decision making
            exp: f64, // expectation
            evidence: Option<Arc<RwLock<Sentence>>>, // evidence, used for anticipation
        }

        // helper to clone evidence
        fn cloneEvidence(v: &Option<Arc<RwLock<Sentence>>>) -> Option<Arc<RwLock<Sentence>>> {
            match v {
                Some(vv) => {Some(Arc::clone(vv))},
                None => None
            }
        }

        let mut bestEntry2: Option<BestEntry> = None;

        // * search if we can satisfy goal
        {
            let trace_view = nar.trace.ret_view();
            for perceptIdx in 0..nar.cfgPerceptWindow as usize {
                if trace_view.len() > perceptIdx {
    
                    let checkedState:Term = trace_view[trace_view.len()-1-perceptIdx].name.clone();
    
                    // check if current state "leads" to action
                    // tuple is (exp, entity)
                    let thisEntry: (f64, Option<(Arc<RwLock<NarGoalSystem::Entry>>, Term)>) = NarGoalSystem::selHighestExpGoalByState(&nar.goalSystem.read(), &checkedState);
    
                    match thisEntry.1 {
                        Some(e) => { // was a candidate found?
    
    
                            match bestEntry2 { // is there best entry?
                                Some(ref bestEntry4) => {
                                    if thisEntry.0 > bestEntry4.exp {
                                        bestEntry2 = Some(BestEntry{
                                            unifiedSeq: e.1.clone(), // pull out unified term
                                            exp:thisEntry.0,
                                            evidence:cloneEvidence(&e.0.read().evidence)});
                                    }
                                },
                                None => {
                                    bestEntry2 = Some(BestEntry{
                                        unifiedSeq: e.1.clone(), // pull out unified term
                                        exp:thisEntry.0,
                                        evidence:cloneEvidence(&e.0.read().evidence)});
                                }
                            }
                        },
                        None => {}
                    }
                }
            }
        }


        { // procedural / Q&A bridge
            // see if the queued answers are good enough to cause an reaction/execution.
            // we pretend that the answer did occur as an event

            let mut writeGuard = nar.goalSystem.write();
            for iQueuedAnswer in &writeGuard.queuedProcQaBridgeAnswers {

                let exp:f64 = Tv::calcExp(&retTv(&iQueuedAnswer.answer).unwrap()); // TODO< calc exp() correctly! >

                let nonunifiedSeq: Term = (*iQueuedAnswer.entry.read().sentence.term).clone();
                let unifiedSeq: Term = nonunifiedSeq.clone(); // HACK< we just take the nonunified term > TODO< unify! >

                match bestEntry2 { // is there best entry?
                    Some(ref bestEntry4) => {
                        if exp > bestEntry4.exp {
                            bestEntry2 = Some(BestEntry{
                                unifiedSeq: unifiedSeq.clone(), // pull out unified term
                                exp:exp,
                                evidence:cloneEvidence(&iQueuedAnswer.entry.read().evidence)});
                        }
                    },
                    None => {
                        bestEntry2 = Some(BestEntry{
                            unifiedSeq: unifiedSeq.clone(), // pull out unified term
                            exp:exp,
                            evidence:cloneEvidence(&iQueuedAnswer.entry.read().evidence)});
                    }
                }
            } // loop
            writeGuard.queuedProcQaBridgeAnswers.clear();
        }

        // MECHANISM< forward depth first prediction
        // tries to plan K steps ahead until it hopefully "hits" a goal
        //
        // motivation: agent can do actions even when the consequences aren't full sure
        // >
        {
            match nar.trace.ret_last_events() {
                Some(concurrent_events) => {
                    for i_concurrent_event in &concurrent_events {

                        // TODO< don't plan forward if term is a op! >

                        // sel last event
                        let checkedState:Term = i_concurrent_event.name.clone();
                        //let mut carriedGoalTerm:Term = checkedState.clone();

                        // build predicated event with term and TV
                        let mut predictedTerm: Term = checkedState.clone();
                        //let mut predictedTv: Tv::Tv = Tv::Tv{f:1.0,c:0.99999}; // axiomatic TV

                        //let mut firstExecEvidence: Option<Arc<RwLock<Sentence>>> = None; // used to remember evidence of first impl seq

                        let mut execEvidence: Vec<Arc<RwLock<Sentence>>> = vec![]; // used to remember chain till hit goal, necessary for correct TV/desire computation

                        let cfg__forwardPredication_steps:i64 = 3; // how many steps are maximaly utilized for forward planning with prediction?, set to 0 to disable feature, IS EXPERIMENTAL
                        for iStep in 0..cfg__forwardPredication_steps {
                            
                            // check if prediction did hit a goal
                            if iStep > 0 {
                                let hitGoalEntryOpt: Option<Arc<RwLock<NarGoalSystem::Entry>>> = NarGoalSystem::query(&nar.goalSystem.read(), &predictedTerm);
                                match hitGoalEntryOpt {
                                    Some(hitGoalEntry) => {
                                        if execEvidence.len() > 0 {
                                            // build decision by deriving goal

                                            let mut carriedGoalTv: Tv::Tv = retTv(&hitGoalEntry.read().sentence).unwrap();
                                            let mut carriedGoalTerm: Term = predictedTerm.clone();

                                            let mut unifiedSeqOpt: Option<Term> = None; // uified last sequence

                                            for iBelief in execEvidence.iter().rev() { // iterate from back becaue we have to derive from goal to goal
                                                // create synthetic sentence for the current goal
                                                let goal:Sentence = newEternalSentenceByTv(&carriedGoalTerm, EnumPunctation::GOAL, &carriedGoalTv, newStamp(&vec![-1]));
                                                //println!("DBG X {}", convSentenceTermPunctToStr(&goal, true));
                                                //println!("DBG Y {}", convSentenceTermPunctToStr(&iBelief.read(), true));

                                                let iConcl:Sentence = NarInfProcedural::infGoalBelief(&goal, &iBelief.read()).unwrap();

                                                // return subj of iConcl and carry over carriedGoalTerm and TV
                                                // TODO< unify somehow ! >
                                                let unifiedSeqTerm: Term = (*(iConcl.term)).clone(); // sequence which has unified vars
                                                carriedGoalTv = retTv(&iConcl).unwrap();
                                                //println!("DBG Z {}", &convTermToStr(&unifiedSeqTerm));
                                                carriedGoalTerm = retSeqCond2(&unifiedSeqTerm);
                                                unifiedSeqOpt = Some(unifiedSeqTerm.clone()); // remember
                                            }

                                            match unifiedSeqOpt {
                                                Some(unifiedSeq) => { // must always have a value!
                                                    let firstExecEvidence = Arc::clone(&execEvidence[0]);
                                                    let exp: f64 = Tv::calcExp(&carriedGoalTv);

                                                    let dbg__verbosity_predictionDescMkgn = 1; // verbosity of prediction based decision making
                                                    if dbg__verbosity_predictionDescMkgn > 0 {
                                                        println!("DBG pred: made prediction based descn {}  exp={}", convTermToStr(&unifiedSeq), exp);
                                                    }
                                                    
                                                    match bestEntry2 { // is there best entry?
                                                        Some(ref bestEntry4) => {
                                                            if exp > bestEntry4.exp {
                                                                bestEntry2 = Some(BestEntry{
                                                                    unifiedSeq: unifiedSeq,
                                                                    exp:exp,
                                                                    evidence:Some(Arc::clone(&firstExecEvidence))});
                                                            }
                                                        },
                                                        None => {
                                                            bestEntry2 = Some(BestEntry{
                                                                unifiedSeq: unifiedSeq,
                                                                exp:exp,
                                                                evidence:Some(Arc::clone(&firstExecEvidence))});
                                                        }
                                                    }
                                                },
                                                None => {}
                                            }

                                            break; // break because we found a goal which was hit
                                        }
                                    },
                                    None => {}
                                }
                            }

                            
                            let queryResult: Vec<Arc<RwLock<Sentence>>> = NarGoalSystem::query_by_antecedent(&predictedTerm, &nar.evidenceMem.read());
                            if queryResult.len() == 0 {
                                // no result for query of chain, give up

                                //firstExecEvidence = None; // discard decision because it didn't hit goal
                                execEvidence = vec![];
                                break;
                            }

                            // select random
                            let sel: &Arc<RwLock<Sentence>> = {
                                let selIdx:usize = nar.rng.gen_range(0..queryResult.len());
                                &queryResult[selIdx]
                            };

                            /*
                            if iStep == 0 { // we need to remember op of first impl seq
                                // remember first impl seq
                                firstExecEvidence = Some(Arc::clone(sel));
                            }*/

                            //let tvOfSelEvidence: Tv::Tv = retTv(&sel.read()).unwrap();

                            //let conclTv: Tv::Tv = Tv::ded(&predictedTv, &tvOfSelEvidence); // TODO< check if math for TV checks out >

                            // ** set predicted TV as TV of concl
                            //predictedTv = conclTv;
                            // ** extract consequent of belief
                            predictedTerm = retPred(&sel.read().term);

                            execEvidence.push(Arc::clone(sel)); // record the part of the chain!
                        }
                    }
                },
                None => {}
            }
        }

        match bestEntry2 {
            Some(bestEntry3) => {
                if bestEntry3.exp > nar.cfgDescnThreshold {
                    match bestEntry3.evidence {
                        Some(evidence) => {
                            
                            let pickedEvidence: Arc<RwLock<Sentence>> = Arc::clone(&evidence);
                            
                            println!("DBG7 {}", convTermToStr(&bestEntry3.unifiedSeq));

                            // extract op of seq
                            enforce(is_seq(&bestEntry3.unifiedSeq)); // must be sequence
                            enforce(is_seqAndLastOp(&bestEntry3.unifiedSeq)); // last must be op!
                            let opTerm:Term = retSeqLast(&bestEntry3.unifiedSeq);
                            
                            
                            { // info
                                let implSeqAsStr = convSentenceTermPunctToStr(&pickedEvidence.read(), true); // unified
                                let actAsStr:String = convTermToStr(&opTerm);
                                let pickedExp:f64 = bestEntry3.exp;
                                if nar.cfgVerbosity > 0 {println!("descnMaking: found best act = {}   implSeq={}    exp = {}", &actAsStr, &implSeqAsStr, pickedExp)};
                            }


                            // put back into goal system as goal
                            // (a, b) a! |- b!
                            {
                                let conclTerm:Term = opTerm.clone();
                                let tv = retTv(&pickedEvidence.read()).unwrap().clone();
                                let sentence = newEternalSentenceByTv(&conclTerm,EnumPunctation::GOAL,&tv,pickedEvidence.read().stamp.clone());

                                NarGoalSystem::addEntry(&mut nar.goalSystem, &declMem.as_ref().unwrap().read(), nar.t, Arc::new(sentence), Some(pickedEvidence), 0); 
                            }
                        },
                        None => {}
                    }
                }
            },
            None => {}
        }
    }

    // select best desired goal from goal system
    {
        let mut bestExp:f64 = 0.0;
        let mut bestEntry4: Option<Arc<RwLock<NarGoalSystem::Entry>>> = None;
        let mut bestEntryIdx: Option<usize> = None; // used to remove goal after exec
        let mut idx = 0;
        for iActiveGoalEntry in &nar.goalSystem.read().activeSet.set {
            if NarGoalSystem::is_desired(&iActiveGoalEntry.read()) { // only consider desired goals!
                let exp:f64 = Tv::calcExp(&retTv(&iActiveGoalEntry.read().sentence).unwrap());
                if exp > bestExp && exp > nar.cfgDescnThreshold { // is the goal a better goal?
                    bestExp = exp;
                    bestEntry4 = Some(Arc::clone(&iActiveGoalEntry));
                    bestEntryIdx = Some(idx);
                } 
            }
            idx+=1;
        }

        
        match bestEntry4 {
            Some(bestEntry3) => {
                let exp:f64 = Tv::calcExp(&retTv(&bestEntry3.read().sentence).unwrap());
                if exp > nar.cfgDescnThreshold {
                    let mut bestEntry5 = bestEntry3.write();
                    match &bestEntry5.evidence {
                        Some(evidence) => {
                            let pickedEvidence: Arc<RwLock<Sentence>> = Arc::clone(&evidence);

                            let opTerm:Term = (*bestEntry5.sentence.term).clone();
                            //let opTerm:Term = opTerm.clone();

                            // must be op, else something is wrong
                            if !decodeOp(&opTerm).is_some() {
                                println!("[warning] descnMaking: sel goal is not a op!");
                            }
                            // must be op, else something is wrong
                            if decodeOp(&opTerm).is_some() {

                                // try to decode op into args and name
                                let decodedOpOpt: Option<(Vec<Term>,String)> = decodeOp(&opTerm);
                                if nar.cfgVerbosity > 1 {println!("descnMaking: could decode op = {}", decodedOpOpt.is_some());};
                                if decodedOpOpt.is_some() { // we can only exec op if op is valid format
                                    //let decodedOpArgsAndName:(Vec<Term>,String) = decodedOpOpt.unwrap();
                    
                                    pickedAction = Some(opTerm.clone());
                                    
                                    bestEntry5.desirability = 0.0; // we executed action which reduces desirability of goal!
                                    nar.goalSystem.write().activeSet.set.remove(bestEntryIdx.unwrap()); // remove immediatly, else pong3 score suffers!

                                    // add anticipated event
                                    let expIntervalIdx:i64 =
                                        if pickedEvidence.read().expDt.is_some() {
                                            pickedEvidence.read().expDt.unwrap()
                                        }
                                        else {0}; // else it needs a default interval
                                    let interval:i64 = nar.expIntervalsTable[expIntervalIdx as usize];
                                    let deadline:i64 = nar.t + interval; // compute real deadline by exponential interval
                                    
                                    if nar.cfg__enAnticipation { // is anticipation enabled?
                                        nar.anticipatedEvents.push(AnticipationEvent {
                                            evi:Arc::clone(&pickedEvidence),
                                            deadline:deadline,
                                        });
                                    }
                                }
                            }
                        },
                        None => {}
                    }
                }
            },
            None => {}
        }
    }


    
    match &pickedAction {
        Some(_) => {},
        None => {
            if nar.cfgEnBabbling && nar.ops.len() > 0 { // we have to have ops to sample from
                // TODO< better distribution >
                let p = nar.rng.gen_range(0..nar.ops.len()*9);
                if p < nar.ops.len() {
                    let idx = p;

                    if nar.ops[idx].isBabbleable() { // op must be allowed for babbling
                        let opName: &String = &nar.ops[idx].retName(); // sel op

                        let callTerm:Term = encodeOp(&vec![Term::SetExt(vec![Box::new(Term::Name("SELF".to_string()))])], opName);
                        
                        if nar.cfgVerbosity > 5 {println!("procedural: babbling: picked act {}", &convTermToStr(&callTerm));};
                        
                        pickedAction = Some(callTerm.clone());                        
                    }
                }
            }
        }
    }
    
    
    match &pickedAction {
        Some(term) => {
            let (opArgs, opName) = decodeOp(&term).unwrap();

            // search for action with name
            let opOpt = ret_op_by_name(nar, &opName);
            if opOpt.is_some() { // was op found?
                opOpt.unwrap().call(nar, declMem, &opArgs); // call op
            
                println!("{}!", &convTermToStr(&term)); // print execution
    
                nar.trace.event_happened( Rc::new(SimpleSentence {name:term.clone(),evi:nar.t,occT:nar.t}) );
            }
            else {
                // op which was searched was not registered
                println!("[WARN] op {} was not registered!", opName);
            }
        },
        None => {},
    }
    
    nar.trace.limit(); // limit trace (AIKR)

    // limit evidence (AIKR)
    if nar.t % 101 == 1 {
        NarMem::limitMemory(&mut nar.evidenceMem.write(), nar.cfg__nConcepts as usize);
    }

    if nar.t % 13 == 1 {
        NarGoalSystem::limitMemory(&nar.goalSystem, &declMem.as_ref().unwrap().read(), nar.t);
    }

    // give goal system resources
    {

        loop {
            let dt:f32 = (timeStart.elapsed().as_micros() as f32)/1000000.0;

            // decide about resource distribution with max utility
            let mut winner:(f32, String) = (1.0, "CYCLEFIN".to_string());

            {
                let utility:f32 = ((nar.cfg__resourceExpectedTime - dt) / nar.cfg__resourceExpectedTime) * nar.cfg__resourceGoalDeriverPrio;
                if utility > winner.0 {
                    winner = (utility, "GOALDERIV".to_string());
                }
            }

            if &winner.1 == "CYCLEFIN" { // finish the cycle because we invested enough resources
                //println!("DBG: CYCLEFIN");
                break;
            }
            else if &winner.1 == "GOALDERIV" { // derive goal
                //println!("DBG: GOALDERIV");
                // TODO< make this more fine grained >
                NarGoalSystem::sampleAndInference(&mut nar.goalSystem, &declMem.as_ref().unwrap().read(), nar.t, &nar.evidenceMem.read(), &mut nar.rng);
            }
        }
    }

    
    nar.t+=1; // increment time of NAR
}

/// return operation by name
pub fn ret_op_by_name(nar: &ProcNar, name: &String) -> Option<Rc<Box<dyn Op>>> {
    for iOp in &nar.ops {
        if iOp.retName() == *name {
            return Some(Rc::clone(iOp));
            break;
        }
    }
    None
}

/// is the term a op which can be called
fn checkIsCallableOp(nar: &ProcNar, term:&Term) -> bool {
    let opArgsAndNameOpt: Option<(Vec<Term>,String)> = decodeOp(&term);
    if !opArgsAndNameOpt.is_some() {
        return false; // isn't callable because it isn't 
    }
    let (_opArgs, opName) = opArgsAndNameOpt.unwrap();
    
    // TODO< check for {SELF} as first argument! >

    for i in &nar.ops {
        if &i.retName() == &opName {
            return true;
        }
    }
    false
}

// abstraction over term

/// return predicate of impl seq
pub fn retPred(term:&Term) -> Term {
    match term {
        Term::Stmt(Copula::PREDIMPL, _subj, pred) => {
            (**pred).clone()
        },
        _ => {
            panic!("expected pred impl!");
        }
    }
}

/// return subject of impl seq
pub fn retSubj(term:&Term) -> Term {
    match term {
        Term::Stmt(Copula::PREDIMPL, subj, _pred) => {
            (**subj).clone()
        },
        _ => {
            panic!("expected pred impl!");
        }
    }
}

pub fn retImplSeqOp(term:&Term) -> Term {
    match term {
        Term::Stmt(Copula::PREDIMPL, subj, _pred) => {
            match &**subj {
                Term::Seq(seq) => {
                    *seq[1].clone()
                },
                _ => {panic!("expected seq!");}
            }
        },
        _ => {panic!("expected pred impl!");}
    }
}

pub fn retSeqLast(term:&Term) -> Term {
    match &term {
        Term::Seq(seq) => {
            *seq[seq.len()-1].clone() // return last
        },
        _ => {panic!("expected seq!");}
    }
}

pub fn retSeqCond(term:&Term) -> Term {
    match term {
        Term::Stmt(Copula::PREDIMPL, subj, _pred) => {
            match &**subj {
                Term::Seq(seq) => {
                    *seq[0].clone()
                },
                _ => {panic!("expected seq!");}
            }
        },
        _ => {panic!("expected pred impl!");}
    }
}

pub fn retSeqCond2(term:&Term) -> Term {
    match &term {
        Term::Seq(seq) => {
            *seq[0].clone()
        },
        _ => {panic!("expected seq!");}
    }
}

// helper
pub fn is_seq(term:&Term) -> bool {
    return match &term {
        Term::Seq(_seq) => {
            true
        },
        _ => {false}
    }
}

// helper
pub fn is_seqAndLastOp(term:&Term) -> bool {
    return match &term {
        Term::Seq(seq) => {
            let isOp = decodeOp(&*seq[seq.len()-1]).is_some();
            isOp
        },
        _ => {false}
    }
}


/// event
/// string and evidence
/// (emulation of sentence and term)
#[derive(Clone)]
pub struct SimpleSentence {
    pub name:Term,
    pub evi:i64, // evidence id
    pub occT:i64, // occurcence time
}

/// helper to return indices of events with OPS
pub fn calcIdxsOfOps(nar: &ProcNar, trace:&Vec<Rc<SimpleSentence>>) -> Vec<i64> {
    let mut res = Vec::new();
    for idx in 0..trace.len() {
        if checkIsCallableOp(nar, &trace[idx].name) {
            res.push(idx as i64);
        }
    }
    res
}


/// helper to find the minimal index of the exponential interval table
pub fn findMinTableIdx(interval:i64, expIntervalsTable:&Vec<i64>) -> i64 {
    for idx in 1..expIntervalsTable.len() {
        if expIntervalsTable[idx] > interval {
            return idx as i64 - 1;
        }
    }
    return expIntervalsTable.len() as i64 - 1;
}

/// helper to debug evidence to console
pub fn debugEvidence(procNar: &ProcNar) {
    println!("EVIDENCE:");
    for iEvi in &mem_ret_evidence_all_nonunique(procNar) {
        let iEviGuard = iEvi.read();

        let implSeqAsStr = convTermToStr(&iEviGuard.term);

        let evi:&Evidence = &iEviGuard.evi.as_ref().unwrap();
        let (pos,cnt) = match evi {
            Evidence::CNT{pos,cnt} => {(pos,cnt)},
            _ => {panic!("expected CNT");}
        };

        let expDtAsStr = if iEviGuard.expDt.is_some() {format!("+EXPDT{}", iEviGuard.expDt.unwrap())} else {"".to_string()}; // convert expDt to string if it exists
        println!("{} {} {}/{}", &implSeqAsStr, expDtAsStr, pos, cnt);
    }
}

/// anticipated event
#[derive(Clone)]
pub struct AnticipationEvent {
    /// evidence
    pub evi:Arc<RwLock<Sentence>>,
    /// deadline in absolute cycles
    pub deadline:i64,
}

/// trait for a op, all implementations implement a op
pub trait Op {
    /// return name of the op
    fn retName(&self) -> String;
    fn call(&self, nar:&mut ProcNar, narMem:&Option<Arc<RwLock<Mem2>>>, args:&Vec<Term>);
    /// can the op be called with motor babbling?
    fn isBabbleable(&self) -> bool;
    /// how much evidence is added or removed when this op is involved?
    fn ret_evi_cnt(&self) -> i64;
}
