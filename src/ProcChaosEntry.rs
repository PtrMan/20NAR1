// chaos environment
// used to check if memory management is working correctly

#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::Term::*;
use crate::Term::convTermToStr;
use crate::NarSentence::*;
use crate::NarProc;
use crate::Nar;
use crate::NarGoalSystem;

pub fn procChaosEntry() {
    let mut rng = rand::thread_rng();

    let mut t:i64 = 0; // discrete time
    let maxT:Option<i64> = Some(1100);

    let mut nar:Nar::Nar = Nar::createNar();
    nar.procNar.cfgNMaxEvidence = 1000; // only allow 5000 beliefs

    nar.procNar.ops.push(Box::new( OpNop {
        selfName: "^L".to_string(),
    }));

    nar.procNar.ops.push(Box::new( OpNop {
        selfName: "^R".to_string(),
    }));
    
    loop { // reasoner/modification mainloop
        if t % 10 == 0 {
            Nar::inputN(&mut nar, &"0-1-xc! :|:".to_string()); // add goal
        }
        
        
        NarProc::narStep0(&mut nar.procNar);

        nar.procNar.trace.push(NarProc::SimpleSentence {name:Term::Name(format!("{}",t)),evi:nar.procNar.t,occT:nar.procNar.t});

        NarProc::narStep1(&mut nar.procNar);
        
        
        
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
        println!("");
        println!("EVIDENCE:");
        for iEvi in &nar.procNar.evidence {
            let implSeqAsStr = convTermToStr(& (*iEvi).borrow().term);
    
            let eviHelper = (*iEvi).borrow();
            let evi:&Evidence = &eviHelper.evi.as_ref().unwrap();
            let (pos,cnt) = match evi {
                Evidence::CNT{pos,cnt} => {(pos,cnt)},
                _ => {panic!("expected CNT");}
            };
    
            println!("{} +EXPDT{} {}/{}", &implSeqAsStr, (*iEvi).borrow().expDt.unwrap(), pos, cnt);
        }
    }

    { // debug goals of goal system 
        println!("{}", NarGoalSystem::dbgRetGoalsAsText(&nar.procNar.goalSystem));
    }
    
    { // print number of evidence
        // TODO< print number of procedural concepts!
        println!("nEvidence={}", nar.procNar.evidence.len());
    }

    if nar.procNar.evidence.len() as f64 > nar.procNar.cfgNMaxEvidence as f64 * 1.5 {
        panic!("procedural reasoner has to much evidence!");
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
    fn call(&self, _args:&Vec<Term>) {
        println!("CALL {}", &self.selfName);
    }
}
