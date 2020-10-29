// interactive NAR

use std::io;
use crate::Nar::*;
use crate::NarInputFacade;

/// run interactive loop until process is terminated by user or until the interactive session is closed
pub fn runInteractive(nar:&mut Nar) {
    let repeatLastInput = true; // repeat command by just pressing enter with empty input?

    let mut lastInput = "".to_string(); // used to repeat command by just pressing enter with empty input
    loop {
        let mut input2 = String::new();
        match io::stdin().read_line(&mut input2) {
            Ok(_) => {
                let mut input = input2.clone();
                trimNewline(&mut input);
                
                if repeatLastInput {
                    if input == "" {
                        input = lastInput.clone();
                    }
                    else {
                        lastInput = input.clone(); // store for last input
                    }
                }

                println!("{}", input);

                let mut quit = false;
                let resLines: Vec<String> = NarInputFacade::input(nar, &input, &mut quit);
                for iLine in &resLines {
                    println!("{}", iLine);
                }
                
                if quit {
                    break;
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