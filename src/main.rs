#![allow(non_snake_case)]

mod map2d;

extern crate rand;

//use std::default::Default;

use rand::Rng;
//use rand::distributions::{Normal, Distribution};

pub fn main() {
    //expGa0();return;

    expInvent0();
}

pub fn expInvent0() {
    let problemMap:map2d::Map2d::<f64> = invent0(); // invent a map of the problem

    println!("TODO - search for NN which solves the task in this environment");
}

// run task invention program
// TODO< can return empty map, we need to check this here inside >
pub fn invent0() -> map2d::Map2d::<f64> {
    
    let mut rng = rand::thread_rng();

    let mut v = vec![0; 9];
    // init with random
    for idx in 0..v.len() {
        v[idx] = (rng.gen::<f64>() * 8.0) as i32;//((6059512.42149 * ((((idx + 5994) as f64) + 63.563)).powf(2.0)) % 50.0) as i32;
    }

    let mut map = map2d::Map2d{
        arr:vec!(0.0;10*10),
        w:10,
        h:10,
    };

    // interpret genes to draw
    map2d::map2dDrawBox(&mut map, v[0],v[1],v[2],v[3],1.0);


    // print to console
    for iy in 0..map.h {
        for ix in 0..map.w {
            if map2d::readAt(&map, iy,ix) > 0.5 {print!("x");}
            else {print!(".");}
            print!(" "); // space for better width ratio
        }
        println!();
    }

    map
}



// run example of GA
pub fn expGa0() {
    let mut rng = rand::thread_rng();

    // found solution programs
    let mut solutions:Vec<Solution0> = vec![];
    
    for iTask in 0..50 { // iterate over tasks
            
        for iRun in 0..50000 {
            //(rng.gen::<f64>() * 2.0 - 1.0) * 0.5;
            
        
            let mut v = vec![0; 9];
            // init with random
            for idx in 0..v.len() {
                v[idx] = (rng.gen::<f64>() * 11.0) as i32;//((6059512.42149 * ((((idx + 5994) as f64) + 63.563)).powf(2.0)) % 50.0) as i32;
            }
            
            let mut ctx = Ctx{ip:0,accu:0,accu2:0,accuf:0.0,accu2f:0.0};
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
    accuf:f64, // float accumulator
    accu2f:f64,
}

// interpret instruction
pub fn interp(ctx:&mut Ctx, instr:i32) {
    let mut incIp=true;
    
    match instr {
        // page 0 : basic integer operations
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

        // page 1 : basic float operations

        10 => {}, // TODO< use >

        11 => ctx.accuf = 0.0, // reset accu
        12 => ctx.accuf = 1.0, // set accu to one
        13 => ctx.accuf+=1.0,
        14 => ctx.accuf-=1.0,
        15 => { // xchg
            let t = ctx.accuf;
            ctx.accuf=ctx.accu2f;
            ctx.accu2f=t;
        },
        16 => ctx.accuf=ctx.accuf+ctx.accu2f,
        17 => ctx.accuf=ctx.accuf-ctx.accu2f,
        18 => ctx.accuf=ctx.accuf*ctx.accu2f,
        19 => ctx.accuf=5.0,
        
        40..=60 => {incIp=false; ctx.ip = instr-20}, // jump absolute
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
