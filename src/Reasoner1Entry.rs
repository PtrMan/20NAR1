#![allow(non_snake_case)]
#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::Term::*;
use crate::Term::convTermToStr;
use crate::NarProc;
use crate::Nar;
use crate::NarGoalSystem;
use crate::NarWorkingCycle::Mem2;

use crate::EnvPong3;

// /return ratio
pub fn reasoner1Entry() -> f64 {
    let mut rng = rand::thread_rng();

    let mut t:i64 = 0; // discrete time
    let maxT:Option<i64> = Some(5000);

    let mut nar:Nar::Nar = Nar::createNar();
    

    let envPong:RefCell<EnvPong3::EnvState> = RefCell::new(EnvPong3::makeEnvState());
    let envPongRc = Rc::new(envPong);

    nar.procNar.ops.push(Rc::new(Box::new( OpPong {
        env: Rc::clone(&envPongRc),
        opDir: 1,
        selfName: "^L".to_string(),
    })));

    nar.procNar.ops.push(Rc::new(Box::new( OpPong {
        env: Rc::clone(&envPongRc),
        opDir: -1,
        selfName: "^R".to_string(),
    })));
    
    loop { // reasoner/modification mainloop
        if t % 10 == 0 {
            Nar::inputN(&mut nar, &"0-1-xc! :|:".to_string()); // add goal
        }
        
        {
            NarProc::narStep0(&mut nar.procNar);

            {
                let envPong = (*envPongRc).borrow();
                let diff:i64 = envPong.ballX-envPong.batX;
                let batWidth:i64 = envPong.batWidth;
                if diff.abs() <= batWidth {
                    nar.procNar.trace.push(Rc::new(NarProc::SimpleSentence {name:Term::Name(format!("{}-{}-x{}", 0, 1, "c")),evi:nar.procNar.t,occT:nar.procNar.t}));
                }
                else if diff < 0 {
                    nar.procNar.trace.push(Rc::new(NarProc::SimpleSentence {name:Term::Name(format!("{}-{}-x{}", 0, 1, "l")),evi:nar.procNar.t,occT:nar.procNar.t}));
                }
                else { // diff > 0
                    nar.procNar.trace.push(Rc::new(NarProc::SimpleSentence {name:Term::Name(format!("{}-{}-x{}", 0, 1, "r")),evi:nar.procNar.t,occT:nar.procNar.t}));
                }
            }
    
            if nar.procNar.trace.len() > 0 {
                println!("{} ballX={} batX={} diff={}", convTermToStr(&nar.procNar.trace[nar.procNar.trace.len()-1].name), (*envPongRc).borrow().ballX, (*envPongRc).borrow().batX, (*envPongRc).borrow().ballX - (*envPongRc).borrow().batX);
            }
            
            NarProc::narStep1(&mut nar.procNar, &Some(Arc::clone(&nar.mem)));
            
            let mut envPong = (*envPongRc).borrow_mut();
            EnvPong3::simStep(&mut envPong, &mut rng);
        }
        
        
        // logic to decide when to break up
        if maxT.is_some() {
            if t >= maxT.unwrap() {
                break; // exit this loop
            }
        }
        t+=1;
    }



    
    // debug all evidence of NAR
    let enDbgEvidence:bool = true;
    if enDbgEvidence {
        println!("");
        NarProc::debugEvidence(&nar.procNar);
        println!("");

        if nar.procNar.evidenceMem.read().concepts.len() == 0 { // check if there is no evidence, which indicates a fatal bug
            panic!("no evidence after running {}", "pong3");
        }
    }

    { // debug goals of goal system 
        println!("{}", NarGoalSystem::dbgRetGoalsAsText(&nar.procNar.goalSystem.read()));
    }
    
    { // print environment score
        println!("[i] env hits={} misses={}", (*envPongRc).borrow().hits, (*envPongRc).borrow().misses);
    }


    println!("[d] reasoner: DONE!");

    return (*envPongRc).borrow().hits as f64 / ((*envPongRc).borrow().misses+(*envPongRc).borrow().hits) as f64;
}





/// ops for pong environment
pub struct OpPong {
    pub env: Rc<RefCell<EnvPong3::EnvState>>, // points at environment
    pub opDir: i64, // direction which is set when this op is called
    pub selfName: String, // name of this op
}


impl NarProc::Op for OpPong {
    fn retName(&self) -> String {
        self.selfName.clone()
    }
    fn call(&self, _nar:&mut NarProc::ProcNar, _narMem:&Option<Arc<RwLock<Mem2>>>, _args:&Vec<Term>) {
        (*self.env).borrow_mut().batVX = self.opDir;
        println!("CALL {}", &self.selfName);
    }
    fn isBabbleable(&self) -> bool {true}
    fn ret_evi_cnt(&self) -> i64 {3}
}
