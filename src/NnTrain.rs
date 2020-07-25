// training of NN's with automatic differentiation

// seems to be bugged somewhere :(

use rand::Rng;
use rand::rngs::ThreadRng;

use ad;
use Nn::{Network, buildNnFromParameters};

// prototyping for training
pub fn testTrainingNn0() {
    let nInput = 5*5+2;
    let nNeuronsLayer0 = 8;
    let nNeuronsLayer1 = 3;

    let mut params:Vec<f64> = vec![];
    // TODO< compute number of parameters ! >
    for i in 0..340 {
        params.push((((1.19667*(i as f64 + 1877.2)) % 1.0) * 2.0 - 1.0) * 0.1); // TODO< take value from gaussian
    }



    //println!("{:?}", params);

    let mut rng:ThreadRng = rand::thread_rng();

    let mut trainingTuples:Vec<TrainingTuple> = Vec::new();
    trainingTuples.push(TrainingTuple {
        i: vec![0.0; nInput as usize],
        o: vec![0.9, 0.1, 0.1],
    });
    
    trainingTuples.push(TrainingTuple {
        i: vec![0.1; nInput as usize],
        o: vec![0.1, 0.9, 0.1],
    });

    trainingTuples.push(TrainingTuple {
        i: vec![0.9; nInput as usize],
        o: vec![0.1, 0.9, 0.1],
    });

    for it in 0..15000000 {
        let ref selTrainingTuple:TrainingTuple = trainingTuples[rng.gen_range(0, trainingTuples.len())];
        
        let paramsIdx:usize = rng.gen_range(0, params.len()); // manipulated weight idx

        let mut paramsIdx2 = 0;
        let network:Network = buildNnFromParameters(&mut paramsIdx2, &params, nInput, 1, nNeuronsLayer0, nNeuronsLayer1, Some(paramsIdx));
        
        //let stimulus:Vec<ad::Ad> = vec![ad::Ad{r:0.1,d:0.0}; nInput as usize];
        let mut stimulus:Vec<ad::Ad> = vec![];
        for idx in 0..selTrainingTuple.i.len() {
            stimulus.push(ad::Ad{r:selTrainingTuple.i[idx], d:0.0});
        }

        // TODO< fill stimulus

        let mut out = vec![];
        {
            // y vector, which is the result of the NN for layer[0]
            let mut ys0 = vec!(ad::Ad{r:0.0,d:0.0}; network.neuronsLayer0.len());
            for ysIdx in 0..ys0.len() {
                ys0[ysIdx] = ad::calc(&stimulus, &network.neuronsLayer0[ysIdx]);
            }
    
            // layer[1]
            let mut ys1 = vec!(ad::Ad{r:0.0,d:0.0}; network.neuronsLayer1.len());
            
            for ysIdx in 0..ys1.len() {
                ys1[ysIdx] = ad::calc(&ys0, &network.neuronsLayer1[ysIdx]);
            }
    
            out = ys1;
        }

        let lRate:f64 = 0.1 / 16.0; // learning rate
    
        //let mut trainingY:Vec<f64> = vec![0.9, 0.1];
        let ref trainingY:Vec<f64> = selTrainingTuple.o;

        for idx in 0..out.len() {
            let diff:f64 = out[idx].r-trainingY[idx];
            params[paramsIdx] -= (/*(if diff > 0.0 {1.0} else {-1.0})*/diff * out[idx].d * lRate);
        }
        
        if it % 100 == 0 {
            //println!("d={}", convVecToStr(out.iter().map(|v| v.d).collect()));
            
            let x:Vec<f64> = out.iter().map(|v| v.r).collect();

            for iv in &x {
                assert!(!iv.is_nan());
            }

            println!("train.y={}", convVecToStr(&trainingY));
            println!("out={}", convVecToStr(&x));
        }

    }
}

// tuple for training
pub struct TrainingTuple {
    pub i:Vec<f64>,
    pub o:Vec<f64>,
}

// helper to convert vector to naked string
pub fn convVecToStr(v:&[f64]) -> String {
    let mut res="".to_string();
    for iv in v {
        res = res + format!("{} ", iv).as_str();
    }
    res
}
