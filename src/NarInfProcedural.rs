//! inference rules for procedural derivations

use crate::Tv;
use crate::NarStamp;
use crate::Term::*;
use crate::NarSentence::EnumPunctation;
use crate::NarSentence::Sentence;
use crate::NarSentence::retTv;
use crate::NarSentence::newEternalSentenceByTv;

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