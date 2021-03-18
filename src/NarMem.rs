// memory system for NAR

use std::sync::{Arc};
use std::collections::HashMap;
use parking_lot::RwLock;

use crate::Term::Term;
use crate::Term::checkEqTerm;
use crate::Term::retSubterms;

use crate::Tv::calcExp;

use crate::NarStamp;

use crate::NarSentence::EnumPunctation;
use crate::NarSentence::Sentence;
use crate::NarSentence::retTv;
use crate::NarSentence::shallowCopySentence;

use crate::NarSentence::calcUsageUsefulness;

/// memory system
pub struct Concept {
    pub name:Term,

    /// beliefs ordered by exp()
    pub beliefsByExp:Vec<Arc<RwLock<Sentence>>>,
    /// beliefs ordered only by usage as in ONA
    pub beliefsByUsage:Vec<Arc<RwLock<Sentence>>>,
}



/// memory
pub struct Mem {
    pub concepts:HashMap<Term, Arc<Concept>>,
}

pub fn make() -> Mem {
    Mem{concepts:HashMap::new(),}
}

pub fn storeInConcepts(mem: &mut Mem, s:&Sentence, nBeliefs: usize, currentTime: i64) {
    storeInConcepts2(mem, s, &retSubterms(&*s.term), nBeliefs, currentTime); // enumerate all terms, we need to do this to add the sentence to all relevant names
}

/// function is a indirection for more control over which subterms are used for storage
pub fn storeInConcepts2(mem: &mut Mem, s:&Sentence, subterms: &Vec<Term>, nBeliefs: usize, currentTime: i64) {
    if s.punct != EnumPunctation::JUGEMENT {
        return; // ignore everything else than JUGEMENT
    }
    
    for iTerm in subterms {
        match mem.concepts.get_mut(&iTerm.clone()) {
            Some(arcConcept) => {
                match Arc::get_mut(arcConcept) {
                    Some(concept) => {
                        { // beliefs by exp
                            let mut exists = false;
                            for iBelief in &concept.beliefsByExp {
                                let iBeliefGuard = iBelief.read();
                                if checkEqTerm(&iBeliefGuard.term, &s.term) && NarStamp::checkOverlap(&iBeliefGuard.stamp, &s.stamp) {
                                    exists = true;
                                    break; // OPT
                                }
                            }
                            
                            if !exists { // add belief only if it doesn't already exist!
                                concept.beliefsByExp.push(Arc::new(RwLock::new(shallowCopySentence(&(*s))))); // add belief
    
                                // order by importance
                                let mut temp:Vec<(f64,Arc<RwLock<Sentence>>)> = concept.beliefsByExp.iter().map(|iv| {
                                        let ivGuard = iv.read();
                                        (calcExp(&retTv(&ivGuard).unwrap()), Arc::clone(iv))
                                    }).collect(); // compute exp for each element, necessary because else we have a deadlock
                                temp.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap()); // do actual sorting by exp
                                concept.beliefsByExp = temp.iter().map(|v| Arc::clone(&v.1)).collect(); // extract Arc back
                                
                                // keep under AIKR
                                concept.beliefsByExp = concept.beliefsByExp[..concept.beliefsByExp.len().min(nBeliefs)].to_vec();
                            }
                        }

                        { // beliefs by usage
                            let mut exists = false;
                            for iBelief in &concept.beliefsByUsage {
                                let iBeliefGuard = iBelief.read();
                                if checkEqTerm(&iBeliefGuard.term, &s.term) && NarStamp::checkOverlap(&iBeliefGuard.stamp, &s.stamp) {
                                    exists = true;
                                    break; // OPT
                                }
                            }
                            
                            if !exists { // add belief only if it doesn't already exist!
                                concept.beliefsByUsage.push(Arc::new(RwLock::new(shallowCopySentence(&(*s))))); // add belief
    
                                // order by importance
                                let mut temp:Vec<(f64,Arc<RwLock<Sentence>>)> = concept.beliefsByUsage.iter().map(|iv| {
                                        let ivGuard = iv.read();
                                        let usage: f64 = calcUsageUsefulness(&(*ivGuard).usage.read(), currentTime);
                                        (usage, Arc::clone(iv))
                                    }).collect(); // compute usage for each element, necessary because else we have a deadlock
                                temp.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap()); // do actual sorting by exp
                                concept.beliefsByUsage = temp.iter().map(|v| Arc::clone(&v.1)).collect(); // extract Arc back
                                
                                // keep under AIKR
                                concept.beliefsByUsage = concept.beliefsByUsage[..concept.beliefsByUsage.len().min(nBeliefs)].to_vec();
                            }
                        }
                        
                    }
                    None => {
                        println!("INTERNAL ERROR - couldn't aquire arc!");
                    }
                }
            },
            None => { // concept doesn't exist
                // * insert new concept if we are here
                
                let concept = Arc::new(Concept {
                    name:iTerm.clone(),
                    beliefsByExp:vec![Arc::new(RwLock::new(shallowCopySentence(&(*s))))],
                    beliefsByUsage:vec![Arc::new(RwLock::new(shallowCopySentence(&(*s))))],
                });
                
                mem.concepts.insert(iTerm.clone(), concept); // add concept to memory
            }
        }
    }
}


