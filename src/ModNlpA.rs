// Automatic differentiation
// version log: 25.04.2021: first draft
//              26.04.2021: handling of more complicated vector
//              28.04.2021: primitive encoding of learning vector
//              29.04.2021: adapt layer[1]
//              29.04.2021: more complicated samples
//              30.04.2021: determining winner takes all neuron index to compute one hot result
//              08.05.2021: load model from sqlite

// run with
// rustc -C opt-level=3 ./Ad25_04_2021.rs && ./Ad25_04_2021 && rm ./Ad25_04_2021

// run with to dump relations
// rm -f ./logX.txt && cargo run --release | grep -E '^[^\[].*'
//     grep is to remove debug and information and so on

// TODO< compute result and compare with expected result, measure training set sucess ratio! >

// TODO< add functionality to use learned NN for example sentence >
//    DONE< abstract away conversion from numeric vector to input vector! >
//    DONE< abstract away conversion from output vector to winner takes all result >
//    DONE< feed action back to sequence before stimulating NN again >
//    PARTIALDONE< implement interpreter to execute instructions returned by the NN on the actual words of the NLP sentence to build narsese-relationships from the input sentence! >

// DONE< select layer[0] with a higher probability than the other layer, should improve learning speed >

// DONE< add logic to produce training sequences and outputs for control of the actions of the NN >
//    DONE< add expected output and add code to generate expected output (last control) >
// DONE< adapt layer[1] >
// DONE< training - more complicated input training over flattened sequence >
// TODO< more training examples >

// TODO LOW< set average mse when it's None >

// TODO IF_REQUIRED< hidden neurons! >

// TODO IF_REQUIRED< relu >
// TODO IF_REQUIRED< sigmoid >


//use std::sync::Arc;

use std::path::Path;
use std::fs;
use rusqlite::{Connection, Result, params};
use rusqlite::NO_PARAMS;

// stores the NN as a sqlite database
fn nn_store(nn:&NeuralNetwork) -> Result<()> {
    fs::remove_file("nn_nlp0.db");

    // make sure the file doesn't exist anymore
    while Path::new("nn_nlp0.db").exists() {};

    let conn = Connection::open("nn_nlp0.db")?;

    // nnid : id of the neural network: functionality can later on be used to store multiple neural networks in the same database
    conn.execute(
        "create table if not exists weights (
            id integer primary key AUTOINCREMENT,
            nnid integer not null,
            layer integer not null,
            neuronidx integer not null,
            valueid integer not null,
            value real not null
         )",
        NO_PARAMS,
    )?;

    // make faster write
    conn.execute("PRAGMA synchronous=OFF;",NO_PARAMS)?;

    for i_idx_layer in 0..nn.layers.len() {
        let ilayer = &nn.layers[i_idx_layer];

        for i_idx_neuron in 0..ilayer.neurons.len() {
            let iNeuron = &ilayer.neurons[i_idx_neuron];

            // store bias
            conn.execute(
                "INSERT INTO weights (nnid,layer,neuronidx,valueid,value) values (?1,?2,?3,?4,?5)",
                params![0, i_idx_layer, i_idx_neuron, -1, iNeuron.bias.0],
            )?;
            println!("store {} {} {} {}", i_idx_layer, i_idx_neuron, -1, iNeuron.bias.0);

            for i_idx in 0..iNeuron.weights.len() {
                conn.execute(
                    "INSERT INTO weights (nnid,layer,neuronidx,valueid,value) values (?1,?2,?3,?4,?5)",
                    params![0, i_idx_layer, i_idx_neuron, i_idx, iNeuron.weights[i_idx as usize].0],
                )?;

                //DBG println!("store {} {} {}", i_idx_layer, i_idx_neuron, i_idx);
            }
        }
    }

    Ok(())
}

