use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::NarStamp::*;
use crate::NarSentence::*;
use crate::Tv::*;
use crate::Term::*;
use crate::TermApi::*;
use crate::NarGoalSystem;

// contains all necessary variables of a procedural NAR
pub struct ProcNar {
    pub cfgIntervalExpBase:f64, // base for the exponential intervals
    pub cfgIntervalMax:i64, // maximal interval time

    pub cfgPerceptWindow:i64, // perception window for current events
    pub cfgDescnThreshold:f64,

    pub cfgNMaxEvidence:i64, // maximal number of evidence

    pub cfgPerceptionSamplesPerStep:i64, // how ofter should event-FIFO get sampled for perception in cycle?
    
    pub cfgVerbosity:i64, // how verbose is the reasoner, mainly used for debugging

    pub evidence: Vec<Arc<Mutex<SentenceDummy>>>, //Vec<Rc<RefCell<SentenceDummy>>>,
    
    pub trace: Vec<SimpleSentence>,
    pub anticipatedEvents: Vec<AnticipationEvent>,

    pub ops: Vec<Box<dyn Op>>, // all registered ops

    pub t:i64, // NAR time


    pub rng: rand::rngs::ThreadRng,

    //table with exponential intervals
    pub expIntervalsTable:Vec<i64>,

    pub goalSystem: NarGoalSystem::GoalSystem,
}

