use std::sync::{Arc, Mutex};
use parking_lot::RwLock;

use crate::Term::{Term, convTermToStr};
use crate::Nar::*;
use crate::NarWorkingCycle::QHandler;
use crate::NarSentence::SentenceDummy;
use crate::NarSentence::convSentenceTermPunctToStr;


// evaluate how good NARS is with the narsese-program
pub fn run(nar:&mut Nar)->Option<i64> {
    let global:Arc<Mutex<Global>> = Arc::new(Mutex::new(Global{foundAnswer:false,}));
    
    nar.mem.read().globalQaHandlers.write().push(Arc::new(RwLock::new(QHandlerImpl{global:Arc::clone(&global)}))); // register Q&A handler to send answers to all clients

    let mut maxCycles = 180;
    for iCycle in 0..maxCycles {
        cycle(nar);

        // check if question was answered
        {
            let data = global.lock().unwrap(); // unwrap to panic when it can't unlock
            if data.foundAnswer {
                return Some(iCycle);
            }
        }

    }
    None // didn't find any answer!
}



pub struct Global {
    pub foundAnswer:bool,
}


/// handler to send answer to clients
pub struct QHandlerImpl {
    pub global:Arc<Mutex<Global>>,
}

impl QHandler for QHandlerImpl {
    fn answer(&mut self, question:&Term, answer:&SentenceDummy) {
        // print question and send answer
        let msg = "TRACE answer: ".to_owned() + &convTermToStr(&question) + "? " + &convSentenceTermPunctToStr(&answer, true);

        let mut data = self.global.lock().unwrap(); // unwrap to panic when it can't unlock
        data.foundAnswer = true;
    }
}

