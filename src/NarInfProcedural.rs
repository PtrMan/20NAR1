//! inference rules for procedural derivations

use std::sync::Arc;
use rand::Rng;

use crate::Tv;
use crate::NarStamp;
use crate::Term::*;
use crate::NarSentence::EnumPunctation;
use crate::NarSentence::Sentence;
use crate::NarSentence::retTv;
use crate::NarSentence::newEternalSentenceByTv;
use crate::NarSentence::shallowCopySentence;

/// does inference of goal with a belief
///
/// we need to derive goals from matching implSeqs by goal deduction
/// a =/> b.
/// b!
/// |-dedGoal
/// a!
///
/// returns derivation
pub fn infGoalBelief(goal: &Sentence, belief: &Sentence)-> Option<Sentence> {
    // check if term is same and inference can be done
    match &*belief.term {
        Term::Stmt(Copula::PREDIMPL, _subj, pred) => {
            if !checkEqTerm(&goal.term, &pred) {
                return None; // can't do inference because terms have to be equal
            }
        },
        _ => {
            // don't do anything here
            return None;
        }
    }
    
    if NarStamp::checkOverlap(&goal.stamp, &belief.stamp) {
        return None; // overlap -> can't derive anything
    }

    // a =/> b.
    // b!
    // |-dedGoal
    // a!
    let tvCompound = retTv(&belief).unwrap();
    let tvComponent = retDesire(&goal);
    let tvConcl = Tv::dedGoal(&tvComponent, &tvCompound);
    
    let stamp = NarStamp::merge(&goal.stamp, &belief.stamp);

    match &*belief.term {
        Term::Stmt(Copula::PREDIMPL, subj, _) => {
            return Some(newEternalSentenceByTv(&subj,EnumPunctation::GOAL,&tvConcl,stamp));
        },
        _ => {
            // don't do anything here
            return None;
        }
    }
}

/// var intro for procedural (of sentence)
pub fn matchandintro_var1_sentence(s: &Sentence,  rng: &mut rand::rngs::ThreadRng) -> Vec<Sentence> {
    let mut res = vec![];
    for i_term in &matchandintro_var1(&s.term, rng) {
        let mut i_sentence: Sentence = shallowCopySentence(&s);
        i_sentence.term = Arc::new(i_term.clone());
        res.push(i_sentence);
    }
    res
}

/// var intro for procedural
pub fn matchandintro_var1(s: &Term,  rng: &mut rand::rngs::ThreadRng) -> Vec<Term> {
    let mut concl: Vec<Term> = vec![];

    // match < ( x --> [y] , ... ) =/> ... >  where x will be var
    match &*s {
        Term::Stmt(cop, subj, pred) if *cop == Copula::PREDIMPL => {

            match &**subj {
                Term::Seq(seq) if seq.len() == 2 => {
                    match &*seq[0] {
                        Term::Stmt(cop2, subj2, pred2) if *cop2 == Copula::INH => {
                            
                            // predicate
                            match **pred2 {
                                Term::SetInt(_) => {},
                                _ => {return vec![];}
                            }

                            return intro_vars(s, &subj2, rng);
                        },
                        _ => {
                            return vec![];
                        }
                    }
                },
                _ => {
                    return vec![];
                }
            }

        },
        _ => {
            return vec![]; // can't intro vars
        }
    }

    concl
}

/// helper to introduce variables, counts possible terms and doesn't intro if count < 2
fn intro_vars(t: &Term, repl: &Term,  rng: &mut rand::rngs::ThreadRng) -> Vec<Term> {
    if count_occurence_subterms(t, repl) < 2 {
        return vec![]; // is not worth to itro vars because it occurs only once!
    }

    // helper which does the replacement
    // /param with is the term with which it will be replaced
    fn helper(t: &Term, repl: &Term, with: &Term) -> Term {
        if checkEqTerm(t, repl) {
            return with.clone();
        }

        return match t {
            Term::Stmt(cop, subj, pred) => {
                let res_subj = helper(subj, repl, with);
                let res_pred = helper(pred, repl, with);
                Term::Stmt(*cop, Box::new(res_subj), Box::new(res_pred))
            }
            Term::Name(name) => Term::Name(name.clone()),
            Term::Seq(seq) => {
                let mut arr = vec![];
                for i in seq {
                    arr.push(Box::new(helper(i, repl, with)));
                }
                Term::Seq(arr)
            },
            Term::SetInt(set) => {
                let mut arr = vec![];
                for i in set {
                    arr.push(Box::new(helper(i, repl, with)));
                }
                Term::SetInt(arr)
            },
            Term::SetExt(set) => {
                let mut arr = vec![];
                for i in set {
                    arr.push(Box::new(helper(i, repl, with)));
                }
                Term::SetExt(arr)
            },
            Term::QVar(name) => {
                Term::QVar(name.clone())
            },
            Term::DepVar(name) => {
                Term::DepVar(name.clone())
            },
            Term::IndepVar(name) => {
                Term::IndepVar(name.clone())
            },
            Term::Conj(elements) => {
                let mut arr = vec![];
                for i in elements {
                    arr.push(Box::new(helper(i, repl, with)));
                }
                Term::Conj(arr)
            },
            Term::Prod(elements) => {
                let mut arr = vec![];
                for i in elements {
                    arr.push(Box::new(helper(i, repl, with)));
                }
                Term::Prod(arr)
            },
            Term::Img(rel,idx,elements) => {
                let mut arr = vec![];
                for i in elements {
                    arr.push(Box::new(helper(i, repl, with)));
                }
                Term::Img(Box::new((**rel).clone()),*idx,arr)
            },
            Term::IntInt(elements) => {
                let mut arr = vec![];
                for i in elements {
                    arr.push(Box::new(helper(i, repl, with)));
                }
                Term::IntInt(arr)
            },
            Term::ExtInt(elements) => {
                let mut arr = vec![];
                for i in elements {
                    arr.push(Box::new(helper(i, repl, with)));
                }
                Term::ExtInt(arr)
            },
            Term::Par(seq) => {
                let mut arr = vec![];
                for i in seq {
                    arr.push(Box::new(helper(i, repl, with)));
                }
                Term::Par(arr)
            },
            Term::Neg(term) => {
                Term::Neg(Box::new(helper(term, repl, with)))
            },
        }
    }

    // generate random variable name
    let with: Term = create_random_varname(rng);

    vec![helper(t, repl, &with)] // do actual substitutation
}

fn create_random_varname(rng: &mut rand::rngs::ThreadRng) -> Term {
    let mut name: String = "Q".to_string();
    for _i in 0..24 {
        let v = "012456789ABCDEF".chars().nth(rng.gen_range(0..16)).unwrap();
        name += &v.to_string();
    }
    Term::IndepVar(name)
}



/// goal detachment rule
///
/// ex: (a, b)! |- a!
pub fn infGoalDetach(premise: &Sentence) -> Option<Sentence> {
    // TODO< assert that premise is a goal >

    // * try to do goal detachment
    match &*premise.term {
        Term::Seq(seq) if seq.len() >= 1 => {
            let detachedGoal:Sentence = newEternalSentenceByTv(&seq[0],EnumPunctation::GOAL,&retTv(&premise).unwrap(),premise.stamp.clone());
            //dbg(&format!("detached goal {}", &NarSentence::convSentenceTermPunctToStr(&detachedGoal, true)));
            Some(detachedGoal)
        },
        _ => {None}
    }
}



/// helper
// not PUBLIC because it's such a small helper which shouldn't get exposed
fn retDesire(goal: &Sentence) -> Tv::Tv {
    retTv(&goal).unwrap() // interpret tv as desire
}