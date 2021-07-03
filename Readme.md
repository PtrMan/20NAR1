This is a NARS inspired General Machine Intelligence (GMI) system. The current implementation only implements a NAR(non-axiomatic reasoner) for NAL1-6 and NAL7-8. NAL-9 is currently only supported with a special operator.

working definition of intelligence:<br />
`solving complex and complicated problems under  adaptation with insufﬁcient knowledge and resources`<br />
is a composition of the definition of intelligence from Dr. Pei Wang [paper](https://sciendo.com/article/10.2478/jagi-2019-0002) and the definition of intelligence from Dr. Ben Goertzel.

# how to run
Running the program is easy. Note that `cargo test` can be omitted if the unittests should be skipped.
## how to run in interactive mode
`cargo test && cargo run --release it`
## how to run in server mode
`cargo test && cargo run --release srv`<br />
connect to server with for example netcat: `nc 127.0.0.1 2039`<br />
## how to run pong
`cargo test && cargo run --release envPong3`
## how to eval Q&A performance for one file
`cargo test && cargo run --release bQA nalExp/ExTimAnimalProd0.nal`
## build documentation
`cargo doc --lib`
## how to train the NN for NLP
`rm -f ./logX.txt && cargo test && cargo run --release nnTrain.nlp`

# Documentation
Documentation will follow when the features(mechanisms) and implementation has settled to a "stable" state.<br />
A paper describing novel parts will follow.<br />
The API documentation can be created from the source code by building the documentation.

# commands
The system accepts either Narsese or commands as inputs. Commands are used to give the reasoner compute in the form of cycles. Commands also can be used to change parameters of the input/output and/or the reasoner itself. Commands also allow to invoke special functionality of the NARS+ implementation, for example for the NLP module.<br />
See in source file [NarInputFacade.rs](https://github.com/PtrMan/20NAR1/blob/master/src/NarInputFacade.rs#L34) for a list of all implemented commands.


# dependencies
## basic
* rust (minimum 1.36, latest stable recommended)
## when using python binding
* python 3.X, recommended 3.5 and up
## robotics
Robot examples need pybullet, install with `pip install pybullet`

# what is implemented
## temporal
* unification of =/> by subj
* goals
* decision making
* Q&A for temporal, ex: <(a,^x) =/> b>?
## non-temporal
* Q&A
# what is missing?
* unification of =/> by pred
* use of non-temporal knowledge for temporal inference    *halfdone*
* quests