/// limit size of memory
pub fn limitMemory(mem: &mut Mem, nConcepts: usize) {
    if mem.concepts.len() <= nConcepts {
        return; // not enough concepts to limit
    }

    let mut concepts: Vec<(Arc<Concept>, f64)> = Vec::new(); // concept with rating
    // scan concepts
    for (_key, mut iConcept) in &mut mem.concepts {
        let mut rating:f64 = 0.0;
        match Arc::get_mut(&mut iConcept) {
            Some(concept) => {
                for iBelief in &concept.beliefsByExp {
                    let iBeliefGuard = iBelief.read();
                    rating = rating.max(calcExp(&retTv(&iBeliefGuard).unwrap()));
                }
            }
            None => {
                println!("INTERNAL ERROR - couldn't aquire arc!");
            }
        }

        concepts.push((Arc::clone(&iConcept), rating));
    }
    mem.concepts.clear();

    // sort
    concepts.sort_by(|(_, aRating), (_, bRating)| bRating.partial_cmp(aRating).unwrap());

    // limit
    concepts = concepts[..concepts.len().min(nConcepts)].to_vec();

    // put back
    for (iConcept, _rating) in &concepts {
        let name:Term = iConcept.name.clone();
        mem.concepts.insert(name.clone(), Arc::clone(&iConcept));
    }
}

/// return beliefs of concept by term
///
/// doesn't examine memory for subterms!
pub fn ret_beliefs_of_concept<'a>(mem: &'a Mem, selTerm: &'a Term) -> Option< std::iter::Chain<std::slice::Iter<'a, Arc<parking_lot::lock_api::RwLock<parking_lot::RawRwLock, crate::NarSentence::Sentence>>>, std::slice::Iter<'a, Arc<parking_lot::lock_api::RwLock<parking_lot::RawRwLock, crate::NarSentence::Sentence>>>> > {
    match mem.concepts.get(&selTerm) {
        Some(concept) => {
            Some(concept.beliefsByExp.iter().chain(
                concept.beliefsByUsage.iter()
            ))

            //concept.beliefsByExp.iter().map(|iv| Arc::clone(iv)).collect().iter().chain(
            //    concept.beliefsByUsage.iter().map(|iv| Arc::clone(iv)).collect().iter()
            //)

            /*
            let res = concept.beliefsByExp.iter().map(|iv| Arc::clone(iv)).collect();

            println!("TODO - add beliefsByUsage");

            res*/
        },
        None => { // concept doesn't exist
            //core::iter::empty().chain(core::iter::empty())
            None
        }
    } 
}

/// return non-unique beliefs by terms and it's subterms
pub fn ret_beliefs_by_terms_nonunique(narMem:&Mem, terms:&[Term]) -> Vec<Arc<RwLock<Sentence>>> {
    let mut res:Vec<Arc<RwLock<Sentence>>> = vec![];
    for iTerm in terms {
        for isubterm in &retSubterms(&iTerm) { // we have to iterate over term and subterm, ex: a-->b   ===> a, b, a-->b
            let beliefsOfConceptOpt = ret_beliefs_of_concept(narMem, &isubterm);

            match beliefsOfConceptOpt {
                Some(beliefsOfConcept) => {
                    // add to result
                    for iBelief in beliefsOfConcept {
                        res.push(Arc::clone(iBelief));
                    }
                },
                None => {}
            }
        }
    }
    
    res
}
