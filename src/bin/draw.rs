extern crate spin;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate generic_array;
extern crate typenum;

use spin::color::Rgb;
use spin::color::colors::*;

use std::f32::consts::PI;

// number of radial bins
const NR: usize = 16;

// number of angular bins
const NPHI: usize = 72;

// fraction of the window we use
const SCALE: f32 = 1.0;

// radius of circle
const RAD: f32 = 1.0;

const DR: f32 = RAD / NR as f32;
const DPHI: f32 = 2.0 * PI / NPHI as f32;

fn main() {
    use spin::color::Gradient;
    let rainbow_grad = Gradient::new(arr![Rgb; RED, YELLOW, GREEN, CYAN, BLUE, MAGENTA]);
    let mut rainbow = rainbow_grad.take(1000).cycle();

    let mut draw = Draw::new();

    // Main loop!
    loop {
        draw.draw_color[0] = rainbow.next().unwrap();
        draw.update();
    }
}

fn to_array(c: &Rgb) -> [u8; 3] {
    [c.r, c.g, c.b]
}

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    color: [u8; 3],
}
implement_vertex!(Vertex, position, color);

impl Vertex {
    fn new(x: f32, y: f32, c: Rgb) -> Vertex {
        Vertex {
            position: [x, y],
            color: [c.r, c.g, c.b],
        }
    }
}

struct Draw {
    display: glium::backend::glutin_backend::GlutinFacade,
    program: glium::Program,

    shapes: Vec<Vec<Vertex>>,
    vertex_buffers: Vec<glium::VertexBuffer<Vertex>>,

    to_draw: [bool; 2],
    draw_color: [Rgb; 2],

    mouse_location: (i32, i32),
}

impl Draw {
    fn new() -> Draw {
        use glium::DisplayBuild;
        let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();
        let program = glium::Program::from_source(
            &display, spin::sim::VERTEX_SHADER, spin::sim::FRAGMENT_SHADER, None).unwrap();

        let mut shapes: Vec<Vec<Vertex>> = Vec::with_capacity(NR);


        for r_i in 0..NR {
            let r0 = r_i as f32 * DR;
            let r1 = (r_i + 1) as f32 * DR;
            let mut ring = Vec::with_capacity(2*NPHI);
            for phi_i in 0..NPHI {
                let phi = phi_i as f32 * DPHI;

                ring.push(Vertex::new(r0*phi.cos(), r0*phi.sin(), BLACK));
                ring.push(Vertex::new(r1*phi.cos(), r1*phi.sin(), BLACK));
            }
            ring.push(Vertex::new(r0, 0.0, BLACK));
            ring.push(Vertex::new(r1, 0.0, BLACK));
            shapes.push(ring);
        }

        let mut vertex_buffers: Vec<_> = Vec::with_capacity(NR);
        for shape in &shapes {
            vertex_buffers.push(glium::VertexBuffer::dynamic(&display, &shape).unwrap());
        }

        Draw {
            display: display,
            program: program,

            shapes: shapes,
            vertex_buffers: vertex_buffers,

            to_draw: [false, false],
            draw_color: [WHITE, BLACK],

            mouse_location: (0, 0),
        }
    }

    fn update(&mut self) {
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

        let mut target = self.display.draw();

        use glium::Surface;
        target.clear_color(0.1, 0.1, 0.1, 1.0);


        // fixme: only update if dimensions change
        let perspective = {
            let (width, height) = target.get_dimensions();
            let (xfac, yfac) = if width > height {
                (height as f32 / width as f32, 1.0)
            } else {
                (1.0, width as f32 / height as f32)
            };

            let f = SCALE / RAD;

            [
                [xfac * f, 0.0],
                [0.0, yfac * f],
            ]
        };

        for (buffer, shape) in self.vertex_buffers.iter().zip(self.shapes.iter()) {
            buffer.write(&shape);
            target.draw(buffer, &indices, &self.program, &uniform!{perspective: perspective},
                        &Default::default()).unwrap();
        }
        self.process_input(&target);
        target.finish().unwrap();
    }

    fn process_input(&mut self, target: &glium::Frame) {
        let events: Vec<_> = self.display.poll_events().collect();
        for event in events {
            use glium::glutin::Event;
            use glium::glutin::{ElementState, MouseButton, VirtualKeyCode};
            match event {
                Event::Closed => ::std::process::exit(0),
                Event::MouseMoved(x, y) => {
                    self.mouse_location = (x, y);
                    if self.to_draw.iter().any(|&b| b) {
                        self.paint(&target);
                    }
                },
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::C)) => {
                    // clear the screen
                    for shape in self.shapes.iter_mut() {
                        for vertex in shape.iter_mut() {
                            vertex.color = to_array(&BLACK);
                        }
                    }
                },
                Event::MouseInput(state, button) => {
                    match (state, button) {
                        (ElementState::Pressed, MouseButton::Left) => {
                            self.to_draw[0] = true;
                            self.paint(&target);
                        },
                        (ElementState::Released, MouseButton::Left) => self.to_draw[0] = false,
                        (ElementState::Pressed, MouseButton::Right) => {
                            self.to_draw[1] = true;
                            self.paint(&target);
                        },
                        (ElementState::Released, MouseButton::Right) => self.to_draw[1] = false,
                        _ => (),
                    }
                },
                _ => (),
            }
        }
    }

    fn paint(&mut self, target: &glium::Frame) {
        use glium::Surface;
        let (width, height) = target.get_dimensions();

        let (xfac, yfac) = if width > height {
            (height as f32 / width as f32, 1.0)
        } else {
            (1.0, width as f32 / height as f32)
        };

        let f = SCALE / RAD;

        let x = 2.0 * (self.mouse_location.0 - width as i32/2) as f32 / width as f32 / xfac / f;
        let y = 2.0 * (height as i32/2 - self.mouse_location.1) as f32 / height as f32 / yfac / f;

        if x*x + y*y < RAD*RAD {
            let r = x.hypot(y);
            let mut phi = y.atan2(x);
            while phi < 0.0 {
                phi += 2.0 * PI;
            }

            let r_i = (r/DR) as usize;
            let phi_i = (phi/DPHI).ceil() as usize;

            let color = self.to_draw.iter().zip(self.draw_color.iter()).filter(|&(&td, _)| td).next().unwrap().1;

            self.shapes[r_i as usize][2*phi_i as usize].color = to_array(color);
            self.shapes[r_i as usize][2*phi_i as usize + 1].color = to_array(color);
        }
    }

}
