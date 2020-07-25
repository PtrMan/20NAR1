use std::rc::Rc;

use Term::Term;
use Tv::*;
use Term::convTermToStr;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum EnumPunctation {
    JUGEMENT, // .
    QUESTION, // ?
    GOAL, // !
}

#[derive(Clone)]
pub struct SentenceDummy {
    pub isOp:bool, // is it a operation?
    pub term:Rc<Term>,
    pub t:i64, // time of occurence 
    pub punct:EnumPunctation,
    //pub stamp:Stamp,
    pub tv:Tv,
}


// convert only term and punctation to string
pub fn convSentenceTermPunctToStr(s:&SentenceDummy) -> String {
    let punct = match s.punct{
        EnumPunctation::QUESTION=>"?",
        EnumPunctation::JUGEMENT=>".",
        EnumPunctation::GOAL=>"!",
    };    
    convTermToStr(&s.term) + punct
}
