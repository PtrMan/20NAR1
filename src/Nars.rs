use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

// DONE< check for events which were anticipated and remove anticipations! >
// DONE< compute expectation while decision making and take the one with the highest exp(), check against decision threshold! >

// DONE< check if we can revise and revise if possible >
// DONE< revise higher expIntervals too >

// DONE< filter for operations in perceptions ! and combine only current event with op and nonop before op >

// DONE< compute table of exponential intervals! >
// DONE< use table of exponential intervals! >

// DONE< compute anticipation deadline by exponential interval thingy >

// contains all necessary variables of a NAR
pub struct Nar {
    pub cfgIntervalExpBase:f64, // base for the exponential intervals
    pub cfgIntervalMax:i64, // maximal interval time

    pub cfgPerceptWindow:i64, // perception window for current events
    pub cfgDescnThreshold:f64,

    pub evidence: Vec<Rc<RefCell<EE>>>,
    
    pub trace: Vec<SimpleSentence>,
    pub anticipatedEvents: Vec<AnticipationEvent>,

    pub ops: Vec<Box<dyn Op>>, // all registered ops

    pub t:i64, // NAR time


    pub rng: rand::rngs::ThreadRng,

    //table with exponential intervals
    pub expIntervalsTable:Vec<i64>,
}

// init and set to default values
pub fn narInit() -> Nar {
    let mut nar = Nar {
        cfgIntervalExpBase: 1.5,
        cfgIntervalMax: 40,
        cfgPerceptWindow: 2,
        cfgDescnThreshold: 0.48,
        evidence: Vec::new(),
        trace: Vec::new(),
        anticipatedEvents: Vec::new(),
        ops: Vec::new(),
        t: 0,

        rng: rand::thread_rng(),

        expIntervalsTable: Vec::new(),
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
    
    // DEBUG
    //for ii in &expIntervalsTable {
    //    println!("{}", ii);
    //}


    nar
}

// does one reasoner step
pub fn narStep0(nar:&mut Nar) {
    println!("ae# = {}", nar.anticipatedEvents.len()); // debug number of anticipated events
    
    // remove confirmed anticipations
    for perceptIdx in 0..nar.cfgPerceptWindow as usize {
        if nar.trace.len() > perceptIdx {
            let curEvent = &nar.trace[nar.trace.len()-1-perceptIdx].name;
            
            let mut newanticipatedEvents = Vec::new();
            for iDeadline in &nar.anticipatedEvents {
                let evi = (*iDeadline).evi.borrow();
                if evi.pred != *curEvent { // is predicted event not current event?
                    newanticipatedEvents.push(iDeadline.clone());
                }
            }
            nar.anticipatedEvents = newanticipatedEvents;
        }
    }

    
    { // neg confirm for anticipated events

        {
            for iDeadlineViolated in nar.anticipatedEvents.iter().filter(|v| v.deadline <= nar.t) {
                let mut mutEvi = (*iDeadlineViolated).evi.borrow_mut();
                mutEvi.eviCnt+=1; // add negative evidence
            }
        }
        
        
        // TODO< refactor this as filtering >
        {
            let mut newanticipatedEvents = Vec::new();
            for iDeadline in &nar.anticipatedEvents {
                if iDeadline.deadline > nar.t {
                    newanticipatedEvents.push(iDeadline.clone());                        
                }
            }
            
            nar.anticipatedEvents = newanticipatedEvents;
        }
        

    }

    let cfgPerceptionSamplesPerStep = 4; // how ofter should event-FIFO get sampled for perception in cycle?
    
    if nar.trace.len() >= 3 { // add evidence
        for sampleIt in 0..cfgPerceptionSamplesPerStep {
            // filter middle by ops and select random first event before that!
            let idxsOfOps:Vec<i64> = calcIdxsOfOps(&nar.trace);
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
                    

                    // is the name a op?
                    let checkIsOp=|name:&String| {
                        for i in &nar.ops {
                            if &i.retName() == name {
                                return true;
                            }
                        }
                        false
                    };

                    // TODO< rewrite to logic which scans for the first op between idxLast and idx1, select random event as idx2 between these!
                    
                    // check if we can select previous event
                    {
                        let sel = nar.trace[nar.trace.len()-1-1].clone();
                        if rng0 == 1 && nar.trace.len()-1-1 > idx1 && !checkIsOp(&sel.name) {
                            idx2 = nar.trace.len()-1-1;
                        }
                    }


                    let mut idxs = vec![idx0,idx1,idx2];
                    idxs.sort();

                    
                    // middle must be op
                    if checkIsOp(&nar.trace[idxs[1]].name) {
                        // first and last must not be op
                        if
                            !checkIsOp(&nar.trace[idxs[0]].name)  &&
                            !checkIsOp(&nar.trace[idxs[2]].name) && 
                            nar.trace[idxs[0]].name != nar.trace[idxs[2]].name
                        {
                            
                            // found a potential sequence to be perceived
                            
                            let e0 = &nar.trace[idxs[0]];
                            let e1 = &nar.trace[idxs[1]];
                            let e2 = &nar.trace[idxs[2]];
                            
                            println!("perceive ({},{})=/>{}", e0.name, e1.name, e2.name);
                            
                            let dt:i64 = e2.occT - e1.occT;
                            // compute exponential delta time
                            let expDt:i64 = findMinTableIdx(dt, &nar.expIntervalsTable);
                            
                            let mut addEvidence:bool = true; // do we need to add new evidence?
                            
                            {
                                for iEERc in &nar.evidence {
                                    let iEE = &mut(*iEERc).borrow_mut();
                                    
                                    if !checkOverlap(&iEE.stamp, &vec!(e0.evi,e1.evi,e2.evi)) { // evidence must no overlap!
                                        if
                                            iEE.expDt >= expDt && // check for greater because we want to count evidence for longer intervals too, because longer ones are "included"
                                            iEE.seqCond == e0.name && iEE.seqOp == e1.name && iEE.pred == e2.name { // does impl seq match?
                                            iEE.stamp = stampMerge(&iEE.stamp, &vec!(e0.evi,e1.evi,e2.evi));
                                            iEE.eviPos += 1;
                                            iEE.eviCnt += 1;
                                            
                                            if false {println!("dbg - REV")};
                                            
                                            addEvidence = false; // because we revised
                                        }                                
                                    }
        
                                }
                            }
                            
                            if addEvidence {
                                nar.evidence.push(Rc::new(RefCell::new(EE {
                                    stamp:vec!(e0.evi,e1.evi,e2.evi),
                                    expDt:expDt,
                                    seqCond:e0.name.clone(),
                                    seqOp:e1.name.clone(),
                                    pred:e2.name.clone(),
                                    eviPos:1,
                                    eviCnt:1,
                                })));
                            }
                        }
                        
                    }
                }
            }
        }
        
    }
    
}

