#![allow(non_snake_case)]

extern crate rand;

//use std::default::Default;

use rand::Rng;
//use rand::distributions::{Normal, Distribution};

// run example of GA
pub fn main() {
    let mut rng = rand::thread_rng();

    // found solution programs
    let mut solutions:Vec<Solution0> = vec![];
    
    for iTask in 0..50 { // iterate over tasks
            
        for iRun in 0..50000 {
            //(rng.gen::<f64>() * 2.0 - 1.0) * 0.5;
            
        
            let mut v = vec![0; 9];
            // init with random
            for idx in 0..v.len() {
                v[idx] = (rng.gen::<f64>() * 20.0) as i32;//((6059512.42149 * ((((idx + 5994) as f64) + 63.563)).powf(2.0)) % 50.0) as i32;
            }
            
            let mut ctx = Ctx{ip:0,accu:0,accu2:0};
            for iStep in 0..9 {
                let ip = ctx.ip;
                let instr = v[ip as usize];
                //println!("ip {}  instr {}", ip, instr);
                interp(&mut ctx, instr);
            }
            
            //println!("res {} {}", ctx.accu, ctx.accu2);
            
            let targetValue = iTask; // the target value must be the task id - easy to find entropy this way
            if ctx.accu == targetValue {
                println!("found solution! {}", targetValue);

                // store solution
                solutions.push(Solution0{prgm:v.clone()});

                break;
            }
        }
    
    }

    // print solutions
    {
        for iSolution in solutions {
            println!("{:?}", iSolution.prgm);
        }
    }

}

// context to store interpretation state
pub struct Ctx {
    ip:i32,
    accu:i32, // accumulator
    accu2:i32,
}

// interpret instruction
pub fn interp(ctx:&mut Ctx, instr:i32) {
    let mut incIp=true;
    
    match instr {
        // Match a single value
        0 => {}, // nop
        1 => ctx.accu = 0, // reset accu
        2 => ctx.accu = 1, // set accu to one
        3 => ctx.accu+=1,
        4 => ctx.accu-=1,
        5 => { // xchg
            let t = ctx.accu;
            ctx.accu=ctx.accu2;
            ctx.accu2=t;
        },
        6 => ctx.accu=ctx.accu+ctx.accu2,
        7 => ctx.accu=ctx.accu-ctx.accu2,
        8 => ctx.accu=ctx.accu*ctx.accu2,
        9 => ctx.accu=5,
        
        20..=40 => {incIp=false; ctx.ip = instr-20}, // jump absolute
        // Handle the rest of cases
        _ => {},
    }

    if incIp {
        ctx.ip+=1;
    }
}


// structure to store solution
struct Solution0 {
    prgm: Vec<i32>, // program
}
