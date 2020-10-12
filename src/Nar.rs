use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::Tv::*;
use crate::Term::*;
use crate::NarseseParser::parseNarsese;
use crate::NarSentence::*;
use crate::NarStamp::*;
use crate::NarWorkingCycle::*;
use crate::Nars;
use crate::NarGoalSystem;

pub struct Nar {
    pub procNar:Nars::ProcNar, // procedural NAR

    pub mem:Mem2, // actual (declarative) memory

    pub cfgVerbosityInput:i32, // verbosity of input
}

pub fn createNar() -> Nar {
    Nar{
        procNar:Nars::narInit(),
        mem:createMem2(),
        cfgVerbosityInput:1, // enable verbose input by default
    }
}

// for eternal
pub fn inputT(nar:&mut Nar, term:&Term, punct:EnumPunctation, tv:&Tv) {
    inputT2(nar, term, punct, tv, false);
}

pub fn inputT2(nar:&mut Nar, term:&Term, punct:EnumPunctation, tv:&Tv, isEvent:bool) {    
    let stamp = newStamp(&vec![nar.mem.stampIdCounter]);
    nar.mem.stampIdCounter+=1;
    let sentence = newEternalSentenceByTv(&term,punct,&tv,stamp);

    if nar.cfgVerbosityInput >= 1 {
        println!("[v] input {}", convSentenceTermPunctToStr(&sentence, true));
    }

    if isEvent {
        if punct == EnumPunctation::GOAL {
            // add to goals
            NarGoalSystem::addEntry(&mut nar.procNar.goalSystem, Arc::new(sentence));
        }
        else {
            // add event
            nar.procNar.trace.push(Nars::SimpleSentence {name:term.clone(),evi:nar.procNar.t,occT:nar.procNar.t});
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
        nar.procNar.evidence.push(Rc::new(RefCell::new(sentence)));
    }
    else {
        if punct == EnumPunctation::GOAL {
            println!("ERR : eternal goals are not supported!");
        }
        else {
            memAddTask(&mut nar.mem, &sentence, true);
        }
    }
}

// input narsese
// return if narsese was parsed and had no error
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

pub fn cycle(nar:&mut Nar) {
    reasonCycle(&mut nar.mem);
}