fn nn_load() -> Result<(NeuralNetwork)> {
    let mut nn:NeuralNetwork = NeuralNetwork{layers:vec![]};

    let conn = Connection::open("nn_nlp0.db")?;


    // make faster write
    conn.execute("PRAGMA synchronous=OFF;",NO_PARAMS)?;

    let mut n_layers:i64 = 0;
    {
        let mut stmt = conn.prepare("SELECT MAX(layer) FROM weights")?;
        let val_iter = stmt.query_map([], |row| {
            Ok(row.get(0)?)
        })?;

        for i_val in val_iter {
            n_layers = i_val.unwrap();
        }
    }
    n_layers += 1;

    for i_idx_layer in 0..n_layers {
        let mut n_neurons:i64 = 0;
        {
            let mut stmt = conn.prepare(&*format!("SELECT MAX(neuronidx) FROM weights where layer={}", i_idx_layer))?;
            let val_iter = stmt.query_map([], |row| {
                Ok(row.get(0)?)
            })?;

            for i_val in val_iter {
                n_neurons = i_val.unwrap();
            }
        }
        n_neurons += 1;

        let mut this_layer: Layer = Layer{neurons:vec![]};

        for i_idx_neuron in 0..n_neurons {
            
            // TODO< order by valueid >
            let mut stmt = conn.prepare(&*format!("SELECT valueid,value FROM weights where layer={} and neuronidx={}", i_idx_layer, i_idx_neuron))?;
            
            struct Row {
                valueid:i64,
                val:f64,
            }

            let val_iter = stmt.query_map([], |row| {
                Ok(Row {
                    valueid: row.get(0)?,
                    val: row.get(1)?,
                })
            })?;

            let mut neuron: WeightBias = WeightBias{bias:(0.0,0.0),weights:vec![],};

            for i_val in val_iter {
                let i_val2 = i_val.unwrap();


                if i_val2.valueid == -1 { // it is the bias
                    //DBG println!("[d99] load {} {} {} {}", i_idx_layer, i_idx_neuron, i_val2.valueid, i_val2.val);

                    neuron.bias = (i_val2.val,0.0);
                }
                else {
                    neuron.weights.push((i_val2.val,0.0));
                }
            }

            this_layer.neurons.push(neuron); // add neuron to layer
        }

        nn.layers.push(this_layer); // add layer
    }

    Ok((nn))
}