pub fn narStep1(nar:&mut Nar) {    
    let mut pickedAction:Option<String> = None;
    
    
    match &pickedAction {
        Some(act) => {},
        None => {
            // TODO< search with highest exp and exec only if above descision threshold! >
            
            struct Picked {
                evidence:Rc<RefCell<EE>>, // the evidence of the picked option
                
            }
            
            let mut pickedOpt:Option<Picked> = None;
            let mut pickedExp:f64 = 0.0;
            
            
            
            // search if we can satisfy goal
            for iEERc in &nar.evidence {
                let iEE:&EE = &(*iEERc).borrow();
                
                // check impl seq first ! for current event!
                for perceptIdx in 0..nar.cfgPerceptWindow as usize {
                    if nar.trace.len() > perceptIdx {

                                
                        if iEE.seqCond == nar.trace[nar.trace.len()-1-perceptIdx].name && iEE.pred == "0-1-xc" { // does it fullfil goal?

                            let iFreq = retFreq(&iEE);
                            let iConf = retConf(&iEE);
                            let exp = calcExp(&Tv{f:iFreq,c:iConf});
                            
                            if exp > pickedExp {
                                pickedExp = exp;
                                pickedOpt = Some(Picked{evidence:Rc::clone(iEERc)});
                            }
                        }
                    }
                }
            }
            
            if pickedExp > nar.cfgDescnThreshold {
                let picked = pickedOpt.unwrap().evidence;
                let implSeqAsStr = format!("({},{})=/>{}",(*picked).borrow().seqCond,(*picked).borrow().seqOp,(*picked).borrow().pred);
                println!("descnMaking: found best act = {}   implSeq={}    exp = {}", (*picked).borrow().seqOp, &implSeqAsStr, pickedExp);

                pickedAction = Some((*picked).borrow().seqOp.clone());
                
                // add anticipated event
                let expIntervalIdx:i64 = (*picked).borrow().expDt;
                let interval:i64 = nar.expIntervalsTable[expIntervalIdx as usize];
                let deadline:i64 = nar.t + interval; // compute real deadline by exponential interval
                nar.anticipatedEvents.push(AnticipationEvent {
                    evi:Rc::clone(&picked),
                    deadline:deadline,
                });
            }
        },
    }
    
    match &pickedAction {
        Some(act) => {},
        None => {
            // TODO< better distribution >
            let p = nar.rng.gen_range(0, 18);
            if p < nar.ops.len() {
                let idx = p;
                pickedAction = Some(nar.ops[idx].retName());
            }
        }
    }
    
    
    match &pickedAction {
        Some(act) => {
            // scan for action
            for iOp in &nar.ops {
                if &iOp.retName() == act {
                    iOp.call(&Vec::new()); // call op
                    break;
                }
            }

            nar.trace.push(SimpleSentence {name:act.clone(),evi:nar.t,occT:nar.t});
        },
        None => {},
    }
    
    
    // limit trace (AIKR)
    if nar.trace.len() > 20 {
        nar.trace = (&nar.trace[nar.trace.len()-20..]).to_vec();
    }
    
    nar.t+=1; // increment time of NAR
}

