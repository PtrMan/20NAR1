// module which implements basic NLP functionality
// 
// implementation spawns a "worker NAR" to process the preprocessed sentence

use std::cell::RefCell;
use std::rc::Rc;

use Nar::*;
use Term::*;
use TermApi::*;
use NarWorkingCycle::{Task2};
use Tv::*;
use NarStamp::newStamp;
use NarSentence::{SentenceDummy, EnumPunctation, Evidence};

pub fn process(natural:&String) {
    let mut workerNar = createNar();

    println!("TODO - convert natural into tokens");
    println!("TODO - convert tokens to inheritance representation and feed into NAR!");

    // relation positive
    //ex:  a dog is a animal
    //ex:  an dog is an animal
    inputN(&mut workerNar, &"<(<{($1*0)} --> a2>&&<{(is*2)} --> rel2>&&<{($2*3)} --> a2>) ==> <{($1*$2)} --> isRel>>. {1.0 0.998}".to_string());

    // ask question directly
    {
        let sentence = SentenceDummy {
            term:Rc::new( s(Copula::INH, &Term::QVar("0".to_string()), &Term::Name("isRel".to_string())) ),
            t:None, // time of occurence 
            punct:EnumPunctation::QUESTION,
            stamp:newStamp(&vec![999]),
            evi:Evidence::TV(Tv{f:1.0,c:0.9}),
            expDt:None
        };

        println!("TODO - add handler");
        workerNar.mem.questionTasks.push(Box::new(Task2 {
            sentence:sentence,
            handler:None,
            bestAnswerExp:0.0, // because has no answer yet
            prio:1.0,
        }));
    }

    for iCycle_ in 0..200 { // give worker NAR time to reason
        cycle(&mut workerNar);
    }


    println!("TODO - process answer to question <?0 --> isRel>?");
}