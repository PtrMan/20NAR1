use std::sync::{Arc};
use parking_lot::RwLock;

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

    pub usage:Arc<RwLock<Usage>>,
}

pub fn shallowCopySentence(s: &Sentence)->Sentence {
    Sentence {
        term:Arc::clone(&s.term),
        t:s.t,
        punct:s.punct,
        stamp:s.stamp.clone(),
        expDt:s.expDt,
        evi:s.evi.clone(),
        usage:Arc::clone(&s.usage),
    }
}

/*
pub fn deepCopySentence(s: &Sentence)->Sentence {
    Sentence {
        term:Arc::clone(&s.term),
        t:s.t,
        punct:s.punct,
        stamp:s.stamp.clone(),
        expDt:s.expDt,
        evi:s.evi.clone(),
        usage:Arc::new(&s.usage.clone()),
    }
}
*/

#[derive(Clone)]
pub struct Usage {
    pub lastUsed: i64, // time
    pub useCount: i64,
}

pub fn usageUpdate(usage: &mut Usage, currentTime: i64) {
    usage.lastUsed = currentTime;
    usage.useCount += 1;
}

// see https://github.com/opennars/OpenNARS-for-Applications/blob/eeeb2ce1cc029f56f3b5eaf27fdf51f93b42a889/src/Usage.c#L27-L32
pub fn calcUsageUsefulness(usage: &Usage, currentTime: i64) -> f64 {
    let recency: f64 = (currentTime - usage.lastUsed).max(0) as f64; // max(0) to prevent underflow!
    let usefulnessToNormalize: f64 = (usage.useCount as f64) / (recency + 1.0);
    let result:f64 = usefulnessToNormalize / (usefulnessToNormalize + 1.0);
    //println!("[d99] recn={} result={}", recency, result);
    result
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
        usage:Arc::new(RwLock::new(Usage{lastUsed:0, useCount:0})),
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
