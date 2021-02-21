use std::sync::{Arc};
use parking_lot::RwLock;

use crate::NarGoalSystem::*; // need it because it is hardcoded for goals

// binary search
pub fn binSearch(arr:&[Arc<RwLock<Entry>>], val:f64) -> Arc<RwLock<Entry>> {
    //println!("{:?}",&arr);
    if arr.len() == 1 {
        return Arc::clone(&arr[0]);
    }
    let idxMid = arr.len()/2;
    binSearch(if arr[idxMid].read().accDesirability > val {&arr[..idxMid]} else {&arr[idxMid..]}, val)
}

/* testts were done for simple array, but code which was rewritten to access goalentries isn't tested
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn _1() {
        assert_eq!(binSearch(&[1.0], 0.5), 1.0);
    }

    #[test]
    pub fn _2a() {
        assert_eq!(binSearch(&[1.0, 2.0], 1.5), 1.0);
    }

    #[test]
    pub fn _2b() {
        assert_eq!(binSearch(&[1.0, 2.0], 0.5), 1.0);
    }

    #[test]
    pub fn _3a() {
        assert_eq!(binSearch(&[1.0, 2.0, 3.0], 1.5), 1.0);
    }

    #[test]
    pub fn _3b() {
        assert_eq!(binSearch(&[1.0, 2.0, 3.0], 0.5), 1.0);
    }

    #[test]
    pub fn _3c() {
        assert_eq!(binSearch(&[1.0, 2.0, 3.0], 2.5), 2.0);
    }
}*/
