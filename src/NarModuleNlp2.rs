// module which implements basic NLP functionality
// 
// implementation spawns a "worker NAR" to process the preprocessed sentence

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::Nar::*;
use crate::Term::*;
use crate::TermApi::*;
use crate::NarWorkingCycle::{Task2, debugCreditsOfTasks, QHandler};
use crate::Tv::*;
use crate::NarStamp::newStamp;
use crate::NarSentence::{SentenceDummy, EnumPunctation};

pub fn processInternal(natural:&String, isQuestion:&mut bool)->Option<SentenceDummy> {
    *isQuestion = false;

    let nCycles = 200; // number of reasoning cycles for "worker NAR"

    let mut tokens: Vec<&str> = natural.split_whitespace().collect(); // split into tokens

    if tokens.len() > 0 && tokens[tokens.len()-1] == "?" { // it is a question if it ends with a question-mark
        *isQuestion = true;
        tokens = tokens[..tokens.len()-1].to_vec();// cut question mark away
    }

    let mut workerNar = createNar();

    { // load NAL module from file
        use crate::NarUtilReadn;
        crate::NarUtilReadn::readNarseseFile(&mut workerNar, &"nalMod/modNlp.nal".to_string());    
    }
    
    let mut termTokens:Vec<Box<Term>> = vec![];
    {
        // feed unknown words into NAR, translate question to tokens
        for iToken in &tokens {
            if false {}
            //else if *iToken == "a" || *iToken == "an") {} 
            else if  *iToken == "is" {} // ignore special token
            else if  *iToken == "are" {} // ignore special token
            else {
                let mut term:Term = Term::Name((&iToken).to_string());
                if iToken.len() > 0 && iToken.chars().nth(0).unwrap().is_uppercase() { // do we have to represent the term as a set because it is a entity?
                    term = Term::SetExt(vec![Box::new(term)]);
                }

                let term:Term = s(Copula::INH, &Term::SetExt( vec![Box::new(p2(&Term::Name(format!("WORDEN{}", iToken)), &term))]), &Term::Name("RELrepresent".to_string()));
                inputT(&mut workerNar, &term, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.998});
            }

            let iTokenReplaced = iToken.replace("'","_"); // make parsable
            termTokens.push(Box::new(Term::Name("WORDEN".to_string()+&iTokenReplaced)));
        }
    }


    // ask question directly
    let answerHandler0:NlpAnswerHandler = NlpAnswerHandler{answer:None};
    let answerHandlerRef0 = Rc::new(RefCell::new(answerHandler0));
    let rc0 = Rc::clone(&answerHandlerRef0);
    {
        let sentence = SentenceDummy {
            term:Arc::new( s(Copula::INH, &p2(&Term::SetExt(vec![Box::new(Term::Prod(termTokens))]), &Term::QVar("0".to_string())), &Term::Name("RELrepresent".to_string())) ),
            t:None, // time of occurence
            punct:EnumPunctation::QUESTION,
            stamp:newStamp(&vec![999]),
            evi:None,
            expDt:None
        };

        workerNar.mem.questionTasks.push(Box::new(Task2 {
            sentence:sentence,
            handler:Some(answerHandlerRef0),
            bestAnswerExp:0.0, // because has no answer yet
            prio:1.0,
        }));
    }

    for _iCycle_ in 0..nCycles { // give worker NAR time to reason
        cycle(&mut workerNar);
    }

    // for debugging
    for iLine in &debugCreditsOfTasks(&workerNar.mem) {
        println!("{}", iLine);
    }

    // return answer of question
    
    let res0 = rc0.borrow_mut().answer.clone();
    if res0.is_some() {
        return res0;
    }

    None
}

// /param parentNar is the NAR to which the beliefs and questions will be fed
pub fn process(parentNar: &mut Nar, natural:&String) {
    let mut isQuestion = false;
    let resTermOpt:Option<SentenceDummy> = processInternal(&natural, &mut isQuestion);
    let punct = match isQuestion { // compute punctuation of narsese if it is a question or not
        true => EnumPunctation::QUESTION,
        false => EnumPunctation::JUGEMENT
    };

    if resTermOpt.is_some() {
        let resTerm:&Term = &(*resTermOpt.unwrap().term);
        match resTerm {
            Term::Stmt(Copula::INH, subj, _pred) => { // is relationship
                //let prod0;
                let prod1;
                
                match &**subj {
                    Term::Prod(arr) if arr.len() == 2 => {
                        //prod0 = *arr[0].clone();
                        prod1 = *arr[1].clone();
                    },
                    _ => {
                        // term doesn't fit expected structure!
                        println!("W term from NLP isn't recognized 2!");
                        return;
                    }
                }

                inputT(parentNar, &prod1, punct, &Tv{f:1.0,c:0.9});
            },
            _ => {
                // term doesn't fit expected structure!
                println!("W term from NLP isn't recognized!");
            }
        }
    }
}

struct NlpAnswerHandler {
    answer: Option<SentenceDummy>, // holds the answer if it was found
}

impl QHandler for NlpAnswerHandler {
    fn answer(&mut self, _question:&Term, answer:&SentenceDummy) {
        self.answer = Some(answer.clone());
    }
}