fn main() {
    let mut rngi:i64 = 8737; // iterator for rng





    // index of the weight which we are changing
    // if -1 : change bias
    let mut mut_idx: i64;

    let mut nn: NeuralNetwork = NeuralNetwork{layers:vec![]};

    { // build layer[0]
        let mut layer0: Vec<WeightBias> = vec![];

        for _i_neuron in 0..6 {
            layer0.push(WeightBias{weights:vec![],bias:(0.0,0.0)});
            {
                let len1 = layer0.len()-1;
                for _n in 0..34*12 {
                    layer0[len1].weights.push((rng_genfloat(&mut rngi) + /*bias to hopefully learn faster*/0.1,0.0));
                }
                layer0[len1].bias = (rng_genfloat(&mut rngi), 0.0);
            }
        }
        
        nn.layers.push(Layer{neurons:layer0}); // append layer
    }

    
    





    { // build output layer
        let mut layer1: Vec<WeightBias> = vec![];
        for _it_neuron in 0..15 {
            layer1.push(WeightBias{weights:vec![],bias:(0.0,0.0)});
            {
                let len2 = layer1.len()-1;
                for _n in 0..nn.layers[0].neurons.len() {
                    layer1[len2].weights.push((rng_genfloat(&mut rngi)+/*bias to hopefully learn faster*/0.05,0.0));
                }
                layer1[len2].bias = (rng_genfloat(&mut rngi), 0.0);
            }
        }
        nn.layers.push(Layer{neurons:layer1});
    }
    






    
    
    
    // stimulus sequence with epected control sequence
    // is used to generate the training sequences
    struct StimulusWithControl {
        stimulus_seq:Vec<i64>,
        control:Vec<i64>,
    }

    let mut stimulus_with_control:Vec<StimulusWithControl> = vec![];

    {
        // "Tom is fat and fat"
        // NNP VBZ JJ CC JJ
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NNP".to_string(), "VBZ".to_string(), "JJ".to_string(), "CC".to_string(), "JJ".to_string()]),
            control:vec![ 1, 7, 0],
        });


        // "can Tom fly ?"
        // MD NNP VB ?
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["MD".to_string(), "NNP".to_string(), "VB".to_string(), "?".to_string()]),
            control:vec![ 12, 0],
        });



        // "Tom is a ant"
        // NN VBZ DT NN
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NN".to_string(), "VBZ".to_string(), "DT".to_string(), "NN".to_string()]),
            control:vec![ 11, 0],
        });

        //////


        // "Tom is fat"
        // "Tom can fly"
        // NNP VBZ JJ
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NNP".to_string(), "VBZ".to_string(), "JJ".to_string()]),
            control:vec![ 1, 0],
        });

        

        // T0km is food 
        // NN VBZ NN
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NN".to_string(), "VBZ".to_string(), "NN".to_string()]),
            control:vec![ 1, 0],
        });
        
        // Tim and Tom are fat
        // NNP CC NNP VBP JJ
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NNP".to_string(), "CC".to_string(), "NNP".to_string(), "VBP".to_string(), "JJ".to_string()]),
            control:vec![ 2,  3, 0],
        });



        /*

        // Tim and Tom are fat and fat
        // NNP CC NNP VBP JJ CC JJ
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NNP".to_string(), "CC".to_string(), "NNP".to_string(), "VBP".to_string(), "JJ".to_string(), "CC".to_string(), "JJ".to_string()]),
            control:vec![2, 3, 5, 6, 0],
        });

        // John likes the house
        // NNP VBZ DT NN
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NNP".to_string(), "CC".to_string(), "NNP".to_string(), "VBP".to_string(), "JJ".to_string()]),
            control:vec![ 4, 0],
        });



        // Cat want food 
        // NN VBP NN
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NN".to_string(), "VBP".to_string(), "NN".to_string()]),
            control:vec![ 1, 0],
        });
        
        // what is SYSTEM doing ?
        // WP VBZ NNP VBG ?
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["WP".to_string(), "VBZ".to_string(), "NNP".to_string(), "VBG".to_string(), "?".to_string()]),
            control:vec![ 9, 8, 0], // TYPE_QUESTION
        });

        // what is purpose of SYSTEM ?
        // WP VBZ NN IN PRP ?
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["WP".to_string(), "VBZ".to_string(), "NN".to_string(), "IN".to_string(), "PRP".to_string(), "?".to_string()]),
            control:vec![ 9, 10, 0], // TYPE_QUESTION
        });

        // purpose of Tom is hunting
        // NN IN NNP VBZ NN
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NN".to_string(), "IN".to_string(), "NNP".to_string(), "VBZ".to_string(), "NN".to_string()]),
            control:vec![ 13, 0],
        });

        // the purpose of Tom is hunting
        // DT NN IN NNP VBZ NN
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["DT".to_string(), "NN".to_string(), "IN".to_string(), "NNP".to_string(), "VBZ".to_string(), "NN".to_string()]),
            control:vec![ 14, 0],
        });




        // Cat want to dance 
        // NN VBT PRT_TO VB
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NN".to_string(), "VBT".to_string(), "PRT_TO".to_string(), "VB".to_string(), "JJ".to_string()]),
            control:vec![ 4, 0],
        });
        */





        /*
        // Cat doesn't want food
        // NN VBZ VB NN
        // NOTE< "doesn't" is problematic for negation, the program needs to check if 2nd word is "doesn't" instead of VBZ >
        stimulus_with_control.push(StimulusWithControl{
            stimulus_seq:prot_pos0(vec!["NN".to_string(), "VBZ".to_string(), "VB".to_string(), "NN".to_string()]),
            control:vec![ 7, 0],
        });
        */
    }
    
    


    // training sequences
    let mut seqs:Vec<Seq> = vec![];

    // *#* translate stimulus_with_control into subsequence training sequences
    {
        for i_stimulus_with_control in &stimulus_with_control {

            for i_control_idx_2 in 0..i_stimulus_with_control.control.len() as i64 { // iterate over control except for last one because the network never sees it
                let mut controlsequence_seen_until_now:Vec<i64> = i_stimulus_with_control.control[0..i_control_idx_2 as usize].to_vec();
                controlsequence_seen_until_now = controlsequence_seen_until_now.iter().map(|iv| pos_ret_codes()+iv).collect::<Vec<i64>>(); // encode to input space

                let mut this_seq:Vec<i64> = vec![]; // sequence
                this_seq.extend(i_stimulus_with_control.stimulus_seq.clone()); // stimulus consists out of the NLP stimulus
                this_seq.extend(controlsequence_seen_until_now); // followed by the control sequence seen until the current control

                //println!("{}", controlsequence_seen_until_now.len());

                seqs.push(Seq{seq:this_seq, outonehot:i_stimulus_with_control.control[i_control_idx_2 as usize]});
            }
        }
    }




    /* OLD code where we add stimulus manually

    // training sequence 0
    seqs.push(Seq{seq: prot_pos0(vec!["NNP".to_string(), "VBZ".to_string(), "JJ".to_string()])});
    seqs.push(Seq{seq: prot_pos0(vec!["NNP".to_string(), "VBZ".to_string(), "JJ".to_string()])});
    {
        let len3 = seqs.len();
        seqs[len3-1].seq.push(8+0); // append code to  ENCODE-REL(0,1,2)
    }
    /* commented because the NN doesn't see FIN as stimulus
    seqs.push(Seq{seq: prot_pos0(vec!["NNP".to_string(), "VBZ".to_string(), "JJ".to_string()])});
    {
        let len3 = seqs.len();
        seqs[len3-1].seq.push(8+0); // append code to  ENCODE-REL(0,1,2)
        seqs[len3-1].seq.push(8+1); // append code to  FIN
    }*/
    // training sequence 1
    seqs.push(Seq{seq: prot_pos0(vec!["NN".to_string(), "VBZ".to_string(), "NN".to_string()])});
    

    // NNP CC NNP VBP JJ
    seqs.push(Seq{seq: prot_pos0(vec!["NNP".to_string(), "CC".to_string(), "NNP".to_string(), "VBP".to_string(), "JJ".to_string()])});
    
    */

    println!("[i  ] #trainingset = {}", seqs.len());
    
    let lr_base:f64 = 0.01; // base learning rate
    let mut lr:f64 = lr_base; // PARAMETER learning rate

    let mut mse_avg:f64=100.0; // running "average" of mse

    // input vector
    let mut b: Vec<(f64,f64)> = vec![];
    for _it in 0..34*12 {
        b.push((0.0,0.0));
    }

    let train:bool = false; // train the model before loading and running it?

    if train {
        // let pieces_of_compute = 5; // 5 was sufficient for simple function
        //let pieces_of_compute = 8; // how many pieces of compute? 8 was sufficient for experiment
        //let pieces_of_compute = 16;
        //let pieces_of_compute = 30; // works fine for more complicated training samples
        //let pieces_of_compute = 40; // testing
        //let pieces_of_compute = 60; // testing
        //let pieces_of_compute = 120; // testing
        //let pieces_of_compute = 80; // testing


        //let pieces_of_compute = 120; // worked fine
        let pieces_of_compute = 30; // smaller training for testing

        let epoch_max:i64 = 5000000*pieces_of_compute; // how many epochs do we train?

        for i_epoch in 0..epoch_max {
            if i_epoch == epoch_max/5 { // 1/5 of compute reached?
                lr = lr_base / 10.0; // fine tuning
            }
            else if i_epoch == epoch_max/2 { // 1/2 of compute reached?
                lr = lr_base / 20.0;  // more fine tuning
            }
            
            
            // COMMENTED< unfair selection of layer >
            //     let sel_mut_layer_idx:usize = rng_genint(&mut rngi, nn.layers.len() as i64) as usize; // select mutated layer

            let sel_mut_layer_idx:usize = {
                let mut val = 0;
                if rng_genfloat(&mut rngi) > 0.9 {
                    val = 1; // mutate 2nd layer in rare cases
                }
                val
            };

            let sel_neuron_idx:usize = rng_genint(&mut rngi, nn.layers[sel_mut_layer_idx].neurons.len() as i64) as usize; // select index of the neuron in the layer


            let rand_number:i64 = rng_genint(&mut rngi, nn.layers[sel_mut_layer_idx].neurons[sel_neuron_idx].weights.len() as i64 + 1);
            mut_idx = (nn.layers[sel_mut_layer_idx].neurons[sel_neuron_idx].weights.len() as i64-1) - rand_number;

            //DEBUG println!("mut_idx = {}", mut_idx);
            
            if mut_idx >= 0 { // do we change weight?
                nn.layers[sel_mut_layer_idx].neurons[sel_neuron_idx].weights[mut_idx as usize].1 = 1.0; // change differentiation
            }
            else { // we change bias
                nn.layers[sel_mut_layer_idx].neurons[sel_neuron_idx].bias.1 = 1.0;
            }
            
            let mut target_out:Vec<f64>; // = vec![0.0; nn.layers[1].neurons.len()]; // target output
            
            // transfer input vector
            let sample_sel:i64 = rng_genint(&mut rngi, seqs.len() as i64); // selected sample
            {
                

                
                let sel_seq3: &Seq = &seqs[sample_sel as usize];

                { // encode selected seq
                    b = conv_seq_to_nninput(sel_seq3);
                }

                // set target output for learning
                target_out = vec![0.1; nn.layers[1].neurons.len()];
                target_out[sel_seq3.outonehot as usize] = 0.9;
            }


            let layer1_out: Vec<(f64,f64)> = forward(&nn, &b);
            
            let sel_out_idx = rng_genint(&mut rngi, layer1_out.len() as i64) as usize; // selected output index to adapt

            let diff:f64 = layer1_out[sel_out_idx].0 - target_out[sel_out_idx]; // compute difference to target
            
            let mse:f64 = diff*diff;
            mse_avg = mse_avg*(1.0 - 0.000001) + mse*0.000001;

            if i_epoch % 16433 == 0 {
                println!("[i  ] progress = {}", i_epoch as f64 / epoch_max as f64);
                println!("[i  ] mse = {}", mse);
                println!("[i  ] mse avg = {}", mse_avg);
            }
            let d2:f64 = lr*diff*layer1_out[sel_out_idx].1;
            
            // adapt
            if mut_idx >= 0 { // do we change weight?
                nn.layers[sel_mut_layer_idx].neurons[sel_neuron_idx].weights[mut_idx as usize].0 -= d2;
            }
            else { // we change bias
                nn.layers[sel_mut_layer_idx].neurons[sel_neuron_idx].bias.0 -= d2;
            }
            
            // reset differentiate
            if mut_idx >= 0 { // do we change weight?
                nn.layers[sel_mut_layer_idx].neurons[sel_neuron_idx].weights[mut_idx as usize].1 = 0.0; // change differentiation
            }
            else { // we change bias
                nn.layers[sel_mut_layer_idx].neurons[sel_neuron_idx].bias.1 = 0.0;
            }
        }

        nn_store(&nn); // store NN


        { // debug: print weights
            let mut str2:String = "".to_string();

            for iv in &nn.layers[0].neurons[0].weights {
                str2 += &format!("{} ", iv.0).to_string();
            }

            str2 = format!("[{}]", str2);

            println!("[d  ] weights = {}", str2);
            println!("[d  ] bias    = {}", nn.layers[0].neurons[0].bias.0);
        }
    }
    


    network_test();
}

