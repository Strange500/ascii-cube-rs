use core::f32;
use glam::Vec3;
use wasm_bindgen::prelude::*;

struct Point {
    pos: Vec3,
    character: char,
}

impl Point {
    fn new(x: f32, y: f32, z: f32, chr: char) -> Point {
        Point {
            pos: glam::vec3(x, y, z),
            character: chr,
        }
    }
}

#[wasm_bindgen]
pub struct Cube {
    buffer_display: Vec<char>,
    height: usize,
    width: usize,
    points: Vec<Point>,
    z_buffer: Vec<f32>,
    distance_from_camera: f32,
    a: f32,
    b: f32,
    c: f32,
}

#[wasm_bindgen]
impl Cube {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Cube {
        let size = width * height;
        let mut cube = Cube {
            buffer_display: vec![' '; size],
            height,
            width,
            points: Vec::new(),
            z_buffer: vec![f32::NEG_INFINITY; size],
            distance_from_camera: 100.0,
            a: 0.0,
            b: 0.0,
            c: 0.0,
        };

        cube.init_cube_points();
        cube
    }

    fn add_point(&mut self, p: Point) {
        self.points.push(p);
    }

    fn init_cube_points(&mut self) {
        let dots_per_face: u64 = 100;
        let step = 2.0 / (dots_per_face as f32);

        let mut add_face = |fixed_axis: usize, fixed_val: f32, character: char| {
            for i in 0..dots_per_face {
                for j in 0..dots_per_face {
                    let u = -1.0 + (i as f32) * step;
                    let v = -1.0 + (j as f32) * step;
                    let (x, y, z) = match fixed_axis {
                        0 => (fixed_val, u, v),
                        1 => (u, fixed_val, v),
                        _ => (u, v, fixed_val),
                    };
                    self.add_point(Point::new(x, y, z, character));
                }
            }
        };

        add_face(1, 1.0, '@'); // Haut
        add_face(1, -1.0, '$'); // Bas
        add_face(0, -1.0, '~'); // Gauche
        add_face(0, 1.0, '#'); // Droite
        add_face(2, 1.0, ';'); // Devant
        add_face(2, -1.0, '+'); // Derrière
    }

    #[wasm_bindgen]
    pub fn next_frame(&mut self) -> String {
        // Increment angles for rotation
        let speed = 1.0;
        self.c += speed * 0.01;
        self.a += speed * 0.0075;
        self.b += speed * 0.005;

        self.buffer_display.fill(' ');
        self.z_buffer.fill(f32::NEG_INFINITY);

        let k1 = 1000.0;

        for p in self.points.iter() {
            let rot = get_cube_rotation_matrice(self.a, self.b, self.c, p.pos.x, p.pos.y, p.pos.z);

            let ooz = 1.0 / (rot.z + self.distance_from_camera);

            let xp = (self.width as f32 / 2.0 + k1 * ooz * rot.x * 2.0) as i32;
            let yp = (self.height as f32 / 2.0 + k1 * ooz * rot.y) as i32;

            if xp >= 0 && xp < self.width as i32 && yp >= 0 && yp < self.height as i32 {
                let idx = (xp + yp * self.width as i32) as usize;

                if idx < self.z_buffer.len() {
                    if ooz > self.z_buffer[idx] {
                        self.z_buffer[idx] = ooz;
                        self.buffer_display[idx] = p.character;
                    }
                }
            }
        }

        self.render_buffer_to_string()
    }

    fn render_buffer_to_string(&self) -> String {
        let mut output = String::with_capacity((self.width * self.height + self.height) as usize);
        for row in 0..self.height {
            let start = (row * self.width) as usize;
            let end = start + self.width as usize;
            output.extend(&self.buffer_display[start..end]);
            output.push('\n');
        }
        output
    }
}

fn get_cube_rotation_matrice(a: f32, b: f32, c: f32, i: f32, j: f32, k: f32) -> Vec3 {
    let (sin_a, cos_a) = a.sin_cos();
    let (sin_b, cos_b) = b.sin_cos();
    let (sin_c, cos_c) = c.sin_cos();

    glam::vec3(
        i * cos_b * cos_c
            + j * (sin_a * sin_b * cos_c - cos_a * sin_c)
            + k * (cos_a * sin_b * cos_c + sin_a * sin_c),
        i * cos_b * sin_c
            + j * (sin_a * sin_b * sin_c + cos_a * cos_c)
            + k * (cos_a * sin_b * sin_c - sin_a * cos_c),
        -i * sin_b + j * sin_a * cos_b + k * cos_a * cos_b,
    )
}
