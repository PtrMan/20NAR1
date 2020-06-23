// finds out if narsese has tv and returns TV if TV exists
// cuts away narsese of TV if TV is detected
// TODO REFACTOR< return option of TV >
pub fn parseNarseseRetTv(narsese:&mut String, f:&mut f64,c:&mut f64,hasTv:&mut bool) {
    *hasTv = false;
    
    if narsese.chars().nth(narsese.len()-1).unwrap() == '}' {
        for revIdx in 1..narsese.len() { // scan for '{'
            let idx = narsese.len() - revIdx; // compute index from back
            
            if narsese.chars().nth(idx).unwrap() == '{' {
                let tvStr = &narsese[idx+1..narsese.len()-1];
                let splitted:Vec<&str> = tvStr.split(" ").collect();
                
                if splitted.len() == 2 { // must have two values
                    // TODO< handle error better >
                    *f = splitted[0].parse::<f64>().unwrap();
                    *c = splitted[1].parse::<f64>().unwrap();
                    *hasTv = true;
                    *narsese = narsese[..idx].to_string(); // cut away
                    return;
                }
                return;
            }
        }
    }
}

/*
pub fn main() {
    let mut f = 0.0;
    let mut c = 0.0;
    let mut hasTv = false;
    let mut narsese = "<a-->b>. {1.0 0.9}".to_string();
    parseNarseseRetTv(&mut narsese, &mut f, &mut c, &mut hasTv);
    println!("{}", &narsese);
    println!("f = {}", f);
    println!("c = {}", c);
    
}*/
