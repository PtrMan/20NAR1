// memory system for NAR

use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;

use crate::Term::Term;
use crate::Term::checkEqTerm;
use crate::Term::retSubterms;

use crate::Tv::calcExp;

use crate::NarStamp;

use crate::NarSentence::EnumPunctation;
use crate::NarSentence::SentenceDummy;
use crate::NarSentence::retTv;

// memory system
pub struct Concept {
    pub name:Rc<Term>,

    pub beliefs:Vec<Arc<SentenceDummy>>, // =/> beliefs
}

// memory
pub struct Mem {
    pub concepts:HashMap<Term, Arc<Concept>>,
}

pub fn storeInConcepts(mem: &mut Mem, s:&SentenceDummy) {
    if s.punct != EnumPunctation::JUGEMENT {
        return; // ignore everything else than JUGEMENT
    }
    
    for iTerm in retSubterms(&*s.term) { // enumerate all terms, we need to do this to add the sentence to all relevant names
        match mem.concepts.get_mut(&iTerm.clone()) {
            Some(arcConcept) => {
                match Arc::get_mut(arcConcept) {
                    Some(concept) => {
                        let mut exists = false;
                        for iBelief in &concept.beliefs {
                            if checkEqTerm(&iBelief.term, &s.term) && NarStamp::checkOverlap(&iBelief.stamp, &s.stamp) {
                                exists = true;
                                break; // OPT
                            }
                        }
                        
                        if !exists { // add belief only if it doesn't already exist!
                            concept.beliefs.push(Arc::new((*s).clone())); // add belief

                            println!("TODO - order by importance");

                            concept.beliefs = concept.beliefs[..concept.beliefs.len().min(20)].to_vec(); // keep under AIKR
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
                    name:Rc::new(iTerm.clone()),
                    beliefs:vec![Arc::new((*s).clone())],
                });
                
                mem.concepts.insert(iTerm.clone(), concept); // add concept to memory
            }
        }
    }
}

// limit size of memory
pub fn limitMemory(mem: &mut Mem, nConcepts: usize) {
    let mut concepts: Vec<(Arc<Concept>, f64)> = Vec::new(); // concept with rating
    // scan concepts
    for (key, mut iConcept) in &mut mem.concepts {
        let mut rating:f64 = 0.0;
        match Arc::get_mut(&mut iConcept) {
            Some(concept) => {
                for iBelief in &concept.beliefs {
                    rating = rating.max(calcExp(&retTv(&iBelief).unwrap()));
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
        let name:Term = (*iConcept.name).clone();
        mem.concepts.insert(name.clone(), Arc::clone(&iConcept));
    }
}