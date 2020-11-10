// TODO< may have some bugs because NARS doesn't seem to learn anything ?! >

use std::io;
use rand::Rng;
use rand::rngs::ThreadRng;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};


use crate::Nar;
use crate::NarProc;
use crate::NarGoalSystem;
use crate::Term::*;
//use crate::NarInputFacade;

#[derive(Debug)]
pub struct Gamestate {
    pub field: Vec<char>,
    pub player: bool, // true: is player next?    false: oponents move
}


pub fn run() {
    // who plays?
    // 'h' human - for testing
    // 'r' random
    // 'a' AI (nars)
    let player:char = 'a';

    let mut wins:i64 = 0;
    let mut losses:i64 = 0;


    let mut rng: ThreadRng = rand::thread_rng();

    let mut nar:Nar::Nar = Nar::createNar();
    nar.procNar.cfg__eviCnt = 10000; // almost axiomatic
    nar.procNar.goalSystem.cfg__enGoalSatisfaction = false; // disable because we want goals to persist
    
    // resources
    nar.procNar.cfg__nConcepts = 10000;
    nar.procNar.cfg__nConceptBeliefs = 1000;
    nar.procNar.goalSystem.nMaxEntries = 5000; // give more resources (memory - goals)
    nar.procNar.cfg__nGoalDeriverSamples = 100; // give a lot of samples so that it builds the tree fast

    // debugging
    //nar.procNar.cfgVerbosity = 1; // debug perceptions
    nar.procNar.cfg__enAnticipation = false; // disable for testing


    nar.procNar.cfgEnBabbling = false; // disable by default


    let move_:RefCell<Option<i64>> = RefCell::new(None);
    let moveRc = Rc::new(move_);

    // add moves
    for iMove in 0..9 {
        nar.procNar.ops.push(Rc::new(Box::new( OpCheckers {
            sharedMove: Rc::clone(&moveRc),
            //act:false,
            opMove: iMove,
            selfName: format!("^{}", iMove),
        })));
    }
    
    Nar::inputN(&mut nar, &"w! :|:".to_string()); // add goal

    let mut cntGames:i64 = -1;

    let maxEpochs:i64 = 500; // maximum number of tried epochs, thest is interrupted after this number is reached

    loop { // loop over individual game "epochs"
        if cntGames >= maxEpochs {
            break; // terminate test because time limit reached
        }
    
        cntGames+=1;

        //let mut gamestate = envRc.borrow_mut();
        let mut gamestate = Gamestate{field: vec![' '; 9], player:true,}; // create(reset) gamestate

        // flush trace  because it shouldn't confuse moves
        nar.procNar.trace = vec![];

        let mut moveCnt:i64 = 0;

        loop { // loop as long as this game is going

            


            // flush anticipations   because anticipations don't matter, and because the moves can happen in fast succession to NARS
            nar.procNar.anticipatedEvents = vec![];


            loop { // loop until we get a valid input from user

                if gamestate.player { // is player or is it automated opponent?
                    



                    if player == 'h' { // is human playing?
                        let mut input2 = String::new();
                        match io::stdin().read_line(&mut input2) {
                            Ok(_) => {
                                let mut input = input2.clone();
                                trimNewline(&mut input);
            
                                let mut isValidMove = false;
            
                                
                                if input == "0" {
                                    isValidMove = tryMove(&mut gamestate, 0);
                                }
                                else if input == "1" {
                                    isValidMove = tryMove(&mut gamestate, 1);
                                }
                                else if input == "2" {
                                    isValidMove = tryMove(&mut gamestate, 2);
                                }
                                else if input == "3" {
                                    isValidMove = tryMove(&mut gamestate, 3);
                                }
                                else if input == "4" {
                                    isValidMove = tryMove(&mut gamestate, 4);
                                }
                                else if input == "5" {
                                    isValidMove = tryMove(&mut gamestate, 5);
                                }
                                else if input == "6" {
                                    isValidMove = tryMove(&mut gamestate, 6);
                                }
                                else if input == "7" {
                                    isValidMove = tryMove(&mut gamestate, 7);
                                }
                                else if input == "8" {
                                    isValidMove = tryMove(&mut gamestate, 8);
                                }
            
                                if isValidMove {
                                    break;
                                }
                            },
                            Err(error) => println!("error: {}", error),
                        }
                    }
                    if player == 'r' { // random agent?
                        let isValidMove = tryMove(&mut gamestate, rng.gen_range(0,9));
                        if isValidMove {
                            break;
                        }
                    }
                    /*
                    else if moveCnt == 0 { // let first move be a random move for more exploration!
                        let isValidMove = tryMove(&mut gamestate, rng.gen_range(0,9));
                        if isValidMove {
                            break;
                        }
                    }*/
                    else { // let NARS pick action
                        
                        let mut narMove:Option<i64> = None;

                        loop {
                            // flush trace  because it shouldn't confuse moves
                            nar.procNar.trace = vec![];
                            // flush anticipations   because anticipations don't matter, and because the moves can happen in fast succession to NARS
                            nar.procNar.anticipatedEvents = vec![];

                            nar.procNar.cfgEnBabbling = true; // must enable

                            // remember NARS about current gamestate
                            {
                                let stimulusVec: String = retFieldAsString(&gamestate.field);
                                println!("NARS stimulus: {}", stimulusVec);
    
                                //NarProc::narStep0(&mut nar.procNar);
                                nar.procNar.trace.push(Rc::new(NarProc::SimpleSentence {name:Term::Name(stimulusVec.clone()),evi:nar.procNar.t,occT:nar.procNar.t}));
                                //NarProc::narStep1(&mut nar.procNar);    
                            }

                            for _iCycle in 0..4 {
                                NarProc::narStep0(&mut nar.procNar);
                                NarProc::narStep1(&mut nar.procNar);
        
                                if moveRc.borrow().is_some() { // did NARS make a move?
                                    let mut x = moveRc.borrow_mut();
                                    narMove = (*x).clone(); // store move
                                    *x = None; // reset move
                                    break;
                                }
    
                                if narMove.is_some() {
                                    break; // break this loop when NARS made a move
                                }
                            }

                            if narMove.is_some() {
                                break; // propagate
                            }
                        }

                        if narMove.is_some() {
                            let isValidMove = tryMove(&mut gamestate, narMove.unwrap());
                            if isValidMove {
                                break;
                            }
                        }

                    }

                    moveCnt+=1;
                }
                else {
                    
                    
                    // random opponent - try random move until we hit a valid move
                    let isValidMove = tryMove(&mut gamestate, rng.gen_range(0,9));
                    if isValidMove {
                        break;
                    }
                }


            }

            println!("{:?}", gamestate);


            // check if someone won
            let mut winner:Option<char> = None;
            
            // check for equal row
            for iRow in 0..3 {
                if gamestate.field[iRow*3+0] != ' ' && gamestate.field[iRow*3+0] == gamestate.field[iRow*3+1] && gamestate.field[iRow*3+1] == gamestate.field[iRow*3+2] {
                    winner = Some(gamestate.field[iRow*3+0]);
                    break;
                }
            }

            // check for equal column
            for iColumn in 0..3 {
                if gamestate.field[iColumn+0] != ' ' && gamestate.field[iColumn+0] == gamestate.field[iColumn+3] && gamestate.field[iColumn+3] == gamestate.field[iColumn+6] {
                    winner = Some(gamestate.field[iColumn+0]);
                    break;
                }
            }

            // check for equal crosses
            if gamestate.field[0] != ' ' && gamestate.field[0] == gamestate.field[3+1] && gamestate.field[3+1] == gamestate.field[6+2] {
                winner = Some(gamestate.field[0]);
            }
            if gamestate.field[2] != ' ' && gamestate.field[2] == gamestate.field[3+1] && gamestate.field[3+1] == gamestate.field[3*2+0] {
                winner = Some(gamestate.field[2]);
            }

            nar.procNar.cfgEnBabbling = false; // disable by default

            if winner.is_some() { // did someone win?
                println!("WINNER: {}", winner.unwrap());

                println!("w/l ratio = {}   games = {}", (wins as f64)/(losses as f64), cntGames);

                if winner.unwrap() == 'a' {
                    wins+=1;
                }
                else if winner.unwrap() == 'b' {
                    losses+=1;
                }

                if winner.unwrap() == 'a' { // NARS won!
                    Nar::inputN(&mut nar, &"w. :|:".to_string()); // add win event

                    for _iStep in 0..50 { // give reasoner time
                        NarProc::narStep0(&mut nar.procNar);
                        NarProc::narStep1(&mut nar.procNar);
                    }    



                    // TESTING - for testing - see if reasoner finds a plan to the goal 
                    /*
                    loop { // give reasoner infinite amount of time
                        
                        for _iStep in 0..50 {
                            NarProc::narStep0(&mut nar.procNar);
                            NarProc::narStep1(&mut nar.procNar);
                        }

                        println!("DBG goals:");
                        println!("{}", &NarGoalSystem::dbgRetGoalsAsText(&nar.procNar.goalSystem));


                        panic!("TROUBLESHOOTING - DONE"); // terminate for trouble shooting

                    }
                    */
                }


                break;
            }
        
            let isDraw = !retFieldAsString(&gamestate.field).contains("S");// did no one win?
            if isDraw {
                println!("DRAW!");
                break;
            }

            if !gamestate.player { // was the enemy active, then let NARS perceive the outcome of the enemy action too
                // compute stimulus vector for NARS
                let stimulusVec: String = retFieldAsString(&gamestate.field);
                println!("NARS stimulus: {}", stimulusVec);

                NarProc::narStep0(&mut nar.procNar);
                nar.procNar.trace.push(Rc::new(NarProc::SimpleSentence {name:Term::Name(stimulusVec.clone()),evi:nar.procNar.t,occT:nar.procNar.t}));
                NarProc::narStep1(&mut nar.procNar);
            }

            for _iStep in 0..50 { // give reasoner time
                NarProc::narStep0(&mut nar.procNar);
                NarProc::narStep1(&mut nar.procNar);
            }

            



            gamestate.player = !gamestate.player; // switch to opposite player
        }



    }
}

fn trimNewline(s: &mut String) {
    // from https://blog.v-gar.de/2019/04/rust-remove-trailing-newline-after-input/
    while s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
    }
}

fn retFieldAsString(field: &Vec<char>) -> String {
    field.clone().into_iter().collect::<String>().replace(" ", "S")
}

/// try move with current player
fn tryMove(gamestate: &mut Gamestate, idx:i64) -> bool {
    let sign = if gamestate.player {'a'} else {'b'}; // compute sign of player

    if gamestate.field[idx as usize] != ' ' {
        return false;
    };
    gamestate.field[idx as usize] = sign;

    true
}


/// ops for checkers environment
pub struct OpCheckers {
    pub sharedMove: Rc<RefCell<Option<i64>>>,
    pub opMove: i64, // move when this op is called
    pub selfName: String, // name of this op
}


impl NarProc::Op for OpCheckers {
    fn retName(&self) -> String {
        self.selfName.clone()
    }
    fn call(&self, _nar:&mut NarProc::ProcNar, _args:&Vec<Term>) {
        let mut x = self.sharedMove.borrow_mut();
        *x = Some(self.opMove); // store move
        println!("CALL {}", &self.selfName);
    }
    fn isBabbleable(&self) -> bool {true}
}
