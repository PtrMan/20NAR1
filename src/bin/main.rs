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
            
            use nar20_1::NarUtilReadn;
            nar20_1::NarUtilReadn::readNarseseFile(&mut nar, &iFilePath);
        }

        nar20_1::NarInteractive::runInteractive(&mut nar);
    }
    else if runEnv == "pong3" { // run environment
        // jump to environment test
        nar20_1::Reasoner1Entry::reasoner1Entry();
    }
    else if runEnv == "envTTT2" { // run environment
        nar20_1::ProcTicTacToe::run();
    }
    else if runEnv == "chaos" { // run chaos environment to stresstest NAR
        nar20_1::ProcChaosEntry::procChaosEntry();
    }
    else if runEnv == "srv" { // run server
        nar20_1::NarServer::run();
    }
}