// helper to test network
fn network_test() {
    // "Tom is fat and fat"
    // NNP VBZ JJ CC JJ
    let stimulus_words: Vec<String> = vec!["Tom".to_string(), "is".to_string(), "fast".to_string(), "and".to_string(), "fat".to_string()];

    let (control_seq, relations) = network_run(&stimulus_words);
}

// load the network from the file and run it
// /param stimulus_words the actual words of the stimulus
// /return (control sequence which was returned from the NN, extracted relations)
pub fn network_run(stimulus_words: &[String]) -> (Vec<i64>, Vec<Relation>) {
    let nn:NeuralNetwork = nn_load().unwrap();


    { // debug: print weights
        let mut str2:String = "".to_string();

        for iv in &nn.layers[0].neurons[0].weights {
            str2 += &format!("{} ", iv.0).to_string();
        }

        str2 = format!("[{}]", str2);

        println!("[d  ] weights = {}", str2);
        println!("[d  ] bias    = {}", nn.layers[0].neurons[0].bias.0);
    }


    use std::fs::OpenOptions;
    use std::io::prelude::*;


    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .append(true)
        .open("logX.txt")
        .unwrap();

    // function to log text to console and log file
    let mut log = |s: &String| {
        println!("{}", &s);
        if let Err(e) = writeln!(file, "{}", &s) {
            eprintln!("Couldn't write to file: {}", e);
        }
    };

    let mut control_log: Vec<i64> = vec![];

    let mut extracted_relations: Vec<Relation> = vec![]; // array of extracted relations
    
    let mut current_stimulus: Vec<i64> = prot_pos0(conv_tokens_to_poss(&stimulus_words)); //prot_pos0(vec!["NNP".to_string(), "VBZ".to_string(), "JJ".to_string(), "CC".to_string(), "JJ".to_string()]);

    for _i_iteration in 0..10 { // iterate only till the maximal iterations aren't reached, we do this to prevent infinite loops
        println!(""); // only do this if debug level is aproriate
        let nn_stimulus: Vec<(f64,f64)> = conv_seq_to_nninput(&Seq{seq:current_stimulus.clone(), outonehot:-1/*dummy*/}); // compute stimulus for NN

        // feed stimulus into NN
        let nn_out: Vec<(f64,f64)> = forward(&nn, &nn_stimulus);

        // DEBUG
        for it_idx in 0..nn_out.len() {
            log(&format!("[d5 ] nnout[{}] = {}", it_idx, nn_out[it_idx].0));               
        }

        

        // determine curent action by determining the index of max one hot winner
        let current_action_idx:i64 = calc_max_onehot_idx(&nn_out);

        control_log.push(current_action_idx); // add to log
        
        log(&format!("[d5 ] current action idx = {}", current_action_idx));

        if current_action_idx == 0 { // is FIN action? then we are done with interpreting actions
            break;
        }

        
        { // debug + build relations
            if current_action_idx == 0 { // is FIN action? then we are done with interpreting actions
                // do nothing
            }
            else if current_action_idx == 1 {
                // action to build relation [1]([0], [2])
        
                log(&format!("//REL {}({}, {})", stimulus_words[1], stimulus_words[0], stimulus_words[2]));
                log(&format!("{}({}, {}).", stimulus_words[1], stimulus_words[0], stimulus_words[2]));
                extracted_relations.push(Relation {
                    head: stimulus_words[1].clone(),
                    args: vec![stimulus_words[0].clone(), stimulus_words[2].clone()],
                    isNegated: false,
                    conf: 0.998, 
                });
            }
            else if current_action_idx == 7 {
                // action to build relation [1]([0], [4])
        
                log(&format!("//REL {}({}, {})", stimulus_words[1], stimulus_words[0], stimulus_words[4]));
                log(&format!("{}({}, {}).", stimulus_words[1], stimulus_words[0], stimulus_words[4]));
                extracted_relations.push(Relation {
                    head: stimulus_words[1].clone(),
                    args: vec![stimulus_words[0].clone(), stimulus_words[4].clone()],
                    isNegated: false,
                    conf: 0.998, 
                });
            }
        }

        
        // add selected action to sequence to tell network that the action was indeed executed
        current_stimulus.push(pos_ret_codes() + current_action_idx);
    }
    
    (control_log, extracted_relations)
}

