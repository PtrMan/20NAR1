#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate rand;

use std::collections::HashMap;

use std::rc::Rc;

//use std::default::Default;
use std::process::exit;
use rand::Rng;
use rand::distributions::{Normal, Distribution};

mod Reasoner0Entry;
use Reasoner0Entry::{reasoner0Entry};
mod map2d;
mod ad;
mod mlutils;

mod narPerception;


pub fn main() {
    //expGa0();return;

    // POWERPLAY like algorithm to solve simple problem(s)
    //expInvent0();

    // jump to reasoner
    reasoner0Entry();
}


// experiment a bit in the direction of PowerPlay
// scifi: is supposed to invent more and more difficult tasks and adapt previous found solutions

// TODO< make task harder with a 2nd task which is harder >
//    TODO< we need to modify the network for that with "Evolutionary Strategies!" >

// TODO< implement peripherial vision too >
pub fn expInvent0() {
    let mut solvedProblems:Vec<Box<dyn ProblemInstance>> = vec![];
    let mut lastParameters:Option<Vec::<f64>> = None; // parameters of agent, is none if we don't yet have best parameters

    // we generate problems on the fly, so we need to iterate over them!
    for iProblem in 0..5 { // iterate over more and more difficult problems
        solveProblem(iProblem, &mut lastParameters, &mut solvedProblems);
    }


    println!("DONE - reason: no unsolved problems");


}

// inner loop which does the problem solving.
// the source of the problem is a problem generator, which generates more and more difficult problems
// /param problemNr number of the problem, starting from zero
pub fn solveProblem(problemNr:i64, lastParameters:&mut Option<Vec::<f64>>, solvedProblems:&mut Vec<Box<dyn ProblemInstance>>) {
    let mut rng = rand::thread_rng();

    let mut cfgVerbosity = 1;

    if cfgVerbosity>0{println!("- search for NN which solves the task in this environment")};
    //println!("- refine NN till it solves the task"); // TODO?

    let mut iNnSearchStep:i64 = -1;



    let mut currentProblems = Vec::<Box<dyn ProblemInstance>>::new(); // problem instances which it needs to all solve with one single modified agent
    { // create problems
        for _iVersion in 0..12 { // iterate over version of the same problem
            let mut boxX0 = 0; // position of the drawn box
            let mut boxY0 = 0;
            let problemDifficulty:i64 = problemNr; // the problem difficulty
            let problemMap:map2d::Map2d::<f64> = invent0(problemDifficulty, &mut boxX0, &mut boxY0); // invent a map of the problem

            currentProblems.push(Box::new(CursorProblem {
                boxX0:boxX0,
                boxY0:boxY0,
                problemMap:problemMap,
                difficulty:problemDifficulty,
            }));
        }
    }


    loop {
        iNnSearchStep+=1;
        if iNnSearchStep >= 8000000 {
            println!("FAILED SEARCH: give up! reason: search took to many iterations!");
            break;
        }






        let nNeuronsLayer0:i64 = 5; // number of neurons
        let nNeuronsLayer1:i64 = 5; // number of neurons

        let mut params:Vec::<f64> = match lastParameters {
            // agent is already parameterized by last winner
            Some(params) => {
                // mutation of EA candidate
                let normal = Normal::new(0.0, 0.02 * (1.0 + 0.00001*(iNnSearchStep as f64))); // standard deviation 0.02, add more randomness with longer searches!
                let mut param2 = params.clone();
                for idx in 0..param2.len() {
                    param2[idx] += normal.sample(&mut rng);
                }

                param2
            },
            // agent isn't parameterized yet, we need to fall back to "random weight guessing" algorithm
            None    => {
                let mut params = vec!(0.0;((5*5+1)*nNeuronsLayer0 + (nNeuronsLayer0+1)*nNeuronsLayer1) as usize);
                // init with random
                for idx in 0..params.len() {
                    params[idx] = ((rng.gen::<f64>() * 2.0) - 1.0) * 1.0;// * 0.3
                }
                params
            }
        };
        
    
        let mut paramsIdx = 0;
        let mut network:Network = buildNnFromParameters(&mut paramsIdx, &params, nNeuronsLayer0, nNeuronsLayer1); // build network from parameters

        let nProblems = currentProblems.len()+solvedProblems.len();
        for iEnvStimulusVersion in 0..nProblems {
            let iProblem =
                if iEnvStimulusVersion < currentProblems.len() {&currentProblems[iEnvStimulusVersion]} // select from current problem
                else {&solvedProblems[iEnvStimulusVersion-currentProblems.len()]}; // select from solved problem because we need to solve them too!

            let mut solverState:SolverState = SolverState {
                cursorX : 3,
                cursorY : 3,
            };

            let isSolved = iProblem.checkSolved(&network, &mut solverState);
            if !isSolved {
                break; // we don't need to continue if unsolved
            }
            if isSolved && iEnvStimulusVersion == nProblems - 1 { // were all problems solved?
                { // store all problems because we solved them
                    for icurrentProblem in currentProblems {
                        solvedProblems.push(icurrentProblem);
                    }
                }
                
                if cfgVerbosity>0{println!("problem#{} steps={} v#{}  archived  F I N A L  goal!", problemNr, iNnSearchStep, iEnvStimulusVersion)};
                
                
                return;
            }

            if iEnvStimulusVersion > 4 { // don't spam output
                if cfgVerbosity>0{println!("problem#{} steps={} v#{}  archived goal!", problemNr, iNnSearchStep, iEnvStimulusVersion)};
            }
        }
    }
}

