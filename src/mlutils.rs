// utilities for ML

pub fn vecAddScale(a: &[f64], b: &[f64], w:f64) -> Vec<f64> {
    let mut res = vec![0.0;a.len()];
    for idx in 0..a.len() {
        res[idx] = a[idx]+b[idx]*w;
    }
    res
}

pub fn vecAdd(a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut res = vec![0.0;a.len()];
    for idx in 0..a.len() {
        res[idx] = a[idx]+b[idx];
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

// compute distance between two vectors
/* commented because not used
pub fn vecDist(a: &[f64], b: &[f64]) -> f64 {
    let mut acc = 0.0;
    for idx in 0..a.len() {
        acc += (a[idx]-b[idx]).powf(2.0);
    }
    acc.sqrt()
}
*/

pub fn vecDot(a: &[f64], b: &[f64]) -> f64 {
    let mut acc = 0.0;
    for idx in 0..a.len() {
        acc += a[idx]*b[idx];
    }
    acc
}

pub fn vecLen(v: &[f64]) -> f64 {
    let mut acc = 0.0;
    for idx in 0..v.len() {
        acc += v[idx].powf(2.0);
    }
    acc.sqrt()
}

// https://www.sciencedirect.com/topics/computer-science/cosine-similarity
pub fn calcCosineSim(a: &[f64], b: &[f64]) -> f64 {
    vecDot(&a,&b) / (vecLen(&a) * vecLen(&b))
}