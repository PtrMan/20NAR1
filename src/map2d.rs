

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