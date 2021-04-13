// util to read narsese file
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::Nar::*;
use crate::NarInputFacade;

// see/from https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html

pub fn readNarseseFile(nar: &mut Nar, path:&String, quit:&mut bool) {
    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines(path) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(line2) = line {
                NarInputFacade::input(nar, &line2, quit); // pass line into facade
                if *quit {
                    break; // break reading of file because we are exiting anyways
                }
            }
        }
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}