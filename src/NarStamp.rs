/// Stamp which keeps track of the evidence.
///
/// is used to prevent overlap. See NARS theory
#[derive(Debug, Clone)]
pub struct Stamp {
    pub ids:Vec<i64>,
}

pub fn newStamp(ids:&Vec<i64>) -> Stamp {
    Stamp{ids:ids.clone()}
}

pub fn checkOverlap(a:&Stamp,b:&Stamp)->bool {
    for ia in &a.ids {
        for ib in &b.ids {
            if ia == ib {
                return true;
            }
        }
    }
    false
}

pub fn merge(a:&Stamp, b:&Stamp) -> Stamp {
    let mut res = Stamp{ids:vec![]};
    let mut idx=0;
    while idx < a.ids.len().min(b.ids.len()) {
        res.ids.push(a.ids[idx]);
        res.ids.push(b.ids[idx]);
        idx+=1;
    }
    res.ids.extend(&a.ids[idx..a.ids.len()]);
    res.ids.extend(&b.ids[idx..b.ids.len()]);

    res.ids = res.ids[0..(res.ids.len()).min(120)].to_vec(); // AIKR

    res
}

pub fn checkSame(a:&Stamp, b:&Stamp) -> bool {
    if a.ids.len() != b.ids.len() {
        return false; // can't be the same!
    }
    let mut idx=0;
    while idx < a.ids.len() {
        if a.ids[idx] != b.ids[idx] {
            return false;
        }
        idx+=1;
    }
    true
}

/// convert stamp to string
pub fn convToStr(s:&Stamp) -> String {
    let mut res = format!("{}", s.ids[0]);
    for iid in &s.ids[1..s.ids.len()] {
        res = format!("{},{}", res, iid);
    }
    res
}