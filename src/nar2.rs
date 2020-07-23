
// prototype of NAR components
pub fn protoNarEntry0() {
    let mut mem = narPerception::Mem{
        concepts:HashMap::new(),
    };

    {
        let sentence = narPerception::SentenceDummy {
            isOp:false, // decide if it is a op by random
            term:Rc::new(narPerception::Term::Cop(narPerception::Copula::INH, Rc::new(narPerception::Term::Name(format!("e{}", 0))), Rc::new(narPerception::Term::Name(format!("E{}", 0))))),
            t:0,
        };
        narPerception::storeInConcepts(&mut mem, &sentence);
    }
}

// TODO< make to unittest >
pub fn testNarPerception0() {
    let mut rng = rand::thread_rng();

    let mut eventsInFifo = vec![];
    let mut fifoSize = 20; // size of fifo

    // add dummy events for testing the fifo algorithm

    
    for iTime in 0..50 {
        eventsInFifo.push(narPerception::SentenceDummy {
            isOp:rng.gen::<f64>() < 0.2, // decide if it is a op by random
            term:Rc::new(narPerception::Term::Cop(narPerception::Copula::INH, Rc::new(narPerception::Term::Name(format!("e{}", iTime))), Rc::new(narPerception::Term::Name(format!("E{}", iTime))))),
            t:iTime,
        });
    }

    eventsInFifo = (&eventsInFifo[(eventsInFifo.len()-fifoSize).max(0)..]).to_vec(); // slice so only that last n events are inside


    // test sampling
    for _iSample in 0..100 {
        // * sample events
        let sampledEvents = narPerception::perceiveImpl(&eventsInFifo, &mut rng);

        //println!("OUT");
        //for iSentence in &sampledEvents {
        //    println!("{}", iSentence.term);
        //}

        // build impl seq
        if sampledEvents.len() == 3 {
            let _0 = &*sampledEvents[0].term;
            let _1 = &*sampledEvents[1].term;
            let _2 = &*sampledEvents[2].term;

            println!("<({} &/ {}) =/> {}>.", narPerception::convTermToStr(&_0), narPerception::convTermToStr(&_1), narPerception::convTermToStr(&_2));
        }
    }
}
