# 
This is a NARS+ General Machine Intelligence (GMI) system.

# commands
The system accepts either Narsese or commands as inputs. Commands are used to give the reasoner compute in the form of cycles. Commands also can be used to change parameters of the input/output and/or the reasoner itself. Commands also allow to invoke special functionality of the NARS+ implementation, for example for the NLP module.
## modules
This implementation provides a NLP/NLU "module" which is exposed with the `!nlp` command. See NAL experiments which are related to NLP on how to use it. The reasoning for the module is done with a NAR.

# how to run
Running the program is easy. Note that `cargo test` can be omitted if the unittests should be skipped.
## how to run in interactive mode
`cargo test && cargo run --release it`
## how to run in server mode
`cargo test && cargo run --release srv`<br />
connect to server with for example netcat: `nc 127.0.0.1 2039`<br />
## how to run pong
`cargo test && cargo run --release pong3`
## build documentation
`cargo doc --lib`

# dependencies
## basic
* rust (minimum 1.36, latest stable recommended)
## when using python binding
* python 3.X, recommended 3.5 and up
## robotics
Robot examples need pybullet, install with `pip install pybullet`
