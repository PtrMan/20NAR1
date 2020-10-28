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
* rust
## when using python binding
* python 3.X, recommended 3.5 and up
## robotics
Robot examples need pybullet, install with `pip install pybullet`
