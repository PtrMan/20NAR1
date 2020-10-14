// module which implements basic NLP functionality
// 
// implementation spawns a "worker NAR" to process the preprocessed sentence

use std::cell::RefCell;
use std::rc::Rc;

use crate::Nar::*;
use crate::Term::*;
use crate::TermApi::*;
use crate::NarWorkingCycle::{Task2, debugCreditsOfTasks, QHandler};
use crate::Tv::*;
use crate::NarStamp::newStamp;
use crate::NarSentence::{SentenceDummy, EnumPunctation, Evidence};

pub fn process(natural:&String, isQuestion:&mut bool)->Option<SentenceDummy> {
    *isQuestion = false;

    let nCycles = 0; // number of reasoning cycles for "worker NAR"

    let mut tokens: Vec<&str> = natural.split_whitespace().collect(); // split into tokens

    if tokens.len() > 0 && tokens[tokens.len()-1] == "?" { // it is a question if it ends with a question-mark
        *isQuestion = true;
        tokens = tokens[..tokens.len()-1].to_vec();// cut question mark away
    }

    let mut workerNar = createNar();

    //let mut relationWords = Vec::new(); // words which describe relations and thus have a special meaning
    // TODO< load relation words from file >
    //relationWords.push("is");
    //relationWords.push("can");
    //relationWords.push("in");
    //relationWords.push("of");
    //
    //relationWords.push("need");


    
    {
        // feed unknown words into NAR
        for iToken in &tokens {
            if false {}
            //else if *iToken == "a" || *iToken == "an") {} 
            else if  *iToken == "is" {} // ignore special token
            else if  *iToken == "are" {} // ignore special token
            else {
                let term:Term = s(Copula::INH, &Term::SetExt( vec![Box::new(p2(&Term::Name(format!("WORDEN{}", iToken)), &Term::Name((&iToken).to_string())))]), &Term::Name("RELrepresent".to_string()));
                inputT(&mut workerNar, &term, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.998});
            }
            
        }
    }


    // ask question directly
    let answerHandler0:NlpAnswerHandler = NlpAnswerHandler{answer:None};
    let answerHandlerRef0 = Rc::new(RefCell::new(answerHandler0));
    let rc0 = Rc::clone(&answerHandlerRef0);
    {
        let sentence = SentenceDummy {
            term:Rc::new( s(Copula::INH, &p2(&Term::SetExt(vec![Box::new(p3(  &Term::Name("WORDENtom".to_string()), &Term::Name("WORDENis".to_string()), &Term::Name("WORDENfat".to_string())   ))]), &Term::QVar("0".to_string())), &Term::Name("RELrepresent".to_string())) ),
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

struct NlpAnswerHandler {
    answer: Option<SentenceDummy>, // holds the answer if it was found
}

impl QHandler for NlpAnswerHandler {
    fn answer(&mut self, _question:&Term, answer:&SentenceDummy) {
        self.answer = Some(answer.clone());
    }
}