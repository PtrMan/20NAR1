// chaos environment
// used to check if memory management is working correctly

#![allow(non_snake_case)]
#![allow(dead_code)]

use std::rc::Rc;
use parking_lot::RwLock;
use std::sync::Arc;

use crate::Term::*;
use crate::NarProc;
use crate::Nar;
use crate::NarGoalSystem;
use crate::NarWorkingCycle::Mem2;

pub fn procChaosEntry() {
    let mut t:i64 = 0; // discrete time
    let maxT:Option<i64> = Some(1100);

    let mut nar:Nar::Nar = Nar::createNar();
    nar.procNar.cfgNMaxEvidence = 1000; // only allow 5000 beliefs

    nar.procNar.ops.push(Rc::new(Box::new( OpNop {
        selfName: "^L".to_string(),
    })));

    nar.procNar.ops.push(Rc::new(Box::new( OpNop {
        selfName: "^R".to_string(),
    })));
    
    loop { // reasoner/modification mainloop
        if t % 10 == 0 {
            Nar::inputN(&mut nar, &"0-1-xc! :|:".to_string()); // add goal
        }
        
        
        NarProc::narStep0(&mut nar.procNar);

        nar.procNar.trace.event_happened( Rc::new(NarProc::SimpleSentence {name:Term::Name(format!("{}",t)),evi:nar.procNar.t,occT:nar.procNar.t}) );

        NarProc::narStep1(&mut nar.procNar, &None);
        
        
        
        // logic to decide when to break up
        if maxT.is_some() {
            if t >= maxT.unwrap() {
                break; // exit this loop
            }
        }
        t+=1;
    }



    
    // debug all evidence of NAR
    let enDbgEvidence:bool = false;
    if enDbgEvidence {
        NarProc::debugEvidence(&nar.procNar);
    }

    { // debug goals of goal system 
        println!("{}", NarGoalSystem::dbgRetGoalsAsText(&nar.procNar.goalSystem.read()));
    }
    
    {
        // print number of procedural concepts
        println!("nConcepts={}", nar.procNar.evidenceMem.read().concepts.len());
    }

    println!("[d] reasoner: DONE!");
}





// ops for pong environment
pub struct OpNop {
    pub selfName: String, // name of this op
}


impl NarProc::Op for OpNop {
    fn retName(&self) -> String {
        self.selfName.clone()
    }
    fn call(&self, _nar:&mut NarProc::ProcNar, _narMem:&Option<Arc<RwLock<Mem2>>>, _args:&Vec<Term>) {
        println!("CALL {}", &self.selfName);
    }
    fn isBabbleable(&self) -> bool {true}
    fn ret_evi_cnt(&self) -> i64 {3}
}
