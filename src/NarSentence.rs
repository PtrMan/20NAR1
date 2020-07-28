use std::rc::Rc;

use Term::Term;
use Tv::*;
use Term::convTermToStr;
use NarStamp::*;

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
    TV(Tv),
}

#[derive(Clone)]
pub struct SentenceDummy {
    pub term:Rc<Term>,
    pub t:Option<i64>, // time of occurence 
    pub punct:EnumPunctation,
    pub stamp:Stamp,

    pub expDt:Option<i64>, // exponential time delta, used for =/>

    pub evi:Evidence,
}

// create new eternal sentence
pub fn newEternalSentenceByTv(term:&Term,punct:EnumPunctation,tv:&Tv,stamp:Stamp)->SentenceDummy {
    SentenceDummy {
        term:Rc::new(term.clone()),
        t:None, // time of occurence 
        punct:punct,
        stamp:stamp,
        evi:Evidence::TV(tv.clone()),
        expDt:None, // not used
    }
}

pub fn retTv(s:&SentenceDummy)->Tv {
    match &s.evi {
        Evidence::TV(tv) => {tv.clone()},
        Evidence::CNT{pos,cnt} => {Tv{f:retFreq(&s.evi),c:retConf(&s.evi)}} // need to compute evidence
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
        Evidence::CNT{pos,cnt} => {(*cnt as f64) / ((*cnt as f64) + 1.0)}
        _ => {panic!("expected CNT")},
    }
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
