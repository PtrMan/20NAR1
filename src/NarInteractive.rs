// interactive NAR

use std::io;
use Nar::*;
use NarWorkingCycle::{debugCreditsOfTasks};

pub fn runInteractive(nar:&mut Nar) {
    // TODO< parse commands ! >
    loop {
        let mut input2 = String::new();
        match io::stdin().read_line(&mut input2) {
            Ok(_) => {
                let mut input = input2.clone();
                trimNewline(&mut input);
                
                println!("{}", input);
                if input.len() >= 2 && &input[..2] == "!s" {
                    let mut nCycles = 1;
                    if input.len() > 2 { // parse number of cycles
                        // TODO< check if it was parsed fine! >
                        nCycles = input[2..].parse::<i64>().unwrap();
                    }
                    for _i in 0..nCycles {
                        cycle(nar);
                    }
                }
                else if input == "!dt" { // debug tasks
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