#![allow(non_snake_case)]

mod map2d;
mod ad;

extern crate rand;

//use std::default::Default;

use rand::Rng;
//use rand::distributions::{Normal, Distribution};

pub fn main() {
    //expGa0();return;

    expInvent0();
}

pub fn expInvent0() {
    let mut rng = rand::thread_rng();

    let mut boxX0 = 0; // position of the drawn box
    let problemMap:map2d::Map2d::<f64> = invent0(&mut boxX0); // invent a map of the problem

    println!("- search for NN which solves the task in this environment");
    //println!("- refine NN till it solves the task"); // TODO?

    for iNnSearchStep in 0..5000000 {
        let nNeurons = 3; // number of neurons

        let mut params:Vec::<f64> = vec!(0.0;(5*5+1)*nNeurons);
        // init with random
        for idx in 0..params.len() {
            params[idx] = ((rng.gen::<f64>() * 2.0) - 1.0) * 0.3;//((6059512.42149 * ((((idx + 5994) as f64) + 63.563)).powf(2.0)) % 50.0) as i32;
        }
    
        let mut paramsIdx = 0;
    
        let mut neurons:Vec::<ad::Neuron> = Vec::<ad::Neuron>::new();
    
        for iNeuronIdx in 0..nNeurons { // loop to transfer to neurons
            let mut weights:Vec::<ad::Ad> = Vec::<ad::Ad>::new();
            for i in 0..5*5 {
                let v = params[paramsIdx];
                paramsIdx+=1;
                weights.push(ad::Ad{r:v,d:0.0});
            }
            let bias = params[paramsIdx] * 11.0; // boost parameter because it is the bias
            paramsIdx+=1;
            neurons.push(ad::Neuron{
                weights: weights,
                bias:ad::Ad{r:bias,d:0.0},
                act: 0,
            });
        }
    
        let mut cursorX = 3;
        let mut cursorY = 3;

        for timer in 0..50 {



            let mut stimulus = vec!(ad::Ad{r:0.0,d:0.0};5*5); // stimulus for NN
    
            //println!("- use NN !");
            {
                let w = 5;
                let h = 5;
        
                let mut destIdx=0;
        
                for iiy in 0..h {
                    for iix in 0..w {
                        let v = map2d::readAt(&problemMap, cursorY-h/2+iiy,cursorY-w/2+iix);
                        stimulus[destIdx].r = v;
                        destIdx+=1;
                    }
                }
    
                {
                    // y vector, which is the result of the NN
                    let mut ys = vec!(0.0; neurons.len());
                        
                    ys[0] = ad::calc(&stimulus, &neurons[0]).r;
                    ys[1] = ad::calc(&stimulus, &neurons[1]).r;
                    ys[2] = ad::calc(&stimulus, &neurons[2]).r;
                    
                    //DEBUG - y array to see if NN computes sensible stuff
                    //println!("y[0] = {}", ys[0]);
                    //println!("y[1] = {}", ys[1]);
                    //println!("y[2] = {}", ys[2]);
    
                    let mut maxActIdx=0;
                    let mut maxActYVal = ys[0];
                    for iYIdx in 0..ys.len() {
                        if ys[iYIdx] > maxActYVal {
                            maxActYVal = ys[iYIdx];
                            maxActIdx = iYIdx;
                        }
                    }

                    if maxActIdx == 0 {} // NOP
                    else if maxActIdx == 1 {cursorX+=1;}
                    else if maxActIdx == 2 {cursorX+=2;}
                    
                    /* commented because it is old code
    
                    if ys[0] > 0.5 {
                        println!("FOUND NN with right y! step={}", iNnSearchStep);
    
    
                        { // for manual testing if it depends on the input
                            stimulus = vec!(ad::Ad{r:0.0,d:0.0};5*5);
    
                            ys[0] = ad::calc(&stimulus, &neurons[0]).r;
                            ys[1] = ad::calc(&stimulus, &neurons[1]).r;
    
                            println!("y[0] for null = {}", ys[0]);
                            println!("y[1] for null = {}", ys[1]);
                        }
    
    
                        break;
                    }
                    */
                }
                
            }
        }

        if (cursorX - boxX0).abs() <= 1 { // did we move with the cursor to the edge of the shape?
            println!("archived goal!");
            break;
        }
    


    }





}

// run task invention program
// TODO< can return empty map, we need to check this here inside >
pub fn invent0(boxX0:&mut i32) -> map2d::Map2d::<f64> {
    
    let mut rng = rand::thread_rng();


    let mut map = map2d::Map2d{
        arr:vec!(0.0;10*10),
        w:10,
        h:10,
    };

    for tryIt in 0..500 { // try as long as no fitting environment was found



        let mut v = vec![0; 9];
        // init with random
        for idx in 0..v.len() {
            v[idx] = (rng.gen::<f64>() * 8.0) as i32;//((6059512.42149 * ((((idx + 5994) as f64) + 63.563)).powf(2.0)) % 50.0) as i32;
        }

        map = map2d::Map2d{
            arr:vec!(0.0;10*10),
            w:10,
            h:10,
        };

        // interpret genes to draw
        *boxX0 = v[0]; // we need to write the value outside
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

        // count how many pixels are enabled
        let mut cnt=0;
        for iy in 0..map.h {
            for ix in 0..map.w {
                if map2d::readAt(&map, iy,ix) > 0.5 {
                    cnt+=1;
                }
            }
        }

        if cnt > 1 { // did we draw one pixel?
            return map;
        }
    }


    return map; // give up
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