/// structure for an relation extracted from natural language
pub struct Relation {
    pub head: String, // head
    pub args: Vec<String>, // arguments
    pub isNegated: bool,
    pub conf: f64, // confidence
}

/// convert relation to narsese
pub fn conv_rel_to_narsese(rel:&Relation) -> String {
    // TODO< handle isNegated >

    if rel.args.len() == 2 {
        return format!("<({}*{}) --> {}>. {{{} {}}}", &rel.args[0], &rel.args[1], &rel.head,   1.0, rel.conf);
    }
    else {
        panic!("not implemented!");
    }
}

// dot product
pub fn dot(a:&[(f64,f64)], b:&[(f64,f64)]) -> (f64,f64) {
    //println!("{} {}", a.len(), b.len());


    // make it safe
    if a.len() != b.len() {
        panic!("dot(): arguments must have equal length!");
    }

    let mut acc:(f64,f64) = (0.0,0.0);
    for idx in 0..b.len() {
        let r0 = mul(a[idx],b[idx]);
        acc = add(acc, r0);
    }
    acc
}

pub fn add(a:(f64,f64),b:(f64,f64)) -> (f64,f64) {
    (a.0+b.0,a.1+b.1)
}

pub fn mul(a:(f64,f64),b:(f64,f64)) -> (f64,f64) {
    (a.0*b.0,a.1*b.0+a.0*b.1)
}




