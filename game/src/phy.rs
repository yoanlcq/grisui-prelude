use v::{Lerp, Vec3, Rgba, Simd3, Aabb};
use gl;
use global::{Global, GlobalDataUpdatePack};
use duration_ext::DurationExt;
use gx;
use grx;
use mesh::{self, Mesh};
use scene::SimStates;


// TODO: Somehow make gravity separate (i.e have multiple gravities?)
//
// NOTE: Code du LeapFrog des particules:
// self.vit += dt/self.m*self.frc
// self.pos += dt*self.vit
// self.frc = Vec3:zero()
//
// NOTE: structure liaisons:
// self.M1
// self.M2
// self.frc
// self.col
// self.l = distance(M1, M2)
//
// liaison: setup:
// self.M1.frc += self.frc
// self.M2.frc += self.frc
//
// ressort: setup:
// d = max(epsilon, distance(m1, m2)) // distance inter-masses
// e = 1. - self.l / d  // élongation
// // force de rappel
// self.frc = self.k * e * (vecteur m1 m2)
// Lisaison.setup(self)
//
// algo ressort :
// d = distance(m1 m2);
// f = k * (1 - l/d) * (vecteur m1 m2)
// m1.frc += f;
// m2.frc -= f;
//
// NOTE(potentiel): La gravité c'est une liaison mais elle n'a qu'un m1, et self.frc = g.


#[derive(Debug)]
pub struct Phy {
    pub is_enabled: bool,
    pub gfx_particles: mesh::Particles,
    pub gfx_springs: Mesh,
    pub gfx_aabb: Mesh,
    pub simulation: SimStates<Simulation>,
}

#[derive(Debug, Clone)]
pub struct Simulation {
    pub integrator: Integrator,
    pub g: Simd3<f32>,
    pub air_resistance: f32,
    pub rebound_vel_factor: f32,
    pub friction_vel_factor: f32,
    pub aabb: Aabb<f32>,

    pub particles: Particles,
    pub springs: Springs,
}
#[derive(Debug, Default, Clone)]
pub struct Particles {
    pub frozen_start_index: usize,
    pub pos: Vec<Simd3<f32>>, // position
    pub vel: Vec<Simd3<f32>>, // velocity
    pub frc: Vec<Simd3<f32>>, // force
    pub m: Vec<f32>, // mass
}
#[derive(Debug, Default, Clone)]
pub struct Springs {
    pub m1: Vec<usize>,
    pub m2: Vec<usize>,
    pub l: Vec<f32>,  // rest length
    pub k: Vec<f32>,  // stiffness constant (aka. spring constant)
    pub kd: Vec<f32>, // damping constant
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            integrator: Integrator(Simulation::leapfrog),
            g: Simd3::down() * 0.98,
            air_resistance: 0.,
            rebound_vel_factor: 0.9,
            friction_vel_factor: 0.98,
            aabb: Aabb {
                min: Vec3::new(-0.9, -0.5, 0.),
                max: Vec3::new( 0.9,  0.5, 0.),
            },
            particles: Default::default(),
            springs: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct Integrator(pub fn(&mut Simulation, f32));

use ::std::fmt::{self, Debug, Formatter};
impl Debug for Integrator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_tuple("Integrator").finish()
    }
}

impl Simulation {
    pub fn explicit_euler(&mut self, dt: f32) {
        let p = &mut self.particles;
        for i in 0..p.frozen_start_index {
            p.pos[i] += p.vel[i] * dt;
            p.vel[i] += (self.g - p.vel[i] * (self.air_resistance / p.m[i])) * dt;
            // self.frc[i] = -self.vel[i] * (self.air_resistance / self.m[i]) - self.g;
        }
    }

    pub fn implicit_euler(&mut self, dt: f32) {
        let p = &mut self.particles;
        for i in 0..p.frozen_start_index {
            p.vel[i] = (p.vel[i] + self.g * dt) * (p.m[i] / (p.m[i] + dt*self.air_resistance));
            p.pos[i] += p.vel[i] * dt;
        }
    }

