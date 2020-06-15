#![allow(non_snake_case)]
#![allow(dead_code)]

use ::Nars;

pub fn reasoner0Entry() {
    let mut t:i64 = 0; // discrete time
    let mut maxT:Option<i64> = Some(350);


    let mut nar:Nars::Nar = Nars::narInit();
    


    let mut ballX:f64 = 3.0;
    let mut batX:f64 = 7.0;
    let mut batVelX:f64 = 0.0;
    
    

    
    loop { // reasoner/modification mainloop
    
        // select option to focus on
        // we hardcoded it so it always returns the first option, which is the only one
        let selFocusItem:usize = pickByMass(&[1.0], 0.5);
        
        if selFocusItem == 0 { // do we want to spend the time in the NARS reasoning?
            Nars::narStep0(&mut nar);

            {
                let diff = ballX - batX;
                if diff > 1.0 {
                    nar.trace.push(Nars::SimpleSentence {name:"r".to_string(),evi:nar.t,occT:nar.t});
                }
                else if diff < -1.0 {
                    nar.trace.push(Nars::SimpleSentence {name:"l".to_string(),evi:nar.t,occT:nar.t});
                }
                else {
                    nar.trace.push(Nars::SimpleSentence {name:"c".to_string(),evi:nar.t,occT:nar.t});
                }
            }
    
    
            println!("{} {}", nar.trace[nar.trace.len()-1].name, ballX - batX);
            
            Nars::narStep1(&mut nar, &mut batVelX);
            
            
            batX += batVelX;
            
            // limit bat
            if batX < 0.0 {
                batX = 0.0;
            }
            if batX > 10.0 {
                batX = 10.0;
            }
        }
        // TODO< add AERA reasoning >
        // TODO< add self improvement things >
        
        
        // logic to decide when to break up
        if maxT.is_some() {
            if t >= maxT.unwrap() {
                break; // exit this loop
            }
        }
        t+=1;
    }



    
    // debug all evidence of NAR
    println!("");
    println!("EVIDENCE:");
    for iEvi in &nar.evidence {
        let implSeqAsStr = format!("({},{})=/>{}",(*iEvi).borrow().seqCond,(*iEvi).borrow().seqOp,(*iEvi).borrow().pred);
        println!("{} +EXPDT{} {}/{}", &implSeqAsStr, (*iEvi).borrow().expDt, (*iEvi).borrow().eviPos, (*iEvi).borrow().eviCnt);
    }
    
    println!("[d] reasoner: DONE!");
}

// pick a option by mass
// /param selVal value for selection in range [0.0;1.0]
pub fn pickByMass(massArr:&[f64], selVal:f64) -> usize {
    let sum:f64 = massArr.iter().sum();
    let mut acc = 0.0;
    let mut idx = 0;
    for iv in massArr {
        if acc >= selVal {
            return idx;
        }
        acc += iv;
        idx+=1;
    }
    
    massArr.len()-1 // sel last
}
