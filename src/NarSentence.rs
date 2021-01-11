use std::sync::{Arc};

use crate::Term::Term;
use crate::Tv;
use crate::Term::convTermToStr;
use crate::NarStamp::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum EnumPunctation {
    JUGEMENT, // .
    QUESTION, // ?
    GOAL, // !
}

// abstraction for evidence
// we need it because we are using AERA inspired TV for procedural knowledge
#[derive(Clone)]
pub enum Evidence {
    CNT{pos:i64,cnt:i64}, // count based evidence
    TV(Tv::Tv),
}

#[derive(Clone)]
pub struct Sentence {
    pub term:Arc<Term>,
    pub t:Option<i64>, // time of occurence 
    pub punct:EnumPunctation,
    pub stamp:Stamp,

    pub expDt:Option<i64>, // exponential time delta, used for =/>

    pub evi:Option<Evidence>, // option because questions don't have tv!
}

// create new eternal sentence
pub fn newEternalSentenceByTv(term:&Term,punct:EnumPunctation,tv:&Tv::Tv,stamp:Stamp)->Sentence {
    Sentence {
        term:Arc::new(term.clone()),
        t:None, // time of occurence 
        punct:punct,
        stamp:stamp,
        evi:if punct != EnumPunctation::QUESTION {Some(Evidence::TV(tv.clone()))} else {None},
        expDt:None, // not used
    }
}

pub fn retTv(s:&Sentence)->Option<Tv::Tv> {
    if !s.evi.is_some() {
        return None;
    }
    
    match &s.evi.as_ref().unwrap() {
        Evidence::TV(tv) => {Some(tv.clone())},
        Evidence::CNT{pos: _,cnt: _} => {Some(Tv::Tv{f:retFreq(&s.evi.as_ref().unwrap()),c:retConf(&s.evi.as_ref().unwrap())})} // need to compute evidence
    }
}


pub fn retFreq(evidence:&Evidence)->f64 {
    match evidence {
        Evidence::CNT{pos,cnt} => {(*pos as f64) / (*cnt as f64)}
        _ => {panic!("expected CNT")},
    }
}

pub fn retConf(evidence:&Evidence)->f64 {
    match evidence {
        Evidence::CNT{pos:_,cnt} => {(*cnt as f64) / ((*cnt as f64) + 1.0)}
        _ => {panic!("expected CNT")},
    }
}

// convert only term and punctation to string
pub fn convSentenceTermPunctToStr(s:&Sentence, enTv:bool) -> String {
    let punct = match s.punct{
        EnumPunctation::QUESTION=>"?",
        EnumPunctation::JUGEMENT=>".",
        EnumPunctation::GOAL=>"!",
    };
    let mut res = convTermToStr(&s.term) + punct;
    if enTv && s.punct != EnumPunctation::QUESTION {
        res = res + " " + &Tv::convToStr(&retTv(&s).unwrap());
    }
    res
}