// init and set to default values
pub fn narInit() -> ProcNar {
    let mut nar = ProcNar {
        cfgIntervalExpBase: 1.5,
        cfgIntervalMax: 40,
        cfgPerceptWindow: 2,
        cfgDescnThreshold: 0.48,
        cfgNMaxEvidence: 5000,
        cfgPerceptionSamplesPerStep:4,
    
        cfgVerbosity: 10, // be silent

        evidence: Vec::new(),
        trace: Vec::new(),
        anticipatedEvents: Vec::new(),
        ops: Vec::new(),
        t: 0,

        rng: rand::thread_rng(),

        expIntervalsTable: Vec::new(),

        goalSystem: NarGoalSystem::makeGoalSystem(20, 8),
    };


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

// does one reasoner step
pub fn narStep0(nar:&mut ProcNar) {
    if nar.cfgVerbosity > 0 {println!("ae# = {}", nar.anticipatedEvents.len());}; // debug number of anticipated events
    
    // remove confirmed anticipations
    for perceptIdx in 0..nar.cfgPerceptWindow as usize {
        if nar.trace.len() > perceptIdx {
            let curEvent:&Term = &nar.trace[nar.trace.len()-1-perceptIdx].name;
            
            let mut newanticipatedEvents = Vec::new();
            for iDeadline in &nar.anticipatedEvents {
                let evi = iDeadline.evi.lock().unwrap();
                if !checkEqTerm( &retPred(& evi.term), &curEvent) { // is predicted event not current event?
                    newanticipatedEvents.push(iDeadline.clone());
                }
            }
            nar.anticipatedEvents = newanticipatedEvents;
        }
    }

    
    { // neg confirm for anticipated events

        {
            for iDeadlineViolated in nar.anticipatedEvents.iter().filter(|v| v.deadline <= nar.t) {
                let mut mutEvi = iDeadlineViolated.evi.lock().unwrap();
                
                match mutEvi.evi.as_ref().unwrap() {
                    Evidence::CNT{pos,cnt} => {
                        mutEvi.evi = Some(Evidence::CNT{pos:*pos,cnt:cnt+1}); // add negative evidence
                    },
                    _ => {panic!("expected CNT!");}
                }
            }
        }
        
        nar.anticipatedEvents = nar.anticipatedEvents.iter().filter(|&iDeadline| iDeadline.deadline > nar.t).map(|v| v.clone()).collect();
    }

    if nar.trace.len() >= 3 { // add evidence
        for _sampleIt in 0..nar.cfgPerceptionSamplesPerStep {
            // filter middle by ops and select random first event before that!
            let idxsOfOps:Vec<i64> = calcIdxsOfOps(&nar, &nar.trace);
            if idxsOfOps.len() > 0 { // there must be at least one op to sample


                let mut idx1 = 0;
                {
                    let idx1Idx = nar.rng.gen_range(0, idxsOfOps.len());
                    idx1 = idxsOfOps[idx1Idx] as usize;
                }
                
                if idx1 > 0 {
                    
                    let rng0:i64 = nar.rng.gen_range(0, 2);
                    
                    let idx0 = nar.rng.gen_range(0, idx1);
                    let mut idx2 = nar.trace.len()-1; // last event is last

                    // TODO< rewrite to logic which scans for the first op between idxLast and idx1, select random event as idx2 between these!
                    
                    // check if we can select previous event
                    {
                        let sel = nar.trace[nar.trace.len()-1-1].clone();
                        if rng0 == 1 && nar.trace.len()-1-1 > idx1 && !checkIsCallableOp(&nar, &sel.name) {
                            idx2 = nar.trace.len()-1-1;
                        }
                    }


                    let mut idxs = vec![idx0,idx1,idx2];
                    idxs.sort();

                    
                    // middle must be op
                    if checkIsCallableOp(&nar, &nar.trace[idxs[1]].name) {
                        // first and last must not be op
                        if
                            !checkIsCallableOp(&nar, &nar.trace[idxs[0]].name)  &&
                            !checkIsCallableOp(&nar, &nar.trace[idxs[2]].name) && 
                            nar.trace[idxs[0]].name != nar.trace[idxs[2]].name
                        {
                            
                            // found a potential sequence to be perceived
                            
                            let e0 = &nar.trace[idxs[0]];
                            let e1 = &nar.trace[idxs[1]];
                            let e2 = &nar.trace[idxs[2]];
                            
                            if nar.cfgVerbosity > 0 {println!("perceive ({},{})=/>{}", convTermToStr(&e0.name), convTermToStr(&e1.name), convTermToStr(&e2.name));};
                            
                            let dt:i64 = e2.occT - e1.occT;
                            // compute exponential delta time
                            let expDt:i64 = findMinTableIdx(dt, &nar.expIntervalsTable);
                            
                            let addEvidence: Arc<AtomicBool> = Arc::new(AtomicBool::new(true)); // do we need to add new evidence?

                            



                            {
                                for iEEArc in &nar.evidence {
                                    let mut iEE = iEEArc.lock().unwrap();
                                    
                                    if !checkOverlap(&iEE.stamp, &newStamp(&vec!(e0.evi,e1.evi,e2.evi))) { // evidence must no overlap!
                                        if
                                            iEE.expDt.unwrap() >= expDt && // check for greater because we want to count evidence for longer intervals too, because longer ones are "included"
                                            
                                            // does impl seq match?
                                            checkEqTerm(&retSeqCond(&iEE.term), &e0.name) &&
                                            checkEqTerm(&retImplSeqOp(&iEE.term), &e1.name) &&
                                            checkEqTerm(&retPred(&iEE.term), &e2.name)
                                        {
                                            iEE.stamp = merge(&iEE.stamp, &newStamp(&vec!(e0.evi,e1.evi,e2.evi)));
                                            match iEE.evi.as_ref().unwrap() {
                                                Evidence::CNT{pos,cnt} => {
                                                    iEE.evi = Some(Evidence::CNT{pos:pos+1,cnt:cnt+1}); // bump positive counter
                                                },
                                                _ => {panic!("expected CNT!");}
                                            }
                                            
                                            if false {println!("dbg - REV")};
                                            
                                            addEvidence.store(false, Ordering::Relaxed); // because we revised
                                        }                                
                                    }
        
                                }
                            }
                            
                            if addEvidence.load(Ordering::Relaxed) {
                                nar.evidence.push(Arc::new(Mutex::new(SentenceDummy {
                                    punct:EnumPunctation::JUGEMENT,
                                    t:None,
                                    stamp:newStamp(&vec!(e0.evi,e1.evi,e2.evi)),
                                    expDt:Some(expDt),
                                    term:Arc::new(s(Copula::PREDIMPL, &seq(&vec![e0.name.clone(), e1.name.clone()]), &e2.name.clone())), // (e0 &/ e1) =/> e2
                                    evi:Some(Evidence::CNT{pos:1,cnt:1})
                                })));
                            }
                        }
                        
                    }
                }
            }
        }
        
    }
    
}

pub fn narStep1(nar:&mut ProcNar) {    
    let mut pickedAction:Option<Term> = None; // complete term of op
    {
        let mut bestEntry:(f64, Option<(Rc<RefCell<NarGoalSystem::Entry>>, Term)>) = (0.0, None); // best entry from goal system to execute

        // * search if we can satisfy goal
        for perceptIdx in 0..nar.cfgPerceptWindow as usize {
            if nar.trace.len() > perceptIdx {

                let checkedState:Term = nar.trace[nar.trace.len()-1-perceptIdx].name.clone();

                // check if current state "leads" to action
                // tuple is (exp, entity)
                let thisEntry: (f64, Option<(Rc<RefCell<NarGoalSystem::Entry>>, Term)>) = NarGoalSystem::selHighestExpGoalByState(&nar.goalSystem, &checkedState);

                if thisEntry.0 > bestEntry.0 { // if exp is higher -> is a better choice
                    bestEntry = thisEntry;
                }
            }
        }



        // * pick action and expected event to anticipations
        if bestEntry.0 > nar.cfgDescnThreshold && bestEntry.1.is_some() {
            let entryAndUnfied:&(Rc<RefCell<NarGoalSystem::Entry>>, Term) = &bestEntry.1.unwrap();
            let entity:&Rc<RefCell<NarGoalSystem::Entry>> = &entryAndUnfied.0;
            let pickedEvidenceOpt: &Option<Arc<Mutex<SentenceDummy>>> = &entity.borrow().evidence;
            
            if pickedEvidenceOpt.is_some() {
                let pickedEvidence: Arc<Mutex<SentenceDummy>> = Arc::clone(&pickedEvidenceOpt.as_ref().unwrap());

                // extract op of seq
                println!("DBG16 {}", &convTermToStr(&entryAndUnfied.1));
                let opTerm:Term = retSeqOp(&entryAndUnfied.1);


                { // info
                    let implSeqAsStr = convTermToStr(&entryAndUnfied.1); // unified
                    let actAsStr:String = convTermToStr(&opTerm);
                    let pickedExp:f64 = bestEntry.0;
                    println!("descnMaking: found best act = {}   implSeq={}    exp = {}", &actAsStr, &implSeqAsStr, pickedExp);
                }
                
                // try to decode op into args and name
                let decodedOpOpt: Option<(Vec<Term>,String)> = decodeOp(&opTerm);
                if decodedOpOpt.is_some() { // we can only exec op if op is valid format
                    //let decodedOpArgsAndName:(Vec<Term>,String) = decodedOpOpt.unwrap();

                    pickedAction = Some(opTerm.clone());
                    
                    // add anticipated event
                    let expIntervalIdx:i64 =
                        if pickedEvidence.lock().unwrap().expDt.is_some() {
                            pickedEvidence.lock().unwrap().expDt.unwrap()
                        }
                        else {0}; // else it needs a default interval
                    let interval:i64 = nar.expIntervalsTable[expIntervalIdx as usize];
                    let deadline:i64 = nar.t + interval; // compute real deadline by exponential interval
                    nar.anticipatedEvents.push(AnticipationEvent {
                        evi:Arc::clone(&pickedEvidence),
                        deadline:deadline,
                    });
                }
            }
        }
    }
    
    match &pickedAction {
        Some(_) => {},
        None => {
            // TODO< better distribution >
            let p = nar.rng.gen_range(0, nar.ops.len()*9);
            if p < nar.ops.len() {
                let idx = p;
                let opName: &String = &nar.ops[idx].retName(); // sel op

                let callTerm:Term = encodeOp(&vec![Term::SetExt(vec![Box::new(Term::Name("SELF".to_string()))])], opName);
                
                if nar.cfgVerbosity > 5 {println!("procedural: babbling: picked act {}", &convTermToStr(&callTerm));};
                
                pickedAction = Some(callTerm.clone());
            }
        }
    }
    
    
    match &pickedAction {
        Some(term) => {
            let (opArgs, opName) = decodeOp(&term).unwrap();

            // scan for action
            for iOp in &nar.ops {
                if iOp.retName() == opName.clone() {
                    iOp.call(&opArgs); // call op
                    break;
                }
            }
            
            println!("{}!", &convTermToStr(&term)); // print execution

            nar.trace.push(SimpleSentence {name:term.clone(),evi:nar.t,occT:nar.t});
        },
        None => {},
    }
    
    
    // limit trace (AIKR)
    if nar.trace.len() > 20 {
        nar.trace = (&nar.trace[nar.trace.len()-20..]).to_vec();
    }

    // limit evidence (AIKR)
    if nar.t % 101 == 1 && nar.evidence.len() > nar.cfgNMaxEvidence as usize {
        nar.evidence.sort_by(|a, b| calcExp(&retTv(&b.lock().unwrap()).unwrap()).partial_cmp(&calcExp(&retTv(&a.lock().unwrap()).unwrap())).unwrap()); // order by importance
        nar.evidence = nar.evidence[..nar.evidence.len().min(nar.cfgNMaxEvidence as usize)].to_vec(); // keep under AIKR
    }


    // give goal system resources
    if nar.t % 3 == 0 {
        let enGoalSystem = true; // DISABLED because we want to test it without deriving goals!
        if enGoalSystem {
            NarGoalSystem::sampleAndInference(&mut nar.goalSystem, nar.t, &nar.evidence, &mut nar.rng);
        }
    }

    if nar.t % 13 == 1 {
        NarGoalSystem::limitMemory(&mut nar.goalSystem, nar.t);
    }
    
    nar.t+=1; // increment time of NAR
}


// is the term a op which can be called
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

// return predicate of impl seq
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

pub fn retSeqOp(term:&Term) -> Term {
    match &term {
        Term::Seq(seq) => {
            *seq[1].clone()
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

// decodes a operator into the arguments and name
// returns None if the term can't be decoded
// expects term to be <{(arg0 * arg1 * ...)} --> ^opname>
pub fn decodeOp(term:&Term) -> Option<(Vec<Term>,String)> {
    match term {
        Term::Stmt(Copula::INH, subj, pred) => {
            match &**pred {
                Term::Name(predName) => {
                    match &**subj {
                        Term::SetExt(subj2) if subj2.len() == 1 => {
                            match &*subj2[0] {
                                Term::Prod(args) if args.len() >= 1 => {
                                    return Some((args.iter().map(|v| (**v).clone()).collect(), predName.clone()));
                                },
                                _ => {return None;}
                            }
                        },
                        _ => {return None;}
                    }
                },
                _ => {return None;}
            }
        },
        _ => {return None;}
    }
}

// encode op, used to get called from external code
pub fn encodeOp(args:&Vec<Term>, name:&String) -> Term {
    let argProd = Term::Prod(args.iter().map(|v| Box::new(v.clone())).collect()); // build product of arg
    Term::Stmt(Copula::INH, Box::new(Term::SetExt(vec![Box::new(argProd)])), Box::new(Term::Name(name.clone())))
}

// event
// string and evidence
// (emulation of sentence and term)
#[derive(Clone)]
pub struct SimpleSentence {
    pub name:Term,
    pub evi:i64, // evidence id
    pub occT:i64, // occurcence time
}

// helper to return indices of events with OPS
pub fn calcIdxsOfOps(nar: &ProcNar, trace:&Vec<SimpleSentence>) -> Vec<i64> {
    let mut res = Vec::new();
    for idx in 0..trace.len() {
        if checkIsCallableOp(nar, &trace[idx].name) {
            res.push(idx as i64);
        }
    }
    res
}


// helper to find the minimal index of the exponential interval table
pub fn findMinTableIdx(interval:i64, expIntervalsTable:&Vec<i64>) -> i64 {
    for idx in 1..expIntervalsTable.len() {
        if expIntervalsTable[idx] > interval {
            return idx as i64 - 1;
        }
    }
    return expIntervalsTable.len() as i64 - 1;
}

// anticipated event
#[derive(Clone)]
pub struct AnticipationEvent {
    pub evi:Arc<Mutex<SentenceDummy>>, // evidence
    pub deadline:i64, // deadline in absolute cycles
}

// trait for a op, all implementations implement a op
pub trait Op {
    fn retName(&self) -> String; // return name of the op
    fn call(&self, args:&Vec<Term>);
}
