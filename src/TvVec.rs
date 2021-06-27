use crate::Tv::*;

// revise vector
pub fn revVec(a: &[Tv], b: &[Tv]) -> Vec<Tv> {
	a.iter().zip(b.iter()).map(|(&a, &b)| rev(&a, &b)).collect()
}

// fold to single TV
pub fn foldVec(a: &[Tv]) -> Tv {
	let mut res = Tv{f:0.0,c:0.0};
    for ia in a {
        res = rev(&res, &ia);
    }
    res
}

/* commented because its prbably not the way to go
// intersection of vectors, useful for classification
pub fn intVec(a: &[Tv], b: &[Tv]) -> Vec<Tv> {
	a.iter().zip(b.iter()).map(|(&a, &b)| int(&a, &b)).collect()
}
*/

// useful for classification
pub fn compVec(a: &[Tv], b: &[Tv]) -> Vec<Tv> {
	a.iter().zip(b.iter()).map(|(&a, &b)| comp(&a, &b)).collect()
}