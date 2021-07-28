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


/**
 * \param phi angle in radiants
 * \param spartialRatioAspect ellipticity of the support of the Gabor function
 */
 pub fn generateGaborKernel(width: i32, phi:f64, lambda:f64, phaseOffset:f64, spartialRatioAspect:f64) -> Map2d<f64> {
    
    // constant from http://bmia.bmt.tue.nl/Education/Courses/FEV/course/pdf/Petkov_Gabor_functions2011.pdf
    let sigma: f64 = 0.56 * lambda;

    let mut result_map = makeMap2d(width, width);

    for yInt in 0..width {
        for xInt in 0..width {
            let x: f64 = ((xInt - width / 2) as f64/width as f64) * 2.0;
            let y: f64 = ((yInt - width / 2) as f64/width as f64) * 2.0;

            let xTick: f64 = x * phi.cos() + y * phi.sin();
            let yTick: f64 = -x * phi.sin() + y * phi.cos();

            let insideExp: f64 = -(xTick*xTick + spartialRatioAspect*spartialRatioAspect * yTick*yTick)/(2.0*sigma*sigma);
            let insideCos: f64 = 2.0*std::f64::consts::PI * (xTick/lambda) + phaseOffset;

            let filterValue: f64 = insideExp.exp()*insideCos.cos();

            writeAt(&mut result_map, yInt,xInt,filterValue);
        }
    }

    result_map
}