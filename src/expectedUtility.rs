fn main() {
    let lRate:f64 = 0.1;
  
    // probability and utility array:
    let mut pArr = vec![0.5, 0.3, 0.99];
    let mut utilArr = vec![20.0, 18.0, 3.1];
    
    for i in 0..50 {
      let mut expUtilArr = vec![];
      for idx in 0..pArr.len() {
        expUtilArr.push(pArr[idx]*utilArr[idx]);
      }
  
      let mut maxIdx=0;
      let mut maxUtil=expUtilArr[0];
      for idx in 0..expUtilArr.len() {
        if expUtilArr[idx] > maxUtil {
          maxIdx = idx;
          maxUtil = expUtilArr[idx];
        }
      }
  
  
      println!("pick[{}]", maxIdx);
      // failed adapt
      pArr[maxIdx] = probUpdate(pArr[maxIdx], 0.0, lRate); // update probability
  
      // renormalize
      {
        let mut sumProb = 0.0;
        for iP in &pArr {
          sumProb += iP;
        }
        for idx in 0..pArr.len() {
          pArr[idx]/=sumProb;
        }
      }
      
      for idx in 0..expUtilArr.len() {
        println!("[{}] p={} expUtil={}",idx,pArr[idx],expUtilArr[idx]);
      }
    }
  }
  
  pub fn probUpdate(a:f64,b:f64,rate:f64)->f64{
    a*(1.0-rate) + (b)*(rate)
  }