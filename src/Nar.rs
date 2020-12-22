//! Non-Axiomatic Reasoner
//! is exposing a NAR as one "unit" which can be instantiated

use std::rc::Rc;
use std::sync::{Arc};
use std::sync::atomic::{Ordering};
use parking_lot::RwLock;

use crate::Tv::*;
use crate::Term::*;
use crate::NarseseParser::parseNarsese;
use crate::NarSentence::*;
use crate::NarStamp::*;
use crate::NarWorkingCycle::*;
use crate::NarProc;
use crate::NarGoalSystem;


/// single Non-Axiomatic Reasoner
///
/// every program which uses NARS must use at least one NAR for reasoning.
// PUBLICAPI
pub struct Nar {
    /// procedural NAR
    pub procNar:NarProc::ProcNar,

    /// actual (declarative) memory
    pub mem:Arc<RwLock<Mem2>>,

    /// verbosity of input
    pub cfgVerbosityInput:i32,

    /// number of beliefs of concept
    pub cfg__nConceptBeliefs: usize,

    pub cfg__maxComplexity: i64,
}

/// creates a new NAR with a default configuration
// PUBLICAPI
pub fn createNar() -> Nar {
    let cfg__nConceptBeliefs = 20;
    let cfg__maxComplexity = 64;

    Nar{
        procNar:NarProc::narInit(),
        mem:createMem2(cfg__maxComplexity, cfg__nConceptBeliefs),
        cfgVerbosityInput:0, // enable verbose input by default
        cfg__nConceptBeliefs:cfg__nConceptBeliefs,
        cfg__maxComplexity: cfg__maxComplexity,
    }
}

/// input for eternal
// PUBLICAPI
pub fn inputT(nar:&mut Nar, term:&Term, punct:EnumPunctation, tv:&Tv) {
    inputT2(nar, term, punct, tv, false);
}

/// input for event or eternal
// PUBLICAPI
pub fn inputT2(nar:&mut Nar, term:&Term, punct:EnumPunctation, tv:&Tv, isEvent:bool) {
    let stampId:i64 = nar.mem.read().shared.read().stampIdCounter.fetch_add(1, Ordering::SeqCst); // TODO< is this ordering ok? >
    let stamp = newStamp(&vec![stampId]);
    let mut sentence = newEternalSentenceByTv(&term,punct,&tv,stamp);

    if nar.cfgVerbosityInput >= 1 {
        println!("[v] input {}", convSentenceTermPunctToStr(&sentence, true));
    }

    if isEvent {
        if punct == EnumPunctation::GOAL {
            // add to goals
            NarGoalSystem::addEntry(&mut nar.procNar.goalSystem, nar.procNar.t, Arc::new(sentence), None, 0);
        }
        else {
            // add event
            nar.procNar.trace.push(Rc::new(NarProc::SimpleSentence {name:term.clone(),evi:nar.procNar.t,occT:nar.procNar.t}));
        }

        return;
    }

    // compute if the term is a temporal term
    let isTemporal = match term {
        Term::Stmt(Copula::PREDIMPL, _, _) => {true},
        _ => {false}
    };

    if isTemporal {
        // add to temporal knowledge
        sentence.evi = Some(Evidence::CNT{pos:10,cnt:10}); // we need to transcribe TV
                                                         // TODO< transcribe TV in a better way, we need to approximate freq and conf! >
        
        NarProc::mem_add_evidence(Arc::clone(&nar.procNar.evidenceMem), &sentence, nar.cfg__nConceptBeliefs);
    }
    else {
        if punct == EnumPunctation::GOAL {
            println!("ERR : eternal goals are not supported!");
        }
        else {
            memAddTask(Arc::clone(&nar.mem.read().shared), &sentence, true, nar.cfg__maxComplexity, nar.cfg__nConceptBeliefs, 1.0);
        }
    }
}

/// input narsese
/// return if narsese was parsed and had no error
// PUBLICAPI
pub fn inputN(nar:&mut Nar, narsese:&String) -> bool {
    match parseNarsese(narsese) {
        Some((term, tv, punct, isEvent)) => {
            inputT2(nar, &term, punct, &tv, isEvent);
            true
        },
        None => {
            // TODO< handle error correctly by returning a error >
            println!("ERR - couldn't parse!");
            false
        }
    }
}


/// do one cycle
///
/// give NAR compute time in the form of one cycle
// PUBLICAPI
pub fn cycle(nar:&mut Nar) {
    reasonCycle(Arc::clone(&nar.mem));
}
