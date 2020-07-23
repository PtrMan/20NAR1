use std::rc::Rc;

use Term::Term;

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
}

