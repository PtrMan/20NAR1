// interactive NAR

use std::rc::Rc;

use Tv::*;
use Term::*;
use NarseseParser::parse0;
use NarSentence::*;

pub struct Nar {
    pub mem:Mem2, // actual (declarative) memory
}

pub fn createNar() -> Nar {
    Nar{mem:createMem2()}
}

pub fn inputT(nar:&mut Nar, term:&Term) {
    println!("[v] input {}", convTermToStr(term));
    println!("TODO - parse puncation and TV");

    let sentence = SentenceDummy{
        isOp:false,
        term:Rc::new(term.clone()),
        tv:Tv{f:1.0,c:0.9},
        t:-1, // time of occurence 
        punct:EnumPunctation::JUGEMENT, // BUG - we need to compute punctation in inference
    };

    memAddTask(&mut nar.mem, &sentence, true);
}

// input narsese
pub fn inputN(nar:&mut Nar, n:&String) {
    let parseRes = parse0(n);
    match parseRes {
        Ok((str_, term)) => {
            inputT(nar, &term);
        },
        Err(_) => {
            println!("! parse error");
        },
    }
}

use std::io;
use NarWorkingCycle::*;

pub fn runInteractive(nar:&mut Nar) {
    // TODO< parse commands ! >
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                println!("{}", input);
                inputN(nar, &input);
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