    pub fn leapfrog(&mut self, dt: f32) {
        let p = &mut self.particles;
        let s = &mut self.springs;

        /*
        for i in 0..s.m1.len() {
            let m1m2 = p.pos[s.m2[i]] - p.pos[s.m1[i]];
            let d = m1m2.magnitude();
            let dscale = 1.; // XXX ???
            let scalar = dscale * s.k[i] * (d - s.l[i]);
            let dir = m1m2 / d;
            let s1 = p.vel[s.m1[i]].dot(dir);
            let s2 = p.vel[s.m2[i]].dot(dir);
            let damping_scalar = -s.kd[i] * (s1 + s2);
            let f = dir * (scalar + damping_scalar);
            p.frc[s.m1[i]] += f;
            p.frc[s.m2[i]] -= f;
        }
        */
        for i in 0..s.m1.len() {
            let m1m2 = p.pos[s.m2[i]] - p.pos[s.m1[i]];
            let d = m1m2.magnitude();
            let f = m1m2 * s.k[i] * (1. - s.l[i] / ::v::partial_max(d, 0.001));
            p.frc[s.m1[i]] += f;
            p.frc[s.m2[i]] -= f;
        }

        let Aabb { min, max } = self.aabb;
        for i in 0..p.frozen_start_index {
            if p.vel[i].y < 0. && p.pos[i].y <= min.y { p.pos[i].y = min.y; p.vel[i].y *= -self.rebound_vel_factor; p.vel[i].x *= self.friction_vel_factor; }
            if p.vel[i].y > 0. && p.pos[i].y >= max.y { p.pos[i].y = max.y; p.vel[i].y *= -self.rebound_vel_factor; p.vel[i].x *= self.friction_vel_factor; }
            if p.vel[i].x < 0. && p.pos[i].x <= min.x { p.pos[i].x = min.x; p.vel[i].x *= -self.rebound_vel_factor; p.vel[i].x *= self.friction_vel_factor; }
            if p.vel[i].x > 0. && p.pos[i].x >= max.x { p.pos[i].x = max.x; p.vel[i].x *= -self.rebound_vel_factor; p.vel[i].x *= self.friction_vel_factor; }
        }

        for i in 0..p.frozen_start_index {
            p.frc[i] += self.g; // TODO: Get rid of that

            p.vel[i] += p.frc[i] * dt / p.m[i];
            p.pos[i] += p.vel[i] * dt;
            p.frc[i] = Simd3::zero();
        }
    }
}

impl Phy {
    pub fn new(g: &Global) -> Self {
        let frozen_particle_count = 1*8;
        let unfrozen_particle_count = 7*8;
        let mut simulation = Simulation::default();

        for i in 0..unfrozen_particle_count {
            let x = ((i%8) as f32 / 7.) - 0.5;
            let y = 0.4 - 0.6 * (1. + (i / 8) as f32) / 8.;
            simulation.particles.pos.push(Simd3::new(x, y, 0.));
            simulation.particles.vel.push(Simd3::zero());
            simulation.particles.frc.push(Simd3::zero());
            simulation.particles.m.push(1.);
        }

        simulation.particles.frozen_start_index = unfrozen_particle_count as _;

        for i in 0..frozen_particle_count {
            let x = (i as f32 / 7.) - 0.5;
            let y = 0.4;
            simulation.particles.pos.push(Simd3::new(x, y, 0.));
            simulation.particles.vel.push(Simd3::zero());
            simulation.particles.frc.push(Simd3::zero());
            simulation.particles.m.push(::std::f32::INFINITY);
        }

        {
            let mut attach = |i, j| {
                simulation.springs.m1.push(i);
                simulation.springs.m2.push(j);
                simulation.springs.l.push((simulation.particles.pos[i] - simulation.particles.pos[j]).magnitude());
                simulation.springs.k.push(8.*8.*8.*8.);
                simulation.springs.kd.push(2.);
                // FIXME 0.1/(dt*dt) < k < 1/(dt*dt)
                // FIXME 0 < dampening < 0.1/dt
            };

            for i in 0..7*8 {
                if (i+1)%8 != 0 {
                    attach(i, i+1);
                    /*
                    if i+8+1 < 7*8 {
                        attach(i, i+8+1);
                    }
                    */
                }
                if i+8 < 7*8 {
                    attach(i, i+8);
                }
            }
            for i in 7*8..8*8 {
                attach(i, i%8 + 8);
                /*
                if i < 8*8 - 1 {
                    attach(i, i%8 + 1);
                }
                */
            }
        }

        let mut vertices = Vec::new();
        for (i, pos) in simulation.particles.pos.iter().enumerate() {
            let (point_size, color) = if i < unfrozen_particle_count {
                let r = i as f32 / (unfrozen_particle_count as f32);
                (16_f32, Rgba::new_opaque(r, 0., 0.))
            } else {
                let r = (i - unfrozen_particle_count) as f32 / (frozen_particle_count as f32);
                (8_f32, Rgba::new_opaque(0., 0., r))
            };
            vertices.push(grx::ParticleRenderingVertex {
                position: (*pos).into(),
                color,
                point_size,
            });
        }

        let gfx_springs = Mesh::from_vertices(
            &g.gl_simple_color_program,
            "GfxSprings",
            gx::UpdateHint::Often,
            gl::LINES,
            (0..2*simulation.springs.m1.len()).map(|_| grx::SimpleColorVertex { position: Vec3::new(-1., -0.5, 0.), color: Rgba::red()  } ).collect()
        );

        // XXX: Must initialize AFTER gfx_aabb. I HAVE NO IDEA WHY THOUGH
        let gfx_particles = mesh::Particles::from_vertices(
            &g.gl_particle_rendering_program,
            "GfxParticles",
            vertices
        );

        let gfx_aabb = Mesh::from_vertices(
            &g.gl_simple_color_program,
            "GfxAabb",
            gx::UpdateHint::Never,
            gl::LINE_LOOP,
            {
                let Aabb { min, max } = simulation.aabb;
                vec![
                    grx::SimpleColorVertex { position: Vec3::new(min.x, min.y, 0.), color: Rgba::red()  },
                    grx::SimpleColorVertex { position: Vec3::new(max.x, min.y, 0.), color: Rgba::red()  },
                    grx::SimpleColorVertex { position: Vec3::new(max.x, max.y, 0.), color: Rgba::red()  },
                    grx::SimpleColorVertex { position: Vec3::new(min.x, max.y, 0.), color: Rgba::red()  },
                ]
            }
        );

        let simulation = SimStates {
            previous: simulation.clone(),
            current : simulation.clone(),
            render  : simulation.clone(),
        };

        Self {
            simulation, gfx_particles, gfx_aabb, gfx_springs,
            is_enabled: false,
        }
    }

