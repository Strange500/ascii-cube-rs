use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

use glam::Vec3;
use terminal_size::terminal_size;

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

fn main() {
    let mut engine = Engine::new(80, 24);

    engine.init_cube_points();

    print!("\x1B[2J");

    loop {
        if let Some((w, h)) = terminal_size() {
            let width = w.0 as u64;
            let height = h.0 as u64;

            if width != engine.width || height != engine.height {
                engine.resize(width, height);
                print!("\x1B[2J");
            }
        }

        print!("\x1B[H");

        print!("{}", engine.gen_frame());
        io::stdout().flush().unwrap();

        thread::sleep(Duration::from_millis(16));
    }
}

struct Engine {
    buffer_display: Vec<char>,
    height: u64,
    width: u64,
    points: Vec<Point>,
    A: f32,
    B: f32,
    C: f32,
}

impl Engine {
    fn new(width: u64, height: u64) -> Engine {
        Engine {
            buffer_display: vec![' '; (width * height) as usize],
            height,
            width,
            points: Vec::new(),
            A: 0.0,
            B: 0.0,
            C: 0.0,
        }
    }

    fn resize(&mut self, width: u64, height: u64) {
        self.width = width;
        self.height = height;
        self.buffer_display = vec![' '; (width * height) as usize];
    }

    fn add_point(&mut self, p: Point) {
        self.points.push(p);
    }

    fn init_cube_points(&mut self) {
        let dots_per_face: u64 = 100;
        let step = 2.0 / (dots_per_face as f32);
        let char_list = ['@', '#', '-'];

        let mut add_face = |fixed_axis: usize, fixed_val: f32| {
            for i in 0..dots_per_face {
                let chr = char_list[((i + fixed_axis as u64) % char_list.len() as u64) as usize];
                for j in 0..dots_per_face {
                    let u = -1.0 + (i as f32) * step;
                    let v = -1.0 + (j as f32) * step;
                    let (x, y, z) = match fixed_axis {
                        0 => (fixed_val, u, v),
                        1 => (u, fixed_val, v),
                        _ => (u, v, fixed_val),
                    };
                    self.add_point(Point::new(x, y, z, chr));
                }
            }
        };

        add_face(1, 1.0); // Haut
        add_face(1, -1.0); // Bas
        add_face(0, -1.0); // Gauche
        add_face(0, 1.0); // Droite
        add_face(2, 1.0); // Devant
        add_face(2, -1.0); // Derrière
    }

    fn gen_frame(&mut self) -> String {
        self.buffer_display.fill(' ');

        let min_dim = std::cmp::min(self.width, self.height) as f32;
        let scale_base = min_dim / 3.5;

        let scale_x = scale_base * 2.0;
        let scale_y = scale_base;

        let offset_x = self.width as f32 / 2.0;
        let offset_y = self.height as f32 / 2.0;

        for p in self.points.iter() {
            let rot = get_cube_rotation_matrice(self.A, self.B, self.C, p.pos.x, p.pos.y, p.pos.z);

            let x = (rot.x * scale_x + offset_x) as i64;
            let y = (rot.y * scale_y + offset_y) as i64;

            if x >= 0 && x < (self.width as i64) && y >= 0 && y < (self.height as i64) {
                self.buffer_display[(x as u64 + (y as u64) * self.width) as usize] = p.character;
            }
        }

        self.C += 0.04;
        self.A += 0.02;

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

fn get_cube_rotation_matrice(A: f32, B: f32, C: f32, i: f32, j: f32, k: f32) -> Vec3 {
    let (sin_a, cos_a) = A.sin_cos();
    let (sin_b, cos_b) = B.sin_cos();
    let (sin_c, cos_c) = C.sin_cos();

    glam::vec3(
        j * sin_a * cos_c - k * cos_a * sin_b * cos_c
            + j * cos_a * sin_c
            + k * sin_a * sin_c
            + i * cos_b * cos_c,
        j * cos_a * cos_c + k * sin_a * cos_c - j * sin_a * sin_b * sin_c
            + k * cos_a * sin_b * sin_c
            - i * cos_b * sin_c,
        k * cos_a * sin_b - j * sin_a * cos_b + i * sin_b,
    )
}