// one hot encoding
pub fn encode_onehot(val:i64, len:i64) -> Vec<f64> {
    let mut res = vec![0.0;len as usize];
    res[val as usize] = 1.0;
    res
}



// datastructure for weight+bias
pub struct WeightBias {
    pub weights: Vec<(f64,f64)>,
    pub bias: (f64,f64),
}

// layer of NN
pub struct Layer {
    pub neurons: Vec<WeightBias>, // weights+bias of individual neurons of layer
}


// neural network
pub struct NeuralNetwork {
    pub layers: Vec<Layer>,
}

// forward propagate with AD
pub fn forward(nn:&NeuralNetwork, stimulus:&Vec<(f64,f64)>) -> Vec<(f64,f64)> {
    let mut layer1_out = vec![];
    { // compute output of NN
        let mut layer0_out:Vec<(f64,f64)> = vec![];
        // compute output of layer[0]
        for i_layer in &nn.layers[0].neurons {
            let mut acc0 = dot(stimulus,&i_layer.weights);
            acc0 = add(acc0, i_layer.bias);
            layer0_out.push(acc0);
        }

        // compute output of layer[1]
        
        for i_layer in &nn.layers[1].neurons {
            let mut acc0 = dot(&layer0_out,&i_layer.weights);
            acc0 = add(acc0, i_layer.bias);
            // TODO< compute sigmoid activation function >
            layer1_out.push(acc0);
        }
    }

    layer1_out
}

