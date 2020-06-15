#![allow(non_snake_case)]
#![allow(dead_code)]

pub fn main() {
    let mut t:i64 = 0; // discrete time
    let mut maxT:Option<i64> = Some(100);
    
    loop { // reasoner/modification mainloop
    
        // select option to focus on
        // we hardcoded it so it always returns the first option, which is the only one
        let selFocusItem:usize = pickByMass(&[1.0], 0.5);
        
        if selFocusItem == 0 { // do we want to spend the time in the NARS reasoning?
            
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
