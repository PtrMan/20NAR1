// classifier

use ::map2d::{Map2d};
use ::mlutils;
use ::Misc::{flattenMap};

pub struct Classifier {
    pub categories:Vec<Category>,
}

pub struct Category {
    v:Vec<f64>, // flattened vector
}

pub fn classify(c:&Classifier,m:&Map2d<f64>) -> i64 {
    //let iSubImg = map2d::crop(&perceptDirMap, ix, iy, subImgSize, subImgSize); // crop the motion map

    let iSubImgMapFlat:Vec<f64> = flattenMap(m); // flatten map to real vector

    let mut bestSim:f64 = -1.0;
    let mut bestCat:i64 = -1;

    // classify for this cropped image from the motion map
    for iCatIdx in 0..c.categories.len() {
        let iCat = &c.categories[iCatIdx];

        let vecA:&Vec<f64> = &iSubImgMapFlat;
        let vecB:&Vec<f64> = &iCat.v;
        
        let sim0 = mlutils::calcCosineSim(&vecA,&vecB);
        
        if sim0 > bestSim {
            bestSim = sim0;
            bestCat = iCatIdx as i64;
        }
        
        //debug
        //println!("cat[{}] {}", iCatIdx, sim0);
    }

    bestCat
}
