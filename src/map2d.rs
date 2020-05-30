

#[derive(Debug, Clone)]
pub struct Map2d<T> {
    pub arr:Vec<T>,
    pub w:i32, // width
    pub h:i32, // height
}

pub fn readAt<T:Copy+Default>(m:&Map2d<T>, y:i32,x:i32) -> T {
    if y<0||x<0||x>=m.w||y>=m.h {
        return T::default();
    }
    m.arr[(y*m.w+x) as usize]
}

pub fn writeAt<T>(m:&mut Map2d<T>, y:i32,x:i32,v:T) -> () {
    if y<0||x<0||x>=m.w||y>=m.h {
        return;
    }
    m.arr[(y*m.w+x) as usize] = v;
}

// helper to draw a box
pub fn map2dDrawBox<T:Copy>(m:&mut Map2d<T>, cx:i32,cy:i32,w:i32,h:i32,v:T) {
    for iy in 0..h {
        for ix in 0..w {
            writeAt(m,cy+iy,cx+ix, v);
        }
    }
}

// helper to draw a circle
pub fn map2dDrawCircle<T:Copy>(m:&mut Map2d<T>, cx:i32,cy:i32,r:i32,v:T) {
    for iy in -(r+1)..r+1 {
        for ix in -(r+1)..r+1 {
            if ix*ix+iy*iy <= r {
                writeAt(m,cy+iy,cx+ix, v);
            }
        }
    }
}

pub fn drawLine<T:Copy>(m:&mut Map2d<T>, ax:i32,ay:i32,bx:i32,by:i32,v:T) {
    let fax = ax as f64;
    let fay = ay as f64;
    let fbx = bx as f64;
    let fby = by as f64;
    let fdiffx = fbx-fax;
    let fdiffy = fby-fay;

    let fdiffLen = (fdiffx*fdiffx+fdiffy*fdiffy).sqrt();
    let nfdiffx = fdiffx/fdiffLen;
    let nfdiffy = fdiffy/fdiffLen;
    
    for iy in 0..m.h {
        for ix in 0..m.w {
            let ofX = (ix as f64) - (ax as f64); // offset from a
            let ofY = (iy as f64) - (ay as f64); // offset from a

            //let ofLen = (ofX*ofX+ofY*ofY).sqrt();
            //let nofX = ofX / ofLen;
            //let nofY = ofY / ofLen;


            let dotRes = ofX*nfdiffx+ofY*nfdiffy;

            let isOnLine = dotRes >= 0.0 && dotRes <= fdiffLen; // is the pixel between the points?
            if !isOnLine {
                continue;
            }

            // projected
            let px = (ax as f64) + dotRes*fdiffx;
            let py = (ay as f64) + dotRes*fdiffy;

            let diff2x = (ix as f64) - px;
            let diff2y = (iy as f64) - py;

            let mut d:f64 = (diff2x*diff2x + diff2y*diff2y).sqrt(); // compute distance from line
            if d > 1.5 {
                continue; // not on line
            }

            writeAt(m,iy,ix, v);
        }
    }
}
