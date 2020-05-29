
// automatic differentiation
#[derive(Debug, Clone)]
pub struct Ad {
    pub r: f64,
    pub d: f64,
}

pub fn add(a: &Ad,b: &Ad) -> Ad {
    Ad{r:a.r+b.r,d:a.d+b.d}
}

pub fn sub(a: &Ad,b: &Ad) -> Ad {
    Ad{r:a.r-b.r,d:a.d-b.d}
}

pub fn mul(a: &Ad,b: &Ad) -> Ad {
    Ad{r:a.r*b.r,d:a.r*b.d+a.d*b.r}
}

pub fn div(a: &Ad,b: &Ad) -> Ad {
    let z=a.r*b.d+a.d*b.r;
    Ad{r:a.r/b.r,d:z/(b.r*b.r)}
}

pub fn exp(v:&Ad) -> Ad {
    let e = v.r.exp();
    Ad{r:e,d:v.d*e}
}

pub fn dot(v:&[Ad], w:&[Ad]) -> Ad {
    let mut acc = mul(&v[0],&w[0]);
    for i in 1..w.len() {
        acc = add(&mul(&v[i],&w[i]), &acc);
    }
    return acc;
}

// ReLU activation function
pub fn reluAct(v:&Ad) -> Ad {
    if v.r > 0.0 {
        Ad{r:v.r,d:v.d}
    }
    else {
        Ad{r:0.0,d:0.0}
    }
}

pub fn sigmoidAct(v:&Ad) -> Ad {
    let z=exp(&v);
    div(&z, &add(&z, &Ad{r:1.0,d:0.0}))
}

#[derive(Debug, Clone)]
pub struct Neuron {
    pub weights: Vec<Ad>,
    pub bias:Ad,
    pub act: u32,
}

// calc activation of neuron
pub fn calc(inAct:&[Ad], n:&Neuron) -> Ad {
    let act = add(&dot(&inAct, &n.weights), &n.bias);
    let act2 = if n.act == 0 {reluAct(&act)} else {sigmoidAct(&act)};
    return act2;
}
