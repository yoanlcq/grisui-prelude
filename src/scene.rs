// Scene = Gameplay scene.

// File format:
//
// I source_shape_name instance_name
// P 1 2 3
// R 90
// S 1 1
//
// I = Begin Instance <shape source> <instance name>
// P = Override position (DOES include Z)
// R = Override rotation in degrees (convenience)
// S = Override scale.
//
//
// First :
// - Create a Scene from memory;
// - Switch to it via F8;
// - Save it to disk.

use std::io;
use xform::Xform2D;
use v::{Vec2, Vec3};

#[derive(Debug, Default, Clone)]
pub struct ShapeInstance {
    pub source_shape_name: String,
    pub name: String,
    pub xform: Xform2D,
}

#[derive(Debug, Default, Clone)]
pub struct Scene {
    pub instances: Vec<ShapeInstance>,
}

impl Scene {
    pub fn save(&self, f: &mut io::Write) -> io::Result<()> {
        for instance in self.instances.iter() {
            let &ShapeInstance {
                ref source_shape_name, ref name,
                xform: Xform2D {
                    position: Vec3 { x, y, z },
                    rotation_z_radians,
                    scale: Vec2 { x: sx, y: sy },
                },
            } = instance;
            writeln!(f, "I {} {}", source_shape_name, name)?;
            writeln!(f, "P {} {} {}", x, y, z)?;
            writeln!(f, "R {}", rotation_z_radians.to_degrees())?;
            writeln!(f, "S {} {}", sx, sy)?;
            writeln!(f)?;
        }
        Ok(())
    }
    pub fn load(f: &mut io::Read) -> io::Result<Self> {
        let data = {
            let mut buf = String::new();
            f.read_to_string(&mut buf)?;
            buf
        };
        let mut scene = Self::default();
        let mut words = data.split_whitespace();
        while let Some(cmd) = words.next() {
            match cmd {
                "I" => {
                    scene.instances.push(ShapeInstance::default());
                    scene.instances.last_mut().unwrap().source_shape_name = words.next().unwrap().to_owned();
                    scene.instances.last_mut().unwrap().name = words.next().unwrap().to_owned();
                },
                "P" => {
                    let p = &mut scene.instances.last_mut().unwrap().xform.position;
                    p.x = words.next().unwrap().parse().unwrap();
                    p.y = words.next().unwrap().parse().unwrap();
                    p.z = words.next().unwrap().parse().unwrap();
                },
                "R" => {
                    let degrees: f32 = words.next().unwrap().parse().unwrap();
                    scene.instances.last_mut().unwrap().xform.rotation_z_radians = degrees.to_radians();
                },
                "S" => {
                    let s = &mut scene.instances.last_mut().unwrap().xform.scale;
                    s.x = words.next().unwrap().parse().unwrap();
                    s.y = words.next().unwrap().parse().unwrap();
                },
                _ => unimplemented!{},
            }
        }
        Ok(scene)
    }
}