// helper to build network from parameters
pub fn buildNnFromParameters(paramsIdx:&mut i64, params:&Vec<f64>, nNeuronsLayer0:i64, nNeuronsLayer1:i64) -> Network {
    let mut network:Network = Network { // network of the solver
        neuronsLayer0:Vec::<ad::Neuron>::new(),
        neuronsLayer1:Vec::<ad::Neuron>::new(),
    };

    for _iNeuronIdx in 0..nNeuronsLayer0 { // loop to transfer to neurons
        let mut weights:Vec::<ad::Ad> = Vec::<ad::Ad>::new();
        for _i in 0..5*5 {
            let v = params[*paramsIdx as usize];
            *paramsIdx+=1;
            weights.push(ad::Ad{r:v,d:0.0});
        }
        let bias = params[*paramsIdx as usize] * 15.0; // boost parameter because it is the bias
        *paramsIdx+=1;
        network.neuronsLayer0.push(ad::Neuron{
            weights: weights,
            bias:ad::Ad{r:bias,d:0.0},
            act: 0,
        });
    }

    for _iNeuronIdx in 0..nNeuronsLayer1 { // loop to transfer to neurons
        let mut weights:Vec::<ad::Ad> = Vec::<ad::Ad>::new();
        for _i in 0..nNeuronsLayer0 {
            let v = params[*paramsIdx as usize];
            *paramsIdx+=1;
            weights.push(ad::Ad{r:v,d:0.0});
        }
        let bias = params[*paramsIdx as usize] * 8.0; // boost parameter because it is the bias
        *paramsIdx+=1;
        network.neuronsLayer1.push(ad::Neuron{
            weights: weights,
            bias:ad::Ad{r:bias,d:0.0},
            act: 1,
        });
    }

    network
}


// structure for a problem where the solver has to find a program to position a cursor to the right spot
pub struct CursorProblem {
    pub boxX0: i32, // target position
    pub boxY0: i32,

    pub difficulty: i64, // how difficult is the task?

    pub problemMap:map2d::Map2d::<f64>, // map with the evironment of the problem
}

