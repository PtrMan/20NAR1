// vision module : prototype based real vector classifier

use crate::Map2d::*;

/// channel of visual information
#[derive(Clone)]
pub struct Channel {
    /// evidence
    pub v: Map2d<f64>,
}

/// learned prototype
pub struct RealPrototype {
    /// channels of actual prototype image
    pub channels: Vec<Channel>,
    /// time of last use, used for forgetting policy
    pub last_use_time: i64,

    pub evidence_count: i64,
}

/// classifier based on prototypes
pub struct RealPrototypeClassifier {
    pub prototypes: Vec<RealPrototype>,
}

pub fn calc_sims(stimulus: &[Channel], cls: &RealPrototypeClassifier) -> Vec<f64> {
    let mut prototypes_sim: Vec<f64> = vec![]; // similarities of prototypes to stimulus

    // compute similarities of all prototypes
    for i_prototype in &cls.prototypes {
        let mut dist = 0.0; // distance

        for i_channel_idx in 0..stimulus.len() { // iterate over channels
            let channel_prototype: Map2d<f64> = divScalar(&i_prototype.channels[i_channel_idx].v, i_prototype.evidence_count as f64); // divide channel of prototype by count of evidence
            let channel_diff: Map2d<f64> = sub(&stimulus[i_channel_idx].v, &channel_prototype);
            let channel_dist: f64 = distPow2(&channel_diff);
            dist += channel_dist;
        }

        // compute similarity acording to formula from foundalis
        // ref  https://www.foundalis.com/res/poc/PrinciplesOfCognition.htm#first_law
        let c: f64 = 0.5;
        let sim: f64 = (dist*-c).exp();

        prototypes_sim.push(sim);
    }

    prototypes_sim
}

pub fn classify_max(prototypes_sim: &[f64]) -> Option<(usize, f64)> {
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

/// classify stimulus
///
/// returns the index of the prototype
///
/// /param add2 do we want to add the new prototype or revise if we found similar one? useful to only classify without addig anything
pub fn classify(cls: &mut RealPrototypeClassifier, stimulus: &[Channel], current_time:i64, add2: bool) -> Option<usize> {
    let dbg:i64 = 2;
    
    { // classify stimulus
        let prototypes_sim: Vec<f64> = calc_sims(stimulus, &cls);
        if dbg>0{println!("{:?}", prototypes_sim);}

        // do actual classification
        match classify_max(&prototypes_sim) {
            Some((idx, sim)) => {
                if dbg>0{println!("[d ] found prototype!");}
                
                if add2 {
                    // revise evidence
                    
                    for i_channel_idx in 0..cls.prototypes[idx].channels.len() {
                        let revised: Map2d<f64> = add(&cls.prototypes[idx].channels[i_channel_idx].v, &stimulus[i_channel_idx].v);
                        cls.prototypes[idx].channels[i_channel_idx].v = revised;
                    }
                    cls.prototypes[idx].evidence_count+=1;
                    
                    cls.prototypes[idx].last_use_time = current_time; // update time
                }

                return Some(idx);
            },
            None => {
                if add2 {
                    // no match -> create new prototype
                    if dbg>0{println!("[d ] create new prototype");}
                    cls.prototypes.push(RealPrototype{channels:stimulus.to_vec().clone(), last_use_time:current_time, evidence_count:1});
                    return Some(cls.prototypes.len()-1); // return index at added entry
                }
                else {
                    return None; // no prototype was found
                }
            }
        }
    }
}
