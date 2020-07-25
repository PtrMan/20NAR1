// sphere tracing computer graphics

pub struct Ray {
    pub p:Vec3,
    pub dir:Vec3,
}

pub struct Vec3 {
    pub x:f64,
    pub y:f64,
    pub z:f64,
}

pub fn newVec3(x:f64,y:f64,z:f64)->Vec3 {
    Vec3{x:x,y:y,z:z}
}

pub struct Obj {
    pub objType:EnumObjType, // object type
    pub p:Vec3, // position
    pub v:Vec3, // velocity of object
    pub r:f64, // radius of sphere
}

pub enum EnumObjType {
    SPHERE,
}

pub struct Scene {
    pub objs:Vec<Obj>,
}

pub struct RayInfo {
    pub p:Vec3,
    pub dir:Vec3,
}

// compute distance to closest object
pub fn calc(ray:&RayInfo, scene:&Scene) -> f64 {
    let mut len:f64 = 60000000000.0;
    
    for iObj in &scene.objs {
        match iObj.objType {
            EnumObjType::SPHERE => {
                let d = ((ray.p.x - iObj.p.x).powf(2.0) + (ray.p.y - iObj.p.y).powf(2.0) + (ray.p.z - iObj.p.z).powf(2.0)).sqrt() - iObj.r;
                len = len.min(d);
            }
        }
    }
    
    len
}

pub fn main() {
    let mut scene:Scene = Scene {
        objs:vec![],
    };
    
    scene.objs.push(Obj {
        objType:EnumObjType::SPHERE,
        p:newVec3(3.0, 1.5, 0.0),
        v:newVec3(0.0,0.0,0.0),
        r:1.0,
    });
    
    scene.objs.push(Obj {
        objType:EnumObjType::SPHERE,
        p:newVec3(1.0, -1.0, 0.0),
        v:newVec3(0.0,0.0,0.0),
        r:0.8,
    });
    
    let mut ray:RayInfo = RayInfo {
        p:newVec3(0.0,0.0,0.0),
        dir:newVec3(1.0,0.0,0.0),
    };
    
    println!("Graphics[{{");
    
    // draw ray as line
    println!("  Line[{{{{0,0}},{{6.44,0}}}}],");
    
    // print spheres in scene as 2d circles
    println!("  Blue,");
    println!("  Circle[{{{},{}}},{}],", scene.objs[0].p.x, scene.objs[0].p.y, scene.objs[0].r);
    println!("  Circle[{{{},{}}},{}],", scene.objs[1].p.x, scene.objs[1].p.y, scene.objs[1].r);

    println!("  Black,");
    
    for it in 0..13 {
        // compute distance to next object
        let distToNext:f64 = calc(&ray, &scene);
        
        // draw sphere tracing cirlce
        println!("  Circle[{{{},{}}},{}],", ray.p.x, ray.p.y, distToNext);
        
        // move ray
        ray.p.x += ray.dir.x * distToNext;
        ray.p.y += ray.dir.y * distToNext;
        ray.p.z += ray.dir.z * distToNext;
    }
    
    println!("}}]");

}
