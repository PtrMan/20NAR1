// module which implements basic NLP functionality
// 
// implementation uses a NN to convert the "natural sentence" to "relation"s which get converted to narsese

use crate::Nar::*;
use crate::Term::*;
use crate::NarWorkingCycle::{Task2, debugCreditsOfTasks, QHandler};
use crate::Tv::*;
use crate::NarSentence::{Sentence, EnumPunctation};
use crate::ModNlpA;


// /param parentNar is the NAR to which the beliefs and questions will be fed
pub fn process(parentNar: &mut Nar, natural:&String) {
    let mut natural: String = natural.clone();
    natural = natural.replace("?", " ? "); // treat "?" as token
    let mut tokens: Vec<&str> = natural.split_whitespace().collect(); // split into tokens

    // convert from &str to string
    let mut tokens2: Vec<String> = vec![];
    for i_token in tokens {
        tokens2.push(i_token.to_string());
    }
    
    let (control_seq, relations) = ModNlpA::network_run(&tokens2);
    
    // convert relations to narsese and put it into NAR
    for i_relation in &relations {
        let i_narsese:String = ModNlpA::conv_rel_to_narsese(i_relation);
        inputN(parentNar, &i_narsese);
    }
}
