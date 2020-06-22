// neuronal network defintion and helpers
// Network and building of network can be used with our without backprop for training

use ad;

// network used to solve a problem
pub struct Network {
    pub neuronsLayer0:Vec::<ad::Neuron>,
    pub neuronsLayer1:Vec::<ad::Neuron>,
}

// helper to build network from parameters
// /param diffIdx ad is set to 1.0 for this idx
pub fn buildNnFromParameters(paramsIdx:&mut i64, params:&Vec<f64>, nNeuronsLayer0:i64, nNeuronsLayer1:i64, diffIdx:Option<usize>) -> Network {
    let mut network:Network = Network { // network of the solver
        neuronsLayer0:Vec::<ad::Neuron>::new(),
        neuronsLayer1:Vec::<ad::Neuron>::new(),
    };

    for _iNeuronIdx in 0..nNeuronsLayer0 { // loop to transfer to neurons
        let mut weights:Vec::<ad::Ad> = Vec::<ad::Ad>::new();
        for _i in 0..5*5 {
            let v = params[*paramsIdx as usize];
            weights.push(ad::Ad{r:v,d:if diffIdx.is_some() && diffIdx.unwrap() as i64 == *paramsIdx {1.0} else {0.0}});
            *paramsIdx+=1;
        }
        let bias = params[*paramsIdx as usize] * 15.0; // boost parameter because it is the bias
        network.neuronsLayer0.push(ad::Neuron{
            weights: weights,
            bias:ad::Ad{r:bias,d:if diffIdx.is_some() && diffIdx.unwrap() as i64 == *paramsIdx {1.0} else {0.0}},
            act: 0,
        });
        *paramsIdx+=1;
    }

    for _iNeuronIdx in 0..nNeuronsLayer1 { // loop to transfer to neurons
        let mut weights:Vec::<ad::Ad> = Vec::<ad::Ad>::new();
        for _i in 0..nNeuronsLayer0 {
            let v = params[*paramsIdx as usize];
            weights.push(ad::Ad{r:v,d:if diffIdx.is_some() && diffIdx.unwrap() as i64 == *paramsIdx {1.0} else {0.0}});
            *paramsIdx+=1;
        }
        let bias = params[*paramsIdx as usize] * 8.0; // boost parameter because it is the bias
        network.neuronsLayer1.push(ad::Neuron{
            weights: weights,
            bias:ad::Ad{r:bias,d:if diffIdx.is_some() && diffIdx.unwrap() as i64 == *paramsIdx {1.0} else {0.0}},
            act: 1,
        });
        *paramsIdx+=1;
    }

    network
}
