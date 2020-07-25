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
    res.ids.extend(&a.ids[idx+1..a.ids.len()]);
    res.ids.extend(&b.ids[idx+1..b.ids.len()]);

    println!("TODO - limit length of stamp");

    res
}

