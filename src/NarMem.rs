// memory system for NAR

use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;

use Term::Term;
use Term::Copula;
use Term::retSubterms;
use Term::calcComplexity;

use NarSentence::SentenceDummy;

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
    for iTerm in retSubterms(&*s.term) { // enumerate all terms, we need to do this to add the sentence to all relevant names
        match mem.concepts.get_mut(&iTerm.clone()) {
            Some(arcConcept) => {
                match Arc::get_mut(arcConcept) {
                    Some(concept) => {
                        println!("TODO - add belief only if it doesn't already exist!");

                        concept.beliefs.push(Arc::new((*s).clone())); // add belief

                        // TODO< order by importance >

                        concept.beliefs = concept.beliefs[..concept.beliefs.len().min(20)].to_vec(); // keep under AIKR
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