/// misc function:
/// function to determine index of maximal value encoded as one hot
/// also manages clamping to 0.0..1.0
pub fn calc_max_onehot_idx(v:&[(f64,f64)]) -> i64 {
    let mut val = 0.0;
    let mut max_idx = 0;
    for i_idx in 0..v.len() {
        let i_val:f64 = v[i_idx].0.clamp(0.0, 1.0); // clamp to probability 0.0 and 1.0
        if i_val > val {
            val = i_val;
            max_idx = i_idx as i64;
        }
    }
    max_idx
}





///////////////////////
// problem specific code
///////////////////////



// structure for training sequence
// encodes a numeric sequence of "words", which are just integers of some alphabet
pub struct Seq {
    pub seq:Vec<i64>,
    pub outonehot:i64, // one hot encoding of the output
}

// convert a sequence of stimulus to a NN-input encoding
// the NN-input encoding which is returned can be stuffed into a NN to yield a result
// /param seq is a sequence of encoded letters from an alphabet
//        the function manages input with different lengths automatically in a GPT-like way

// TODO< refactor input to &[i64] type because we don't need the onehot output >
pub fn conv_seq_to_nninput(seq:&Seq) -> Vec<(f64,f64)> {
    let n_codes:i64 = 34; // how many codes are there?

    let n_signs:i64 = 12; // how many signs

    // generate coded vector which is empty
    // the coded vector are the letters which will be encoded as one hot encoding
    let mut codes:Vec<i64> = vec![n_codes-1;n_signs as usize];
    // fill vector with end of to encoding vector
    let len_codes = codes.len();

    
    let mut b:Vec<(f64,f64)> = vec![]; // NN-input encoding

    { // encode selected seq
        //let seqidx = sample_sel as usize; // index in seqs

        let seq2_len = seq.seq.len();

        for seq2_idx in 0..seq2_len { // iterate from end of sequence to beginning
            // TODO< don't iterate over maximal capacity of encoding!
            
            
            codes[len_codes-1-(seq.seq.len()-1-seq2_idx)] = seq.seq[seq2_idx];
        }
    }

    /*
    if sample_sel == 0 {
        let seqidx = 0; // index in seqs
        codes[len_codes-1-(sel_seq3.seq.len()-1-3)] = sel_seq3.seq[3];
        codes[len_codes-1-(sel_seq3.seq.len()-1-2)] = sel_seq3.seq[2];
        codes[len_codes-1-(sel_seq3.seq.len()-1-1)] = sel_seq3.seq[1];
        codes[len_codes-1-(sel_seq3.seq.len()-1-0)] = sel_seq3.seq[0];
    }
    else if sample_sel == 1 {
        let seqidx = 1; // index in seqs
        codes[len_codes-1-(sel_seq3.seq.len()-1-3)] = sel_seq3.seq[3];
        codes[len_codes-1-(sel_seq3.seq.len()-1-2)] = sel_seq3.seq[2];
        codes[len_codes-1-(sel_seq3.seq.len()-1-1)] = sel_seq3.seq[1];
        codes[len_codes-1-(sel_seq3.seq.len()-1-0)] = sel_seq3.seq[0];
    }
    else if sample_sel == 2 {
        let seqidx = 2; // index in seqs
        codes[len_codes-1-(sel_seq3.seq.len()-1-4)] = sel_seq3.seq[4];
        codes[len_codes-1-(sel_seq3.seq.len()-1-3)] = sel_seq3.seq[3];
        codes[len_codes-1-(sel_seq3.seq.len()-1-2)] = sel_seq3.seq[2];
        codes[len_codes-1-(sel_seq3.seq.len()-1-1)] = sel_seq3.seq[1];
        codes[len_codes-1-(sel_seq3.seq.len()-1-0)] = sel_seq3.seq[0];
    }
    */

    
    for i_code in &codes { // iterate over codes
        // encode as one-hot
        let subvec_as_ad = encode_onehot(*i_code, n_codes).iter().map(|v| (*v, 0.0)).collect::<Vec<(f64,f64)>>(); // convert to autodiff
        b.extend(subvec_as_ad);
    }

    b
}





// rng
pub fn rng_genint(v:&mut i64, maxval:i64) -> i64 {
    let rand_number:i64 = 1383462957253 + *v*1337 + ((*v as f64).sin() * 8461256485.0) as i64;
    *v+=1;
    rand_number%maxval
}

pub fn rng_genfloat(v:&mut i64) -> f64 {
    let rand_number:f64 = 1383462957253.0 + *v as f64*1337.0 + ((*v as f64).sin() * 8461256485.0);
    *v+=1;
    rand_number.fract()
}

