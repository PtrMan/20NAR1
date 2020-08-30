// facade which consumes narse, comments, empty lines and commands

use std::io;
use crate::Nar::*;
use crate::NarWorkingCycle::{debugCreditsOfTasks};
use crate::NarModuleNlp;
use crate::Term::*;
use crate::TermApi::*;
use crate::NarSentence::{SentenceDummy, EnumPunctation};
use crate::Tv::{Tv};

// /param quit is used to signal if program has to get terminated
// returns requested information as strings!
pub fn input(nar:&mut Nar, line: &String, quit: &mut bool) -> Vec<String> {
    let mut retInfo = Vec::new();
    
    *quit = false;
    
    let mut input = line.clone();
    trimNewline(&mut input);

    // TODO< trim comment by // away   >
    if input.len() >= 2 && &input[..2] == "//" { // is commented line?
        // ignore
    }
    else if input.len() >= 2 && &input[..2] == "!s" {
        let mut nCycles = 1;
        if input.len() > 2 { // parse number of cycles
            // TODO< check if it was parsed fine! >
            nCycles = input[2..].parse::<i64>().unwrap();
        }
        for _i in 0..nCycles {
            cycle(nar);
        }
    }
    else if input.len() > 6 && &input[..6] == "!.nlp " { // command to stuff nlp input into nlp module
        let natural = &input[6..].to_string();
        let mut isQuestion = false;
        let resTermOpt:Option<SentenceDummy> = NarModuleNlp::process(&natural, &mut isQuestion);
        let punct = match isQuestion { // compute punctuation of narsese if it is a question or not
            true => EnumPunctation::QUESTION,
            false => EnumPunctation::JUGEMENT
        };

        if resTermOpt.is_some() {
            let resTerm:&Term = &(*resTermOpt.unwrap().term);
            match resTerm {
                Term::Stmt(Copula::INH, subj, pred) => { // is relationship
                    let prod0;
                    let prod1;
                    let mut prod2:Option<Term> = None;
                    
                    match &**subj {
                        Term::SetExt(set) => {
                            if let [set0] = &set[..1] { 
                                match &**set0 {
                                    Term::Prod(arr) if arr.len() == 2 => {
                                        prod0 = *arr[0].clone();
                                        prod1 = *arr[1].clone();
                                    },
                                    Term::Prod(arr) if arr.len() == 3 => {
                                        prod0 = *arr[0].clone();
                                        prod1 = *arr[1].clone();
                                        prod2 = Some(*arr[2].clone());
                                    },
                                    _ => {
                                        // term doesn't fit expected structure!
                                        println!("W term from NLP isn't recognized 2!");
                                        return retInfo;
                                    }
                                }
                            }
                            else {
                                // term doesn't fit expected structure!
                                println!("W term from NLP isn't recognized 3!");
                                return retInfo;
                            }
                        },
                        _ => {
                            // term doesn't fit expected structure!
                            println!("W term from NLP isn't recognized 1!");
                            return retInfo;
                        }
                    }

                    match &**pred {
                        Term::Name(name) if name == "relIs" => {
                            // translate to inheritance
                            inputT(nar, &s(Copula::INH, &prod0, &prod1), punct, &Tv{f:1.0,c:0.9});
                        },
                        Term::Name(name) if name == "relIs2" => {
                            // translate to inheritance
                            inputT(nar, &s(Copula::INH, &prod0, &prod1), punct, &Tv{f:1.0,c:0.9});
                        },
                        Term::Name(name) if name == "relGENERIC" => {
                            let a = prod2.unwrap();
                            let prod222 = a.clone();
                            let prod223 = a.clone();
                            match prod222 {
                                Term::Name(name) if name == "is" => { // we ave special case for is relation, translate to inheritance
                                    // translate to inheritance
                                    // subj is always a SetInt
                                    inputT(nar, &s(Copula::INH, &prod0, &Term::SetInt(vec![Box::new(prod1)])), punct, &Tv{f:1.0,c:0.9});
                                },
                                _ => {
                                    // pass on to NAR
                                    inputT(nar, &resTerm.clone(), punct, &Tv{f:1.0,c:0.9}); // as raw relation
                                    inputT(nar, &s(Copula::INH, &p2(&prod0, &prod1), &prod223), punct, &Tv{f:1.0,c:0.9}); // pass as inheritance
                                }
                            }
                        },
                        
                        
                        Term::Name(name) if name == "relIsQuery" => {
                            // translate to inheritance question
                            inputT(nar, &s(Copula::INH, &prod0, &prod1), EnumPunctation::QUESTION, &Tv{f:1.0,c:0.9});
                        },
                        
                        _ => {
                            // term doesn't fit expected structure!
                            println!("W term from NLP isn't recognized!");
                            return retInfo;
                        }
                    }
                },
                _ => {
                    // term doesn't fit expected structure!
                    println!("W term from NLP isn't recognized!");
                }
            }
        }
    }
    else if input == "!dt" { // debug tasks
        return debugCreditsOfTasks(&nar.mem);
    }
    else if input == "!dmd" { // debug memory declarative
        // TODO< put into function and call it here >
        return vec![format!("concept count = {}", nar.mem.mem.borrow().concepts.len())];
    }
    else if input == "!QQ" { // quit
        *quit = true;
    }
    else {
        inputN(nar, &input);
    }

    retInfo
}

fn trimNewline(s: &mut String) {
    // from https://blog.v-gar.de/2019/04/rust-remove-trailing-newline-after-input/
    while s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
    }
}