// evidence
pub struct EE {
    pub stamp:Vec<i64>, // collection of evidence of stamp
    
    pub seqCond:String, // condition of sequence
    pub seqOp:String, // op of sequence
    pub pred:String, // predicate of impl seq
    
    pub expDt:i64, // exponential time delta
    
    pub eviPos:i64,
    pub eviCnt:i64,
}

pub fn retFreq(evidence:&EE)->f64 {
    (evidence.eviPos as f64) / (evidence.eviCnt as f64)
}

pub fn retConf(evidence:&EE)->f64 {
    (evidence.eviCnt as f64) / ((evidence.eviCnt as f64) + 1.0)
}

// event
// string and evidence
// (emulation of sentence and term)
#[derive(Clone)]
pub struct SimpleSentence {
    pub name:String,
    pub evi:i64, // evidence id
    pub occT:i64, // occurcence time
}

// helper to return indices of events with OPS
pub fn calcIdxsOfOps(trace:&Vec<SimpleSentence>) -> Vec<i64> {
    let mut res = Vec::new();
    for idx in 0..trace.len() {
        if trace[idx].name.chars().next().unwrap() == '^' { // is it a op?
            res.push(idx as i64);
        } 
    }
    
    return res;
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
    pub evi:Rc<RefCell<EE>>, // evidence
    pub deadline:i64, // deadline in absolute cycles
}

#[derive(Clone)]
pub struct Tv {
    pub f:f64,
    pub c:f64,
}

pub fn calcExp(tv:&Tv)->f64 {
    tv.c*(tv.f - 0.5)+0.5
}


// stamp function
pub fn checkOverlap(a:&Vec<i64>, b:&Vec<i64>) -> bool {
    for ia in a {
        if b.iter().any(|i| i==ia) {
           return true;
        }
    }
    false
}

pub fn stampMerge(a:&Vec<i64>, b:&Vec<i64>) -> Vec<i64> {
    // TODO< merge the propper way >
    // TODO< limit size >
    
    let mut res = a.clone();
    for ib in b {
        res.push(*ib);
    }
    return res;
}

// trait for a op, all implementations implement a op
pub trait Op {
    fn retName(&self) -> String; // return name of the op
    fn call(&self, args:&Vec<String>);
}
