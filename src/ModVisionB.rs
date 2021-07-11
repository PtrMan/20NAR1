use crate::Tv::*;
use crate::TvVec::*;
use crate::Map2d::*;

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
    let n_quantize:i64 = 3; // how many quatization steps are used?
    let stimulus_conf: f64 = 0.05; // confidence of the perceived stimulus
    
    { // fill prototypes with test-prototyping for prototyping
        /*{
            let aVec = vec![Tv{f:1.0, c:0.1}, Tv{f:0.0, c:0.1}, Tv{f:0.0, c:0.1}];
            cls.prototypes.push(Prototype{v:aVec});
        }
         */
    }

    { // classify stimulus
        let map: Map2d<f64> = makeMap2d(4, 4);

        // * quantize image from Map2d
        let stimulus: Vec<Tv>  = conv_img_to_tv_vec(&map, n_quantize, stimulus_conf);

        // * classify
        let current_time = 0;
        classify(&mut cls, &stimulus, current_time, true);
    }

    { // classify stimulus
        let mut map: Map2d<f64> = makeMap2d(4, 4);
        for iy in 0..map.h {
            for ix in 0..map.w {
                writeAt(&mut map, iy,ix,1.0);
            }
        }

        // * quantize image from Map2d
        let stimulus: Vec<Tv>  = conv_img_to_tv_vec(&map, n_quantize, stimulus_conf);

        // * classify
        let current_time = 0;
        classify(&mut cls, &stimulus, current_time, true);
    }
}

/// helper to convert image to TV-vector
pub fn conv_img_to_tv_vec(map: &Map2d<f64>, n_quantize:i64, stimulus_conf:f64) -> Vec<Tv> {
    let mut stimulus: Vec<Tv> = vec![];

    for iy in 0..map.h {
        for ix in 0..map.w {
            let v:f64 = readAt(&map, iy,ix);
            stimulus.extend(&conv_to_tv_vec(v, n_quantize, stimulus_conf));
        }
    }

    stimulus
}

/// learned prototype
pub struct Prototype {
    /// TV vector of evidence
    pub v: Vec<Tv>,
    /// time of last use, used for forgetting policy
    pub last_use_time: i64,
}

/// classifier based on prototypes
pub struct PrototypeClassifier {
    pub prototypes: Vec<Prototype>,
}

pub fn calc_sims(stimulus: &[Tv], cls: &PrototypeClassifier) -> Vec<f64> {
    let mut prototypes_sim: Vec<f64> = vec![]; // similarities of prototypes to stimulus

    // compute similarities of all prototypes
    for i_prototype in &cls.prototypes {
        let tvSim: Tv = foldVec(&compVec(&stimulus, &i_prototype.v));        
        let sim: f64 = calcExp(&tvSim); // we interpret expectation as similarity of the TV-vectors
        
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
/// /param add do we want to add the new prototype or revise if we found similar one? useful to only classify without addig anything
pub fn classify(cls: &mut PrototypeClassifier, stimulus: &[Tv], current_time:i64, add: bool) -> Option<usize> {
    let dbg:i64 = 0;
    
    { // classify stimulus
        let prototypes_sim: Vec<f64> = calc_sims(stimulus, &cls);
        if dbg>0{println!("{:?}", prototypes_sim);}

        // do actual classification
        match classify_max(&prototypes_sim) {
            Some((idx, sim)) => {
                if dbg>0{println!("[d ] found prototype!");}
                
                if add {
                    // revise evidence
                    let revised: Vec<Tv> = revVec(&cls.prototypes[idx].v, &stimulus);
                    cls.prototypes[idx].v = revised;

                    cls.prototypes[idx].last_use_time = current_time; // update time
                }

                return Some(idx);
            },
            None => {
                if add {
                    // no match -> create new prototype
                    if dbg>0{println!("[d ] create new prototype");}
                    cls.prototypes.push(Prototype{v:stimulus.to_vec(), last_use_time:current_time});
                    return Some(cls.prototypes.len()-1); // return index at added entry
                }
                else {
                    return None; // no prototype was found
                }
            }
        }
    }
}



// helper
pub fn quantisize(v:f64, n:i64) -> i64 {
    (v * ((n-1) as f64)) as i64
}

pub fn conv_to_tv_vec(v:f64, n:i64, conf:f64) -> Vec<Tv> {
    let quantisized: i64 = quantisize(v,n);
    let mut vec = vec![Tv{f:0.0, c:conf};n as usize];
    vec[quantisized as usize] = Tv{f:1.0, c:conf};
    vec
}
