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

    let mut tokens: Vec<&str> = natural.split_whitespace().collect(); // split into tokens

    if tokens.len() > 0 && tokens[tokens.len()-1] == "?" { // it is a question if it ends with a question-mark
        *isQuestion = true;
        tokens = tokens[..tokens.len()-1].to_vec();// cut question mark away
    }

    let mut workerNar = createNar();

    // convert tokens to inheritance representation and feed into NAR
    {
        let mut idx:usize = 0;
        while idx < tokens.len() {
            let idxAsStr = format!("{}", idx);
            
            if (tokens[idx] == "a" || tokens[idx] == "an") && idx+1 < tokens.len() {
                let token2nd = tokens[idx+1];
                let term:Term = s(Copula::INH, &Term::SetExt(vec![Box::new(p2(&Term::Name(token2nd.to_string()), &Term::Name(idxAsStr)))]), &Term::Name("a2".to_string()));
                inputT(&mut workerNar, &term, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.998});

                idx+=2;
            }
            else if tokens[idx] == "is" || tokens[idx] == "can" {
                if tokens[idx] == "is" {
                    let term:Term = s(Copula::INH, &Term::SetExt(vec![Box::new(Term::Name(idxAsStr))]), &Term::Name("TOKEN".to_string() + tokens[idx]));
                    inputT(&mut workerNar, &term, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.998});
                }
                else {
                    let term:Term = s(Copula::INH, &Term::SetExt(vec![Box::new(p2(&Term::Name(tokens[idx].to_string()), &Term::Name(idxAsStr.clone())))]), &Term::Name("rel2".to_string()));
                    inputT(&mut workerNar, &term, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.998});    
                }

                idx+=1;
            }
            else if tokens[idx] == "and" {
                let term:Term = s(Copula::INH, &Term::SetExt(vec![Box::new(Term::Name(idxAsStr))]), &Term::Name("TOKENand".to_string()));
                inputT(&mut workerNar, &term, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.998});

                idx+=1;
            }
            else if tokens[idx] == "?" { // we need special handling for special characters
                let term:Term = s(Copula::INH, &Term::SetExt(vec![Box::new(p2(&Term::Name("QUESTION".to_string()), &Term::Name(idxAsStr)))]), &Term::Name("sign2".to_string()));
                inputT(&mut workerNar, &term, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.998});

                idx+=1;
            }
            else { // raw token
                let term:Term = s(Copula::INH, &Term::SetExt(vec![Box::new(p2(&Term::Name(tokens[idx].to_string().clone()), &Term::Name(idxAsStr)))]), &Term::Name("TOKEN".to_string()));
                inputT(&mut workerNar, &term, EnumPunctation::JUGEMENT, &Tv{f:1.0,c:0.998});

                idx+=1;
            }
        }
    }


    // combine tokens for "and"
    // ex: b and c
    inputN(&mut workerNar, &"<(<{3} --> TOKENand>&&<{($0*2)} --> TOKEN>&&<{($1*4)} --> TOKEN>) ==> <{(($0|$1)*2)} --> AT2>>. {1.0 0.998}".to_string());
    // for x y z is b and c
    inputN(&mut workerNar, &"<(<{5} --> TOKENand>&&<{($0*4)} --> TOKEN>&&<{($1*6)} --> TOKEN>) ==> <{(($0|$1)*4)} --> AT2>>. {1.0 0.998}".to_string());

    // for beginning of phrase
    // ex "x and y ..."
    inputN(&mut workerNar, &"<(<{1} --> TOKENand>&&<{($0*0)} --> TOKEN>&&<{($1*2)} --> TOKEN>) ==> <{(($0|$1)*2)} --> AT2>>. {1.0 0.998}".to_string());


    // instance relation positive
    //ex:  tom is a cat
    //commented because we need to overhaul "a cat"
    //inputN(&mut workerNar, &"<(<{($1*0)} --> TOKEN>&&<{(is*1)} --> rel2>&&<{($2*2)} --> a2>) ==> <{({$1}*$2)} --> relIs>>. {1.0 0.998}".to_string());

    // relation positive
    //ex:  a dog is a animal
    //ex:  an dog is an animal
    //commented because we need to overhaul "a cat"
    //inputN(&mut workerNar, &"<(<{($1*0)} --> a2>&&<{(is*2)} --> rel2>&&<{($2*3)} --> a2>) ==> <{($1*$2)} --> relIs>>. {1.0 0.998}".to_string());





    // ex: tom is fat
    //<{($0*0)} --> TOKEN>
    //<{($rel*1)} --> rel2>
    //<{($1*2)} --> TOKEN>
    //==>
    //<{($0*$1*$rel)} --> relGENERIC>
    inputN(&mut workerNar, &"<(<{($0*0)} --> TOKEN>&&<{1} --> TOKENis>&&<{($1*2)} --> TOKEN>) ==> <{({$0}*$1*is)} --> relGENERIC>>. {1.0 0.998}".to_string());
    inputN(&mut workerNar, &"<(<{($0*0)} --> TOKEN>&&<{($rel*1)} --> rel2>&&<{($1*2)} --> TOKEN>) ==> <{($0*$1*$rel)} --> relGENERIC>>. {1.0 0.998}".to_string());


    // ex: tom and tim is fat
    //<{($0*2)} --> AT2>
    //<{($rel*3)} --> rel2>
    //<{($1*4)} --> TOKEN>
    //==>
    //<{($0*$1*$rel)} --> relGENERIC>
    inputN(&mut workerNar, &"<(<{($0*2)} --> AT2>&&<{3} --> TOKENis>&&<{($1*4)} --> TOKEN>) ==> <{({$0}*$1*is)} --> relGENERIC>>. {1.0 0.998}".to_string());
    inputN(&mut workerNar, &"<(<{($0*2)} --> AT2>&&<{($rel*3)} --> rel2>&&<{($1*4)} --> TOKEN>) ==> <{($0*$1*$rel)} --> relGENERIC>>. {1.0 0.998}".to_string());

    // ex: tom and tim is fat and lazy
    //<{($0*2)} --> AT2>
    //<{($rel*3)} --> rel2>
    //<{($1*4)} --> AT2>
    //==>
    //<{($0*$1*$rel)} --> relGENERIC>
    inputN(&mut workerNar, &"<(<{($0*2)} --> AT2>&&<{3} --> TOKENis>&&<{($1*4)} --> AT2>) ==> <{($0*$1*is)} --> relGENERIC>>. {1.0 0.998}".to_string());
    inputN(&mut workerNar, &"<(<{($0*2)} --> AT2>&&<{($rel*3)} --> rel2>&&<{($1*4)} --> AT2>) ==> <{($0*$1*$rel)} --> relGENERIC>>. {1.0 0.998}".to_string());



    
    // ex: tom is fat and sick
    //<{($0*0)} --> TOKEN>
    //<{1} --> TOKENis>
    //<{($1*2)} --> AT2>
    //==>
    //<{($0*$1)} --> relIs2>
    inputN(&mut workerNar, &"<(<{($0*0)} --> TOKEN>&&<{1} --> TOKENis>&&<{($1*2)} --> AT2>) ==> <{({$0}*$1*is)} --> relGENERIC>>. {1.0 0.998}".to_string());
    inputN(&mut workerNar, &"<(<{($0*0)} --> TOKEN>&&<{($rel*1)} --> rel2>&&<{($1*2)} --> AT2>) ==> <{($0*$1*$rel)} --> relGENERIC>>. {1.0 0.998}".to_string());









    // relation negative
    //ex:  a dog isn't a animal
    //ex:  an dog isn't an animal
    println!("TODO - implement parsing of negation!");
    println!("TODO - add this negation rule");
    //inputN(&mut workerNar, &"<(<{($1*0)} --> a2>&&<{(isn_t*2)} --> rel2>&&<{($2*3)} --> a2>) ==> (--,<{($1*$2)} --> isRel>)>. {1.0 0.998}");




    // query for a relation
    // ex:    is a dog a animal ?
    inputN(&mut workerNar, &"<(<{(is*0)} --> rel2>&&<{($1*1)} --> a2>&&<{($2*3)} --> a2>&&<{(QUESTION*5)} --> sign2>) ==> <{($1*$2)} --> relIsQuery>>. {1.0 0.998}".to_string());



    // ask question directly
    let answerHandler0:NlpAnswerHandler = NlpAnswerHandler{answer:None};
    let answerHandlerRef0 = Rc::new(RefCell::new(answerHandler0));
    let rc0 = Rc::clone(&answerHandlerRef0);
    {
        let sentence = SentenceDummy {
            term:Rc::new( s(Copula::INH, &Term::QVar("0".to_string()), &Term::Name("relGENERIC".to_string())) ),
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

    let answerHandler1:NlpAnswerHandler = NlpAnswerHandler{answer:None};
    let answerHandlerRef1 = Rc::new(RefCell::new(answerHandler1));
    let rc1 = Rc::clone(&answerHandlerRef1);
    {
        let sentence = SentenceDummy {
            term:Rc::new( s(Copula::INH, &Term::QVar("0".to_string()), &Term::Name("relIsQuery".to_string())) ),
            t:None, // time of occurence 
            punct:EnumPunctation::QUESTION,
            stamp:newStamp(&vec![999]),
            evi:None,
            expDt:None
        };

        workerNar.mem.questionTasks.push(Box::new(Task2 {
            sentence:sentence,
            handler:Some(answerHandlerRef1),
            bestAnswerExp:0.0, // because has no answer yet
            prio:1.0,
        }));
    }

    let answerHandler2:NlpAnswerHandler = NlpAnswerHandler{answer:None};
    let answerHandlerRef2 = Rc::new(RefCell::new(answerHandler2));
    let rc2 = Rc::clone(&answerHandlerRef2);
    {
        let sentence = SentenceDummy {
            term:Rc::new( s(Copula::INH, &Term::QVar("0".to_string()), &Term::Name("relIs2".to_string())) ),
            t:None, // time of occurence 
            punct:EnumPunctation::QUESTION,
            stamp:newStamp(&vec![999]),
            evi:None,
            expDt:None
        };

        workerNar.mem.questionTasks.push(Box::new(Task2 {
            sentence:sentence,
            handler:Some(answerHandlerRef2),
            bestAnswerExp:0.0, // because has no answer yet
            prio:1.0,
        }));
    }

    for _iCycle_ in 0..300 { // give worker NAR time to reason
        cycle(&mut workerNar);
    }

    // for debugging
    debugCreditsOfTasks(&workerNar.mem);

    // return answer of question
    
    let res2 = rc2.borrow_mut().answer.clone(); // first because it has a higher "priority" to answer
    if res2.is_some() {
        return res2;
    }
    let res0 = rc0.borrow_mut().answer.clone();
    if res0.is_some() {
        return res0;
    }
    let res1 = rc1.borrow_mut().answer.clone();
    if res1.is_some() {
        return res1;
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