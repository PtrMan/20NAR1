use crate::Tv::*;
use crate::TvVec::*;

// function to prototype vision
pub fn prototypeV2() {
    { // prototyping: compute similarity between vectors

        let aVec = vec![Tv{f:1.0, c:0.1}, Tv{f:0.0, c:0.1}];
        let bVec = vec![Tv{f:0.0, c:0.1}, Tv{f:1.0, c:0.1}];
        let tvSim: Tv = foldVec(&compVec(&aVec, &bVec));
    
        let sim: f64 = calcExp(&tvSim); // we interpret expectation as similarity of the TV-vectors
    
        println!("comp sim = {}", sim);    
    }

    let mut cls: PrototypeClassifier = PrototypeClassifier{prototypes:vec![]};
    { // fill prototypes with test-prototyping for prototyping
        {
            let aVec = vec![Tv{f:1.0, c:0.1}, Tv{f:0.0, c:0.1}, Tv{f:0.0, c:0.1}];
            cls.prototypes.push(Prototype{v:aVec});
        }
    }

    { // classify stimulus
        let stimulus_conf: f64 = 0.05; // confidence of the perceived stimulus
        let stimulus: Vec<Tv> = vec![Tv{f:1.0, c:stimulus_conf}, Tv{f:0.0, c:stimulus_conf}, Tv{f:0.0, c:stimulus_conf}];

        let prototypes_sim: Vec<f64> = calc_sims(&stimulus, &cls);
        println!("{:?}", prototypes_sim);

        // do actual classification
        // TODO< implement >
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
