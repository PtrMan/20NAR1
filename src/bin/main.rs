#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate rand;
extern crate nom;
//extern crate rand_distr;

//use std::default::Default;
//use std::process::exit;
//use rand_distr::Normal;

pub fn main() {
    //NnTrain::testTrainingNn0();

    let runEnv:String = std::env::args().nth(1).expect("no environment given");

    if runEnv == "it" { // run interactive
        let mut nar = nar20_1::Nar::createNar();

        for iFilepathIdx in 0..std::env::args().len()-2 { // iterate over paths of nars files to load
            let iFilePath:String = std::env::args().nth(2+iFilepathIdx).unwrap();
            
            nar20_1::NarUtilReadn::readNarseseFile(&mut nar, &iFilePath);
        }

        nar20_1::NarInteractive::runInteractive(&mut nar);
    }
    else if runEnv == "envPong3" { // run environment
        // jump to environment test
        nar20_1::Reasoner1Entry::reasoner1Entry();
    }
    else if runEnv == "envTTT2" { // run environment
        nar20_1::ProcTicTacToe::run(500);
    }
    else if runEnv == "envTTTprof" { // run environment for profiling
        nar20_1::ProcTicTacToe::run(10);
    }
    else if runEnv == "chaos" { // run chaos environment to stresstest NAR
        nar20_1::ProcChaosEntry::procChaosEntry();
    }
    else if runEnv == "srv" { // run server
        nar20_1::NarServer::run();
    }
    else if runEnv == "pe" { // run procedural evaluation
        // run envPong3 a lot of times
        // TODO< compute number of runs based on statistics >
        let mut avgRatio: f64 = 0.0;
        let mut nRuns = 0;
        for _iRun in 0..10 {
            let runRatio:f64 = nar20_1::Reasoner1Entry::reasoner1Entry();
            avgRatio += runRatio;
            nRuns += 1;
        }
        avgRatio /= nRuns as f64;

        println!("avg score = {}", avgRatio);
    }
    else if runEnv == "bQA" { // run Q&A benchmark
        

        for iFilepathIdx in 0..std::env::args().len()-2 { // iterate over paths of nars files to load
            let mut nar = nar20_1::Nar::createNar();
            
            let iFilePath:String = std::env::args().nth(2+iFilepathIdx).unwrap();
            
            nar20_1::NarUtilReadn::readNarseseFile(&mut nar, &iFilePath);


            let evalRes:Option<i64> = nar20_1::Eval::run(&mut nar);
            match evalRes {
                Some(cycles) => {
                    println!("{}", cycles);
                },
                None => {
                    println!("None"); // found no result
                }
            }
        }
    }
    else if runEnv == "bQA2" { // run Q&A benchmark
        let mut acc:f64 = 0.0;
        let mut nRuns = 5000; // how may runs are added up?

        for _iRun in 0..nRuns {

            

            for iFilepathIdx in 0..std::env::args().len()-2 { // iterate over paths of nars files to load
                let mut nar = nar20_1::Nar::createNar();
                
                let iFilePath:String = std::env::args().nth(2+iFilepathIdx).unwrap();
                
                nar20_1::NarUtilReadn::readNarseseFile(&mut nar, &iFilePath);

                let evalRes:Option<i64> = nar20_1::Eval::run(&mut nar);
                match evalRes {
                    Some(cycles) => {
                        println!("CYCLES {}", cycles);
                        acc += ((cycles as f64*0.002*-1.0).exp());
                    },
                    None => {
                        println!("CYCLES None"); // found no result
                    }
                }
            }
        }
        acc /= (nRuns as f64); // calc average

        println!("SCORE {}", acc); // print score

    }
    
}
