//! utilities which don't fit into another file

/// ensures something or panics
// idea is taken from D language
pub fn enforce(v:bool) {
    if !v {
        panic!("");
    }
}