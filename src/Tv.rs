#[derive(Clone,Copy)]
pub struct Tv {
    pub f:f64,
    pub c:f64,
}

pub fn calcExp(tv:&Tv)->f64 {
    tv.c*(tv.f - 0.5)+0.5
}

pub fn ded(a:&Tv,b:&Tv)->Tv {
    let f = a.f*b.f;
    let c = a.c*b.c*f;
    Tv{f:f,c:c}
}

// special function of ded for goals
// as suggested by Patrick on ~19.04.2021 in #nars
// /param a TV of goal
// /param b TV of belief
pub fn dedGoal(a:&Tv,b:&Tv)->Tv {
    let res1: Tv = ded(a, b);
    let res2: Tv = neg(&ded(&neg(a), b));
    if res1.c >= res2.c {res1} else {res2}
}

pub fn abd(a:&Tv,b:&Tv)->Tv {
    let w = b.f*a.c*b.c;
    let f = a.f;
    let c = w2c(w);
    Tv{f:f,c:c}
}

pub fn ind(a:&Tv,b:&Tv)->Tv {
    abd(b,a)
}

pub fn rev(a:&Tv,b:&Tv)->Tv {
    let w1:f64 = c2w(a.c);
    let w2:f64 = c2w(b.c);
    let w:f64 = w1 + w2;
    let mut f:f64 = 0.0;
    if w > 0.0 {
        f = (w1 * a.f + w2 * b.f) / w;
    }
    let c:f64 = w2c(w);
    Tv{f:f,c:c}
}

pub fn comp(a:&Tv,b:&Tv)->Tv {
    let f0 = or(a.f, b.f);
    let f = if f0.abs()<1.0e-8 {0.0} else { (a.f*b.f) / f0 };
    let c = w2c(f0 * a.c * b.c);
    Tv{f:f,c:c}
}

// intersection
pub fn int(a:&Tv,b:&Tv)->Tv {
    Tv{f:a.f*b.f,c:a.c*b.c}
}

pub fn neg(a:&Tv)->Tv {
    Tv{f:1.0-a.f,c:a.c}
}

pub fn w2c(w:f64) -> f64 {
    let h=1.0;
    w / (w + h)
}

pub fn c2w(c:f64) -> f64 {
    let h=1.0;
    h * c / (1.0 - c)
}

pub fn or(a:f64,b:f64) -> f64 {
    1.0 - (1.0 - a) * (1.0 - b)
}

pub fn convToStr(tv:&Tv) -> String {
    format!("{{{} {}}}", tv.f,tv.c)
}
