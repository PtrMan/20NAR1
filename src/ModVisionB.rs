use crate::Tv::*;
use crate::TvVec::*;

// function to prototype vision
pub fn prototypeV2() {
    if false { // prototyping: compute similarity between vectors

        let aVec = vec![Tv{f:1.0, c:0.1}, Tv{f:0.0, c:0.1}];
        let bVec = vec![Tv{f:0.0, c:0.1}, Tv{f:1.0, c:0.1}];
        let tvSim: Tv = foldVec(&compVec(&aVec, &bVec));
    
        let sim: f64 = calcExp(&tvSim); // we interpret expectation as similarity of the TV-vectors
    
        println!("comp sim = {}", sim);    
    }

    let mut cls: PrototypeClassifier = PrototypeClassifier{prototypes:vec![]};
    { // fill prototypes with test-prototyping for prototyping
        /*{
            let aVec = vec![Tv{f:1.0, c:0.1}, Tv{f:0.0, c:0.1}, Tv{f:0.0, c:0.1}];
            cls.prototypes.push(Prototype{v:aVec});
        }
         */
    }

    { // classify stimulus
        let stimulus_conf: f64 = 0.05; // confidence of the perceived stimulus
        let stimulus: Vec<Tv> = vec![Tv{f:1.0, c:stimulus_conf}, Tv{f:0.0, c:stimulus_conf}, Tv{f:0.0, c:stimulus_conf}];

        let prototypes_sim: Vec<f64> = calc_sims(&stimulus, &cls);
        println!("{:?}", prototypes_sim);

        // do actual classification
        match classify_max(&prototypes_sim) {
            Some((idx, sim)) => {
                println!("[d ] found prototype!");
                
                println!("TODO - revise evidence!");
                println!("TODO - update time!");
            },
            None => {
                // no match -> create new prototype
                println!("[d ] create new prototype");
                cls.prototypes.push(Prototype{v:stimulus.clone()});
            }
        }
    }
}

/// learned prototype
pub struct Prototype {
    /// TV vector of evidence
    pub v: Vec<Tv>,
    // time of last use, used for forgetting policy
    //pub lastUseTime: i64,
}

/// classifier based on prototypes
pub struct PrototypeClassifier {
    pub prototypes: Vec<Prototype>,
}

pub fn calc_sims(stimulus: &Vec<Tv>, cls: &PrototypeClassifier) -> Vec<f64> {
    let mut prototypes_sim: Vec<f64> = vec![]; // similarities of prototypes to stimulus

    // compute similarities of all prototypes
    for i_prototype in &cls.prototypes {
        let tvSim: Tv = foldVec(&compVec(&stimulus, &i_prototype.v));        
        let sim: f64 = calcExp(&tvSim); // we interpret expectation as similarity of the TV-vectors
        
        prototypes_sim.push(sim);
    }

    prototypes_sim
}

pub fn classify_max(prototypes_sim: &[f64]) -> Option<(i64, f64)> {
    if prototypes_sim.len() == 0 {
        return None;
    }

    let mut max_sim = 0.0;
    let mut max_idx = 0;

    let mut idx = 0;
    for iv in prototypes_sim {
        if *iv > max_sim {
            max_sim = *iv;
            max_idx = idx;
        }
        idx+=1;
    }

    if max_sim < 0.5 {
        return None; // no positive match -> no match at all
    }

    Some((max_idx, max_sim))
}