    pub fn replace_previous_state_by_current(&mut self) {
        let sim = &mut self.simulation;
        for i in 0..sim.previous.particles.pos.len() {
            sim.previous.particles.pos[i] = sim.current.particles.pos[i];
            sim.previous.particles.vel[i] = sim.current.particles.vel[i];
            sim.previous.particles.frc[i] = sim.current.particles.frc[i];
            sim.previous.particles.m  [i] = sim.current.particles.m  [i];
        }
        for i in 0..sim.previous.springs.m1.len() {
            sim.previous.springs.m1[i] = sim.current.springs.m1[i];
            sim.previous.springs.m2[i] = sim.current.springs.m2[i];
            sim.previous.springs.l [i] = sim.current.springs.l [i];
            sim.previous.springs.k [i] = sim.current.springs.k [i];
            sim.previous.springs.kd[i] = sim.current.springs.kd[i];
        }
    }
    pub fn integrate(&mut self, tick: &GlobalDataUpdatePack) {
        let dt = tick.dt.to_f64_seconds() as f32;
        (self.simulation.current.integrator.0)(&mut self.simulation.current, dt);
        trace!("Integration: {:?}", &self.simulation.current.particles);
    }
    pub fn prepare_render_state_via_lerp_previous_current(&mut self, alpha: f32) {
        let sim = &mut self.simulation;
        for i in 0..sim.render.particles.pos.len() {
            sim.render.particles.pos[i] = Lerp::lerp(sim.previous.particles.pos[i], sim.current.particles.pos[i], alpha);
            sim.render.particles.vel[i] = Lerp::lerp(sim.previous.particles.vel[i], sim.current.particles.vel[i], alpha);
            sim.render.particles.frc[i] = Lerp::lerp(sim.previous.particles.frc[i], sim.current.particles.frc[i], alpha);
            sim.render.particles.m  [i] = Lerp::lerp(sim.previous.particles.m  [i], sim.current.particles.m  [i], alpha);
            self.gfx_particles.vertices[i].position = sim.render.particles.pos[i].into();
        }
        self.gfx_particles.update_vbo();
        trace!("Render state: {:?}", &sim.render.particles);

        for i in 0..sim.render.springs.m1.len() {
            sim.render.springs.m1[i] = Lerp::lerp(sim.previous.springs.m1[i], sim.current.springs.m1[i], alpha);
            sim.render.springs.m2[i] = Lerp::lerp(sim.previous.springs.m2[i], sim.current.springs.m2[i], alpha);
            sim.render.springs.l [i] = Lerp::lerp(sim.previous.springs.l [i], sim.current.springs.l [i], alpha);
            sim.render.springs.k [i] = Lerp::lerp(sim.previous.springs.k [i], sim.current.springs.k [i], alpha);
            sim.render.springs.kd[i] = Lerp::lerp(sim.previous.springs.kd[i], sim.current.springs.kd[i], alpha);
            self.gfx_springs.vertices[2*i + 0].position = sim.render.particles.pos[sim.render.springs.m1[i]].into();
            self.gfx_springs.vertices[2*i + 1].position = sim.render.particles.pos[sim.render.springs.m2[i]].into();
        }
        self.gfx_springs.update_vbo();
    }
}