impl ProblemInstance for CursorProblem {
    fn checkSolved(&self, solverNetwork:&Network, solverState:&mut SolverState) -> bool {
        
        for _timer in 0..50 {


            let w:i32 = 5;
            let h:i32 = 5;
            let mut stimulus = vec!(ad::Ad{r:0.0,d:0.0};(w*h) as usize); // stimulus for NN
    
            //println!("- use NN !");
            {
                
        
                let mut destIdx=0;
        
                for iiy in 0..h {
                    for iix in 0..w {
                        let v = map2d::readAt(&self.problemMap, solverState.cursorY-h/2+iiy,solverState.cursorX-w/2+iix);
                        stimulus[destIdx].r = v;
                        destIdx+=1;
                    }
                }
    
                {
                    // y vector, which is the result of the NN for layer[0]
                    let mut ys0 = vec!(ad::Ad{r:0.0,d:0.0}; solverNetwork.neuronsLayer0.len());
                    for ysIdx in 0..ys0.len() {
                        ys0[ysIdx] = ad::calc(&stimulus, &solverNetwork.neuronsLayer0[ysIdx]);
                    }

                    // layer[1]
                    let mut ys1 = vec!(ad::Ad{r:0.0,d:0.0}; solverNetwork.neuronsLayer1.len());
                    
                    for ysIdx in 0..ys1.len() {
                        ys1[ysIdx] = ad::calc(&ys0, &solverNetwork.neuronsLayer1[ysIdx]);
                    }

                    // TODO< wire up output layer >
                    
                    //DEBUG - y array to see if NN computes sensible stuff
                    //println!("y[0] = {}", ys[0]);
                    //println!("y[1] = {}", ys[1]);
                    //println!("y[2] = {}", ys[2]);
    
                    let mut maxActIdx=0;
                    let mut maxActYVal = ys1[0].r;
                    for iYIdx in 0..ys1.len() {
                        if ys1[iYIdx].r > maxActYVal {
                            maxActYVal = ys1[iYIdx].r;
                            maxActIdx = iYIdx;
                        }
                    }

                    if maxActIdx == 0 {} // NOP
                    else if maxActIdx == 1 {solverState.cursorX+=1; solverState.cursorX = solverState.cursorX % self.problemMap.w;}
                    else if maxActIdx == 2 {solverState.cursorX-=1; solverState.cursorX = (solverState.cursorX + self.problemMap.w) % self.problemMap.w;}
                    else if maxActIdx == 3 {solverState.cursorY+=1; solverState.cursorY = solverState.cursorY % self.problemMap.h;}
                    else if maxActIdx == 4 {solverState.cursorY-=1; solverState.cursorY = (solverState.cursorY + self.problemMap.h) % self.problemMap.h;}
                }
                
            }
        }

        
        
        if (solverState.cursorX - self.boxX0).abs() <= 1 {
            if self.difficulty >= 3 {

                if ( solverState.cursorY - self.boxY0).abs() <= 1 { // y position is even more difficult
                    return true;
                }
                return false;
            }
            else {
                return true;
            }
        }

        return false;
    }
}


// run task invention program
// TODO< can return empty map, we need to check this here inside >
// /param problemDifficulty difficulty of the problem itself
pub fn invent0(problemDifficulty:i64, boxX0:&mut i32, boxY0:&mut i32) -> map2d::Map2d::<f64> {
    
    let mut rng = rand::thread_rng();


    let mut map = map2d::Map2d{
        arr:vec!(0.0;10*10),
        w:10,
        h:10,
    };

    for _tryIt in 0..500 { // try as long as no fitting environment was found



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

        *boxY0 = 0;
        if problemDifficulty > 0 {
            *boxY0 = v[1] % (map.h-1-4); // let y position depend on random var in this more complicated case!
        }

        // interpret genes to draw
        *boxX0 = v[0] % (map.w-1-4); // we need to write the value outside
        //map2d::map2dDrawBox(&mut map, *boxX0 ,v[1],v[2],v[3],1.0); // commented becaus it was to random for simple difficulty
        
        if problemDifficulty >= 4 {
            // draw vertical cross
            map2d::drawLine(&mut map, *boxX0 - 1, *boxY0 - 1, *boxX0 + 1, *boxY0 + 1, 1.0);
            map2d::drawLine(&mut map, *boxX0 + 2, *boxY0 + 2, *boxX0 - 2, *boxY0 - 2, 1.0);
        }
        else if problemDifficulty >= 3 {
            map2d::drawLine(&mut map, *boxX0 - 1, *boxY0 - 1, *boxX0 + 1, *boxY0 + 1, 1.0);
        }
        else if problemDifficulty >= 2 {
            let mut r=3;
            map2d::map2dDrawCircle(&mut map, *boxX0 ,*boxY0,r,1.0); // draw circle with radius 3
        }
        else {
            // TODO< draw with nonfixed height >
            map2d::map2dDrawBox(&mut map, *boxX0 ,*boxY0,v[2],50,1.0);
        }




        // DEBUG print to console
        let debugProblemImage = true; // do we want to debug the image of the problem?
        if debugProblemImage {
            println!("image of problem:");
            for iy in 0..map.h {
                for ix in 0..map.w {
                    if map2d::readAt(&map, iy,ix) > 0.5 {print!("x");}
                    else {print!(".");}
                    print!(" "); // space for better width ratio
                }
                println!();
            }
            println!("---");
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

        if cnt > 2 { // did we draw one pixel?
            return map;
        }
    }


    return map; // give up
}


// network used to solve a problem
pub struct Network {
    pub neuronsLayer0:Vec::<ad::Neuron>,
    pub neuronsLayer1:Vec::<ad::Neuron>,
}

// state of the solver
pub struct SolverState {
    pub cursorX:i32, // x position of cursor
    pub cursorY:i32, // y position of cursor
}

// a problem instance to be solved with the solver
pub trait ProblemInstance {
    // does the proposed solution solve the problem?
    // /param solverNetwork neural-network of the tested solver
    fn checkSolved(&self, solverNetwork:&Network, solverState:&mut SolverState) -> bool;
}
