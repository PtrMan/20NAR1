#![allow(non_snake_case)]
#![allow(dead_code)]

use ::mlutils;
use ::map2d;
use ::map2d::{Map2d,writeAt,readAt,map2dDrawCircle};

// TODO< draw a object in motion and display result maxima classifications >

// TODO< print visualization of the result with the highest ranked category to console >

// TODO< discremenate between different classifications my keeping track of max >

// TODO< use NN and train NN to filter results from primitive classifier! >

// experiment for classification of motion in image
// current result: doesn't work because it fids similarity to not moved circle
pub fn expMotionClassifier0() {
    let subImgSize = 16;


    let mut categories:Vec<Vec<f64>> = vec![]; // vector with flattened map2d's of categories

    // array of movement directions of background
    let bgMotionDirs = &[
        Vec2{x:0.001,y:0.001},
        Vec2{x:0.1,y:0.001},
        Vec2{x:-0.1,y:0.001},
        Vec2{x:0.001,y:0.1},
        Vec2{x:0.001,y:-0.1}];

    // fill with uniform movement info without any object in front
    for iBackgroundMotion in bgMotionDirs {
        let catMap: Map2d<Vec2> = Map2d{arr:vec![*iBackgroundMotion;16*16],w:16,h:16};
        categories.push(flattenVec2Map(&catMap));
    }

    let nBgMotionCats = categories.len() as i32; // number of background motion categories

    // with with movements when a object is in front
    for iBgMotionIdx in 0..bgMotionDirs.len() {
        let iBgMotion = &bgMotionDirs[iBgMotionIdx];

        for iFgMotionIdx in 0..bgMotionDirs.len() {
            let iFgMotion = &bgMotionDirs[iFgMotionIdx];
            if iBgMotionIdx != iFgMotionIdx { // we can't differentiate between a object which is moving with the same velocity as the background!
                let mut catMap: Map2d<Vec2> = Map2d{arr:vec![*iBgMotion;16*16],w:16,h:16};

                // draw object, in this case a circle
                let w = catMap.w;
                let h = catMap.h;
                map2dDrawCircle(&mut catMap, w/2,h/2,w/3,*iFgMotion);

                categories.push(flattenVec2Map(&catMap));
            }
        }
    }
    


    let perceptDirMap:Map2d<Vec2> = Map2d{arr:vec![Vec2{x:0.06,y:0.001};32*32],w:32,h:32};



    let mut clfnMaps:Vec<Map2d<f64>> = vec![];
    for _i in 0..categories.len() {
        clfnMaps.push(Map2d::<f64>{arr:vec![0.0;(perceptDirMap.w*perceptDirMap.h) as usize],w:perceptDirMap.w,h:perceptDirMap.h});
    }


    for iy in 0..perceptDirMap.h-subImgSize {
        for ix in 0..perceptDirMap.w-subImgSize {


            let iSubImg = map2d::crop(&perceptDirMap, ix, iy, subImgSize, subImgSize); // crop the motion map

            let iSubImgMapFlat:Vec<f64> = flattenVec2Map(&iSubImg); // flatten map to real vector

            // classify for this cropped image from the motion map
            for iCatIdx in 0..categories.len() {
                let iCat = &categories[iCatIdx];

                let vecA:&Vec<f64> = &iSubImgMapFlat;
                let vecB:&Vec<f64> = iCat;
                
                let sim0 = mlutils::calcCosineSim(&vecA,&vecB);
                
                /* commented because not necessary because it didn't work
                // represent as boolean vectors
                let mut vecABool:Vec<bool> = convRealVecToQuantizedBoolVec(&iSubImgMapFlat);
                let mut vecBBool:Vec<bool> = convRealVecToQuantizedBoolVec(&vecB);
                
                // compute similarity based on quantized bool vectors
                // the similarity isn't useful with the extreme quantification
                let sim1:f64 = boolVecSim(&vecABool,&vecBBool);
                */
                
                //debug
                //println!("cat[{}] {}", iCatIdx, sim0);
                writeAt(&mut clfnMaps[iCatIdx], iy,ix, sim0);
            }
        }
    }

    let threshold = 0.2;
    let maxima:Vec<ClsnWVal> = max(&clfnMaps, nBgMotionCats, threshold);



    println!("maxima =");
    for iMaxima in maxima {
        println!("  {:?}", iMaxima);
    }

    println!("DONE");
}

pub fn main() {
    expMotionClassifier0();
}

