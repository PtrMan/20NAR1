// utilities for ML

pub fn vecAddScale(a: &[f64], b: &[f64], w:f64) -> Vec<f64> {
    let mut res = vec![0.0;a.len()];
    for idx in 0..a.len() {
        res[idx] = a[idx]+b[idx]*w;
    }
    res
}

pub fn vecScale(a: &[f64], w:f64) -> Vec<f64> {
    let mut res = vec![0.0;a.len()];
    for idx in 0..a.len() {
        res[idx] = a[idx]*w;
    }
    res
}
