// interactive NAR

use std::rc::Rc;

use Tv::*;
use Term::*;
use NarseseParser::parse0;
use NarSentence::*;
use NarStamp::*;

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
        stamp:newStamp(&vec![nar.mem.stampIdCounter]),
        t:-1, // time of occurence 
        punct:EnumPunctation::JUGEMENT, // BUG - we need to compute punctation in inference
    };
    nar.mem.stampIdCounter+=1;

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

pub fn cycle(nar:&mut Nar) {
    reasonCycle(&mut nar.mem);
}

use std::io;
use NarWorkingCycle::*;

pub fn runInteractive(nar:&mut Nar) {
    // TODO< parse commands ! >
    loop {
        let mut input2 = String::new();
        match io::stdin().read_line(&mut input2) {
            Ok(_) => {
                let mut input = input2.clone();
                trimNewline(&mut input);
                
                println!("{}", input);
                if input == "s" {
                    cycle(nar);
                }
                else if input == "dt" { // debug tasks
                    debugCreditsOfTasks(&nar.mem);
                }
                else {
                    inputN(nar, &input);
                }
                

            }
            Err(error) => println!("error: {}", error),
        }
    }
}

fn trimNewline(s: &mut String) {
    // from https://blog.v-gar.de/2019/04/rust-remove-trailing-newline-after-input/
    while s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
    }
}