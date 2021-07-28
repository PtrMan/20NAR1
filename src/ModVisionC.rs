use crate::Map2d::*;

/// convolution witout border
pub fn convWithoutBorder(v:&Map2d<f64>, kernel:&Map2d<f64>) -> Map2d<f64> {
    let mut res:Map2d<f64> = makeMap2d(v.h,v.w);
    
    for iy in 0+(kernel.h/2+1)..v.h-(kernel.h/2+1) {
        for ix in 0+(kernel.w/2+1)..v.w-(kernel.w/2+1) {

            let mut acc:f64 = 0.0;

            for idy in -kernel.h/2..kernel.h/2 {
                for idx in -kernel.w/2..kernel.w/2 {
                    let kx = kernel.w/2+idx; // kernel x
                    let ky = kernel.h/2+idy;
                    
                    acc += readAt(kernel,ky,kx)*readAt(v,idy+iy,idx+ix);
                }
            }

            writeAt(&mut res,iy,ix,acc);
        }
    }
    res
}