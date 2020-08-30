use crate::Tv::*;
use crate::Term::*;
use crate::NarseseParser::parseNarsese;
use crate::NarSentence::*;
use crate::NarStamp::*;
use crate::NarWorkingCycle::*;
use crate::Nars;

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

pub fn inputT(nar:&mut Nar, term:&Term, punct:EnumPunctation, tv:&Tv) {
    let stamp = newStamp(&vec![nar.mem.stampIdCounter]);
    nar.mem.stampIdCounter+=1;
    let sentence = newEternalSentenceByTv(&term,punct,&tv,stamp);

    if nar.cfgVerbosityInput >= 1 {
        println!("[v] input {}", convSentenceTermPunctToStr(&sentence, true));
    }

    memAddTask(&mut nar.mem, &sentence, true);
}

// input narsese
pub fn inputN(nar:&mut Nar, narsese:&String) {
    match parseNarsese(narsese) {
        Some((term, tv, punct)) => {
            inputT(nar, &term, punct, &tv);
        },
        None => {
            // TODO< handle error correctly by returning a error >
            println!("ERR - couldn't parse!");
        }
    }
}

pub fn cycle(nar:&mut Nar) {
    reasonCycle(&mut nar.mem);
}
