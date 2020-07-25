// interactive NAR

use Term::*;
use NarseseParser::parse0;

pub struct Nar {
}

pub fn inputT(nar:&mut Nar, t:&Term) {
    println!("[v] input {}", convTermToStr(t));
    println!("TODO - process term!");
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

pub fn runInteractive(nar:&mut Nar) {
    // TODO< parse commands ! >
    while true {
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