// input: part of speech of input sentence:

//  0 : Verb            : VBZ
//  1 : Adjective       : JJ
//  2 : Noun            : NNP
//  3 : Conjunction     : CC     "and"
//  4 : ,               : ,      special symbol
//  5 : Noun            : NN
//  6 : Verb            : VBP
//  7 : SPECIAL: stop        is used to signal the stop of the sentence, other stuff follows, such as control commands
//  8 : Determiner      : DT
//  9 : preteritum "to" : PRT_TO
// 10 : verb            : VB
// 11 : verb            : VBT
// 12 : pronoun         : WP
// 13 : ?               : ?      special symbol for Q&A
// 14 : verb            : VBG
// 15 : preposition     : IN
// 16 : pronoun         : PRP
// 17 : verb            : MD


// 

// "tom is fat" 
// NNP VBZ JJ

// part of speech tagger (isn't that great): https://parts-of-speech.info/
//  * website seems to wrongly classify "can" as the wrong type!

// returns the number of codes for "part of speech" encodings
pub fn pos_ret_codes()->i64 {
    18
}

// part of speech prototype
// TODO< use for training data! >
pub fn prot_pos0(cls:Vec<String>) -> Vec<i64> {
    // vector of part of speech classification

    // "tom is fat" 
    // NNP VBZ JJ
    //let mut cls:Vec<String> = vec!["NNP".to_string(), "VBZ".to_string(), "JJ".to_string()];

    // T0km is food 
    // NN VBZ NN

    // Tim and Tom are fat 
    // NNP CC NNP VBP JJ

    let mut cls2:Vec<i64> = vec![];
    for i_cls in &cls {
        if      i_cls == "VBZ"    { cls2.push(0); }
        else if i_cls == "JJ"     { cls2.push(1); }
        else if i_cls == "NNP"    { cls2.push(2); }
        else if i_cls == "CC"     { cls2.push(3); }
        else if i_cls == ","      { cls2.push(4); }
        else if i_cls == "NN"     { cls2.push(5); }
        else if i_cls == "VBP"    { cls2.push(6); }
        else if i_cls == "DT"     { cls2.push(8); }
        else if i_cls == "PRT_TO" { cls2.push(9); }
        else if i_cls == "VB"     { cls2.push(10); }
        else if i_cls == "VBT"    { cls2.push(11); }
        else if i_cls == "WP"     { cls2.push(12); }
        else if i_cls == "?"      { cls2.push(13); }
        else if i_cls == "VBG"    { cls2.push(14); }
        else if i_cls == "IN"     { cls2.push(15); }
        else if i_cls == "PRP"    { cls2.push(16); }
        else if i_cls == "MD"     { cls2.push(17); }
        else {
            panic!("{} not implemented!", i_cls); // bail out (fail fast)
        }
    }

    cls2.push(7); // STOP

    cls2
}


// convert words to part of speech
pub fn conv_tokens_to_poss(tokens:&[String]) -> Vec<String> {
    let mut res = vec![];
    for i_token in tokens {
        if      i_token == &"Tom".to_string() {res.push("NNP".to_string());}
        else if i_token == &"Tim".to_string() {res.push("NNP".to_string());}
        else if i_token == &"John".to_string() {res.push("NNP".to_string());}
        else if i_token == &"is".to_string()  {res.push("VBZ".to_string());}
        else if i_token == &"can".to_string()  {res.push("MD".to_string());}
        else if i_token == &"are".to_string()  {res.push("VBP".to_string());}
        else if i_token == &"?".to_string()  {res.push("?".to_string());}
        else if i_token == &"to".to_string()  {res.push(" PRT_TO".to_string());}
        else if i_token == &"of".to_string()  {res.push("IN".to_string());}
        else if i_token == &"the".to_string()  {res.push("DT".to_string());}
        else if i_token == &"what".to_string()  {res.push("WP".to_string());}
        else if i_token == &"and".to_string()  {res.push("CC".to_string());}
        else if i_token == &"fast".to_string()  {res.push("JJ".to_string());}
        else if i_token == &"fat".to_string()  {res.push("JJ".to_string());}
        else if i_token == &"hunting".to_string()  {res.push("NN".to_string());}
        else if i_token == &"purpose".to_string()  {res.push("NN".to_string());}
        else if i_token == &"food".to_string()  {res.push("NN".to_string());} // is this right?
        else {panic!("unknown word");}
    }
    res
}

