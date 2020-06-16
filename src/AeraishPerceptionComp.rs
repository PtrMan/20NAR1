// AERA'ish perception component based on Kristinn's paper on attention mechanisms for AGI

// a perceived element
#[derive(Clone)]
pub struct PerceptItem<T> {
    pub dat:T, // actual data
    pub salience:f64, // how much "base priority" has this item? [0.0;1.0), can be updated by processes
    pub novelity:f64, // how novel is this item? range [0.0;1.0)
}

// used to limit attention to the top values
pub fn limit<T: std::clone::Clone>(arr:&Vec<PerceptItem<T>>, maxLen:usize) -> Vec<PerceptItem<T>> {
    if arr.len() > maxLen {
        arr[..maxLen].to_vec()
    }
    else {
        arr.to_vec()
    }
}
