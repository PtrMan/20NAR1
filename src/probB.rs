// small test program to select instructions by probability

pub fn main() {
    let mut probUpdateRate = 0.1;

    // init probability of instructions
    let mut z = Vec::new();
    for i_ in 0..10 {
        z.push(0.5);
    }

    z[0] *= 0.8;
    
    println!("z[0] {}",z[0]);
    
    {
        let idx = sel(&z, 0.6);    
        z[idx] = probUpdate(z[idx],0.0,probUpdateRate);
    }

    {
        let idx = sel(&z, 0.6);    
        z[idx] = probUpdate(z[idx],0.0,probUpdateRate);
    }
    
    {
        let idx = sel(&z, 0.6);    
        z[idx] = probUpdate(z[idx],0.0,probUpdateRate);
    }

    println!("z[7] {}",z[7]);
    println!("z[6] {}",z[6]);
    
    
}

// select instruction
// /param selVal selection value between 0.0 and 1.0
pub fn sel(arr:&Vec<f64>, selVal:f64) -> usize {
    // select one instruction
    let mass:f64 = arr.iter().sum();
    
    let chosenMass:f64 = mass * selVal;
    
    let mut acc=0.0;
    let mut idx=0;
    while acc < chosenMass {
        acc+=arr[idx as usize];
        idx+=1;
    }
    
    println!("idx = {}",idx);
    
    idx
}

pub fn probUpdate(a:f64,b:f64,rate:f64)->f64{
    a*(1.0-rate) + (b)*(rate)
}
