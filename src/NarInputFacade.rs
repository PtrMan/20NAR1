//! facade which consumes narse, comments, empty lines and commands

use std::rc::Rc;
use std::sync::Arc;

use crate::Nar::*;
use crate::NarWorkingCycle::{debugCreditsOfTasks};
use crate::NarModuleNlp;
use crate::NarModuleNlp2;
use crate::Term::*;
use crate::TermApi::*;
use crate::NarSentence::{SentenceDummy, EnumPunctation};
use crate::Tv::{Tv};
use crate::NarProc;
use crate::OpLib;

/// gives facade a new line, which can be narsese or a command
///
/// /param quit is used to signal if program has to get terminated
/// returns requested information as strings!
// PUBLICAPI
pub fn input(nar:&mut Nar, line: &String, quit: &mut bool) -> Vec<String> {
    let retInfo = Vec::new();
    
    *quit = false;
    
    let mut input = line.clone();
    trimNewline(&mut input);

    // TODO< trim comment by // away   >
    if input.len() >= 2 && &input[..2] == "//" { // is commented line?
        // ignore
    }
    
    else if input.len() >= 3 && &input[..3] == "!sp" { // step procedural
        //let mut nCycles = 1;
        //if input.len() > 5 && false { // parse number of cycles
        //    // TODO< check if it was parsed fine! >
        //    nCycles = input[5..].parse::<i64>().unwrap();
        //}
        
        NarProc::narStep0(&mut nar.procNar);
        NarProc::narStep1(&mut nar.procNar, &Some(Arc::clone(&nar.mem)));
    }
    else if input == "!spA" { // step procedural A
        NarProc::narStep0(&mut nar.procNar);
    }
    else if input == "!spB" { // step procedural B
        NarProc::narStep1(&mut nar.procNar, &Some(Arc::clone(&nar.mem)));
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
    else if input.len() >= 5 && &input[..5] == "!por " { // procedural op register --- register op, argument is type/name of op, 2nd argument is name of op
        let args:Vec<&str> = input[5..].split_whitespace().collect();
        if args.len() == 1 {
            let argOpType:String = args[0].to_string();
            if argOpType == "execinj" { // it it a exec and inject NAL9 op to get registered
                // add op
                nar.procNar.ops.push(Rc::new(Box::new(OpLib::Op_nal9__exec_and_inject{})));
                println!("added op");
            }
            else if argOpType == "nlpRel0" { // NLP op to add relation to declarative knowledge
                // add op
                nar.procNar.ops.push(Rc::new(Box::new(OpLib::Op__nlp_rel_0{})));
                println!("added op");
            }
            else {} // other types aren't supported
        }
        else if args.len() == 2 { // must have two args
            let argOpType:String = args[0].to_string();
            let argOpName:String = args[1].to_string();
            if argOpType == "NOP" { // it it a NOP operator to get registered?
                // add op
                nar.procNar.ops.push(Rc::new(Box::new(OpLib::OpNop{name:argOpName})));
                println!("added op");
            }
            else {} // other types aren't supported
        }
    }
    else if input == "!pse" { // procedural show evidence
        NarProc::debugEvidence(&nar.procNar);
    }
    else if input == "!peb 0" { // procedural enable babbling
        nar.procNar.cfgEnBabbling = false;
    }
    else if input == "!peb 1" {
        nar.procNar.cfgEnBabbling = true;
    }
    else if input.len() > 7 && &input[..7] == "!.nlp2 " {
        let natural = &input[6..].to_string();
        NarModuleNlp2::process(nar, &natural);
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
        return debugCreditsOfTasks(&*nar.mem.read());
    }
    else if input == "!dmd" { // debug memory declarative
        // TODO< put into function and call it here >
        return vec![format!("concept count = {}", nar.mem.read().shared.read().mem.read().concepts.len())];
    }
    else if input == "!QQ" { // quit
        *quit = true;
    }
    else if input == "" {}
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
