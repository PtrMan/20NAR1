use ::map2d;
use ::map2d::{Map2d,readAt};

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

// flattens a float map to a f64 array
pub fn flattenMap(m:&Map2d<f64>) -> Vec<f64> {
    let mut res = vec![];
    for iy in 0..m.h {
        for ix in 0..m.w {
            res.push(readAt(&m,iy,ix));
        }
    }
    res
}