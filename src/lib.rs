use wasm_bindgen::prelude::*;

use glam::Vec3;

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
struct Engine {
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
impl Engine {
    pub fn new(width: usize, height: usize) -> Engine {
        let size = width * height;
        let mut engine = Engine {
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

        engine.init_cube_points();
        engine
    }

    fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        let size = (width * height) as usize;
        self.buffer_display = vec![' '; size];
        self.z_buffer = vec![f32::NEG_INFINITY; size];
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

    fn init_pyramid_points(&mut self) {
        let dots_per_face: u64 = 100;

        let step = 1.0 / (dots_per_face as f32);

        let apex = glam::vec3(0.0, 1.0, 0.0);

        let mut add_face = |base_p1: Vec3, base_p2: Vec3, character: char| {
            for i in 0..dots_per_face {
                for j in 0..dots_per_face {
                    let u = (i as f32) * step;
                    let v = (j as f32) * step;

                    let point_on_base = base_p1 + (base_p2 - base_p1) * u;

                    let point = apex + (point_on_base - apex) * v;

                    self.add_point(Point::new(point.x, point.y, point.z, character));
                }
            }
        };

        add_face(
            glam::vec3(-1.0, -1.0, -1.0),
            glam::vec3(1.0, -1.0, -1.0),
            '^',
        );
        add_face(glam::vec3(1.0, -1.0, -1.0), glam::vec3(1.0, -1.0, 1.0), '%'); // Right
        add_face(glam::vec3(1.0, -1.0, 1.0), glam::vec3(-1.0, -1.0, 1.0), '&'); // Back
        add_face(
            glam::vec3(-1.0, -1.0, 1.0),
            glam::vec3(-1.0, -1.0, -1.0),
            '*',
        );

        let mut add_base = |p1: Vec3, p2: Vec3, p3: Vec3, p4: Vec3, character: char| {
            for i in 0..dots_per_face {
                for j in 0..dots_per_face {
                    let u = (i as f32) * step;
                    let v = (j as f32) * step;
                    let start = p1 + (p2 - p1) * u;
                    let end = p4 + (p3 - p4) * u;
                    let point = start + (end - start) * v;
                    self.add_point(Point::new(point.x, point.y, point.z, character));
                }
            }
        };

        add_base(
            glam::vec3(-1.0, -1.0, -1.0),
            glam::vec3(1.0, -1.0, -1.0),
            glam::vec3(1.0, -1.0, 1.0),
            glam::vec3(-1.0, -1.0, 1.0),
            '.',
        );
    }

    pub fn render_frame(&mut self) -> String {
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

        self.c += 0.01;
        self.a += 0.01;
        self.b += 0.01;

        return self.render_buffer_to_string();
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

    // Rotation matrix for ZYX Euler angles (Rz(c) * Ry(b) * Rx(a))
    // Applied to point (i, j, k)
    // x' = i*(cos_b*cos_c) + j*(sin_a*sin_b*cos_c - cos_a*sin_c) + k*(cos_a*sin_b*cos_c + sin_a*sin_c)
    // y' = i*(cos_b*sin_c) + j*(sin_a*sin_b*sin_c + cos_a*cos_c) + k*(cos_a*sin_b*sin_c - sin_a*cos_c)
    // z' = i*(-sin_b) + j*(sin_a*cos_b) + k*(cos_a*cos_b)
    glam::vec3(
        i * cos_b * cos_c + j * (sin_a * sin_b * cos_c - cos_a * sin_c)
            + k * (cos_a * sin_b * cos_c + sin_a * sin_c),
        i * cos_b * sin_c + j * (sin_a * sin_b * sin_c + cos_a * cos_c)
            + k * (cos_a * sin_b * sin_c - sin_a * cos_c),
        -i * sin_b + j * sin_a * cos_b + k * cos_a * cos_b,
    )
}