// helper to convert real vector to quantized bool vector by cutting the interval [0.0;1.0] into different parts
// TODO< refactor: refactor conversion to boolean array with #of ranges >
pub fn convRealVecToQuantizedBoolVec(v:&[f64]) -> Vec<bool> {
    let mut res:Vec<bool> = vec![];
    for iv in v {
        res.push(*iv < 0.5);
        res.push(*iv > 0.5);
    }
    res
}

// helper to compute similarity between bool vectors
pub fn boolVecSim(a: &[bool], b: &[bool]) -> f64 {
    // compute difference
    let mut diffCnt:i64 = 0;
    for idx in 0..a.len() {
        if a[idx] != b[idx] {
            diffCnt+=1;
        }
    }
    let sim1:f64 = 1.0 - (diffCnt as f64) / (a.len() as f64);
    return sim1;
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2{
    pub x:f64,
    pub y:f64,
}

impl Default for Vec2 {
    fn default() -> Vec2 {Vec2{x:0.0,y:0.0}}
}

// flattens a vec2 map to a f64 array
pub fn flattenVec2Map(m:&Map2d<Vec2>) -> Vec<f64> {
    let mut res = vec![];
    for iy in 0..m.h {
        for ix in 0..m.w {
            res.push(readAt(&m,iy,ix).x);
            res.push(readAt(&m,iy,ix).y);
        }
    }
    res
}


// classification with value
#[derive(Debug, Clone, Copy)]
pub struct ClsnWVal {
    pub val:f64, // classification
    pub cls:i64,
    pub x:i32,
    pub y:i32,
}

// need this for reading of map
impl Default for ClsnWVal {
    fn default() -> ClsnWVal {ClsnWVal{val:0.0,cls:-1,x:-1,y:-1}}
}


// searches for the maximum classification of different maps
// /param nBgMotionCats number of categories of background
pub fn max(arr:&Vec<Map2d<f64>>, nBgMotionCats:i32, threshold:f64) -> Vec<ClsnWVal> {
    let w = arr[0].w;
    let h = arr[0].h;
    let mut maxMap = Map2d::<Option<ClsnWVal>>{arr:vec![None;(w*h) as usize],w:w,h:h}; // map with current maxima

    if nBgMotionCats == 0 { // ignore background categories
        for iy in 0..h {
            for ix in 0..w {
                let v = readAt(&arr[0],iy,ix);
                writeAt(&mut maxMap, iy,ix, Some(ClsnWVal{val:v,cls:0,x:ix,y:iy}));
            }
        }
    }

    let mut iClass:i64=1;
    for iArr in &arr[1..] {
        if (iClass as i32) > nBgMotionCats { // ignore background categories
            for iy in 0..h {
                for ix in 0..w {
                    let r = readAt(&maxMap,iy,ix);
                    let v = readAt(&iArr,iy,ix);

                    if r.is_some() {
                        let rval = r.unwrap().val;
    
                        
                        if v>rval {
                            writeAt(&mut maxMap, iy,ix, Some(ClsnWVal{val:v,cls:iClass,x:ix,y:iy}));
                        }
                    }
                    else {
                        writeAt(&mut maxMap, iy,ix, Some(ClsnWVal{val:v,cls:iClass,x:ix,y:iy}));
                    }
                }
            }
        }

        iClass+=1;
    }

    // search maxima
    loop { // loop until it doesn't change anymore
        let mut wasChanged = false;

        for iy in 0..h {
            for ix in 0..w {
                let maxV:Option<ClsnWVal> = readAt(&maxMap,iy,ix);
                let maxVX:Option<ClsnWVal> = readAt(&maxMap,iy,ix-1);
                let maxVY:Option<ClsnWVal> = readAt(&maxMap,iy-1,ix);

                if maxV.is_some() && maxVX.is_some() {
                    if maxV.unwrap().val > maxVX.unwrap().val {
                        writeAt(&mut maxMap,iy,ix-1,None);
                        wasChanged = true;
                    }
                    else if maxV.unwrap().val < maxVX.unwrap().val {
                        writeAt(&mut maxMap,iy,ix,None);
                        wasChanged = true;
                    }
                }

                if maxV.is_some() && maxVY.is_some() {
                    if maxV.unwrap().val > maxVY.unwrap().val {
                        writeAt(&mut maxMap,iy-1,ix,None);
                        wasChanged = true;
                    }
                    else if maxV.unwrap().val < maxVY.unwrap().val {
                        writeAt(&mut maxMap,iy,ix,None);
                        wasChanged = true;
                    }
                }
            }
        }


        if !wasChanged {
            break; // terminate if it wasn't changed
        }
    }

    // compute result array
    maxMap.arr.iter().filter(|v| v.is_some()).filter(|v| v.unwrap().val >= threshold).map(|v| v.unwrap()).collect()
}


