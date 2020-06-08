// NAR perception has the following independent components (the idea is composable modularity):

// - component to filter array of list of sentences by if it is a op or not

// - component to select indices of items which are not empty

// - component to select random item from list

// - component to sample from array by indices

// - component to sort and remove duplicates

// - component to retrieve items from array by indices

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

extern crate rand;

use rand::rngs::ThreadRng;
use rand::Rng;

#[derive(Clone)]
pub struct SentenceDummy {
    pub isOp:bool, // is it a operation?
    pub term:Rc<Term>,
    pub t:i64, // time of occurence 
}

// map if the sentence is a op
pub fn mapArrSentencesByOp(sentences:&Vec<SentenceDummy>) -> Vec<bool> {
    sentences.into_iter().map(|v| v.isOp).collect()
}

// select a random inde which is true
pub fn selRngTrue(arr:Vec<bool>, rng:&mut ThreadRng) -> Option<usize> {
    let mut candidateIndices = vec![];

    for idx in 0..arr.len() {
        if arr[idx] {
            candidateIndices.push(idx);
        }
    }

    if candidateIndices.len() == 0 {
        None
    }
    else {
        let idx = candidateIndices[rng.gen_range(0, candidateIndices.len())]; // select random index
        Some(idx)
    }
}

// tries to perceive events with ops as pivots
pub fn perceiveImpl(events:&Vec<SentenceDummy>, rng:&mut ThreadRng) -> Vec<SentenceDummy> {
    let opIndices = mapArrSentencesByOp(events); // select indices of ops, because we want ops as "pivots"
    let pivotEventIdx = selRngTrue(opIndices, rng);
    match pivotEventIdx {
        Some(pivotIdx) => {
            if pivotIdx > 0 && pivotIdx < events.len()-1 { // can't be last idx!
                // select random idx before
                let idxBefore = rng.gen_range(0, pivotIdx);
                // select random idx after
                let idxAfter = rng.gen_range(pivotIdx+1, events.len());

                // fuse indices
                let mut idces = vec![idxBefore, pivotIdx, idxAfter];

                // sort and remove dupes
                idces.sort();
                idces.dedup();

                // return
                return idces.into_iter().map(|v| events[v].clone()).collect();
            }
            else {
                return vec![];
            }
        },
        None => {
            return vec![];
        }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Copula {
    SIM, // <-> similarity
    INH, // --> inheritance
    PREDIMPL, // =/> predictive implication
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Term {
    Cop(Copula, Rc<Term>, Rc<Term>),
    Name(String),
    Seq(Vec<Rc<Term>>), // sequence
}

pub fn convTermToStr(t:&Term) -> String {
    match t {
        Term::Cop(Copula, subj, pred) => {
            let subjStr = convTermToStr(subj);
            let predStr = convTermToStr(pred);
            let copStr = match Copula {Copula::SIM=>{"<->"},Copula::INH=>{"-->"},Copula::PREDIMPL=>"=/>"};
            format!("<{} {} {}>", subjStr, copStr, predStr)
        }
        Term::Name(name) => name.to_string(),
        Term::Seq(seq) => {
            let mut inner = convTermToStr(&seq[0]);
            for i in 1..seq.len() {
                inner = format!("{} &/ {}", inner, convTermToStr(&seq[i]));
            }
            format!("( {} )", inner)
        }
    }
}





// memory system
pub struct Concept {
    pub name:Rc<Term>,

    pub implBeliefs:Vec<Arc<SentenceDummy>>, // =/> beliefs
}

// memory
pub struct Mem {
    pub concepts:HashMap<Term, Arc<Concept>>,
}

pub fn storeInConcepts(mem: &mut Mem, s:&SentenceDummy) {
    for iTerm in termEnum(&*s.term) { // enumerate all terms, we need to do this to add the sentence to all relevant names
        match mem.concepts.get_mut(&iTerm.clone()) {
            Some(arcConcept) => {
                match Arc::get_mut(arcConcept) {
                    Some(concept) => {
                        println!("TODO - add belief only if it doesn't already exist!");

                        concept.implBeliefs.push(Arc::new((*s).clone())); // add belief

                        // TODO< order by importance >

                        concept.implBeliefs = concept.implBeliefs[..concept.implBeliefs.len().min(20)].to_vec(); // keep under AIKR
                    }
                    None => {
                        println!("INTERNAL ERROR - couldn't aquire arc!");
                    }
                }
            },
            None => { // concept doesn't exist
                // * insert new concept if we are here
                
                let concept = Arc::new(Concept {
                    name:Rc::new(iTerm.clone()),
                    implBeliefs:vec![Arc::new((*s).clone())],
                });
                
                mem.concepts.insert(iTerm.clone(), concept); // add concept to memory
            }
        }
    }
}

// enumerate terms inside term
// DESC< doesn't remove duplicates! >
pub fn termEnum(t:&Term) -> Vec<Term> {
    match t {
        Term::Cop(Copula, subj, pred) => {
            let mut res = vec![t.clone()];
            for i in termEnum(subj) {
                res.push(i.clone());
            }
            for i in termEnum(pred) {
                res.push(i.clone());
            }
            res
        }
        Term::Name(name) => vec![t.clone()],
        Term::Seq(seq) => {
            let mut res = vec![t.clone()];

            for iv in seq {
                for i in termEnum(iv) {
                    res.push(i.clone());
                }
            }
            res
        }
    }
}




// small test to check if the idea for handling anticipation with exponential intervals is correct
pub fn expExpIntervals() {
    // example of the evidence for pong1
    // exponential interval   evidence pos    evidence
    // 0                    |  1               1
    // 1                    |  3               3
    // 2                    |  1               5
    // 3                    |  0               7

    // table with evidence by exponential interval
    let mut eviPlus:Vec<Impl2> = Vec::new();
    let mut eviNeg:Vec<Impl2> = Vec::new();

    // TODO< add table with anticipations >

    let mut batX = 1.0;
    let mut ballX = 2.0;
    let mut batVelX = 0.0;
    let mut ballVelX = 0.1;
    let mut ballDistY = 3.0;

    let mut decisionThreshold = 0.6;

    for _t in 0..10000 {
        // move ball and bat
        batX += batVelX;
        ballX += ballVelX;

        // limit movement of bat
        if ballX < 0.0 {
            ballX = 0.0;
            ballVelX = abs(ballVelX);
        }
        if ballX > 10.0 {
            ballX = 10.0;
            ballVelX = -abs(ballVelX);
        }

        // TODO< reflect ball on walls >
        println!("TODO - reflect ball on walls");

        if ballDistY < 0.0 {
            let diff = batX - ballX;
            let hit = diff > -1.0 && diff < 1.0;
            if hit {
                // TODO< confirm anticipations >
            }
            else { // miss
            }

            // reset ball
            ballDistY = 3.0;
        }

        // TODO< check for failed anticipations >
        println!("TODO - check for failed anticipations");
    }
}

// math helper
pub fn abs(v:f64)->f64 {
    if v < 0.0 {-v} else {v}
}

pub fn retConf(impl_:&Impl2)->f64 {
    let k = 1.0;
    let w = impl_.eviPos as f64;
    w/(w+k)
}

pub fn retFreq(impl_:&Impl2)->f64 {
    (impl_.eviPos as f64) / (impl_.eviCnt as f64)
}

// TESTING
// implication with exponential horizon etc for testing
pub struct Impl2 {
    pub dir:i64, // direction for pong

    pub eviPos:i64, // positive evidence
    pub eviCnt:i64, // count of evidence

    pub expT:i64, // exponential deadline "index" - higher value coresponds to exponential higher deadline
}


