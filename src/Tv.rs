#[derive(Clone)]
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
