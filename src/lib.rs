use glam::{Vec3, Quat};
use wasm_bindgen::prelude::*;

#[derive(Clone)]
struct Point {
    pos: Vec3,
    character: u8,
    color: u32,
}

impl Point {
    fn new(x: f32, y: f32, z: f32, chr: u8, color: u32) -> Point {
        Point {
            pos: glam::vec3(x, y, z),
            character: chr,
            color,
        }
    }
}

#[derive(Clone)]
struct FaceConfig {
    default_char: u8,
    color: u32,
    logo: Option<Vec<Vec<u8>>>,
    logo_colors: Option<Vec<Vec<u32>>>,
}

#[wasm_bindgen]
pub struct Cube {
    buffer_chars: Vec<u8>,
    buffer_colors: Vec<u32>,
    height: usize,
    width: usize,
    points: Vec<Point>,
    z_buffer: Vec<f32>,
    distance_from_camera: f32,
    rotation: Quat,
    delta_a: f32,
    delta_b: f32,
    delta_c: f32,
    zoom: f32,
    faces: Vec<FaceConfig>,
}

#[wasm_bindgen]
impl Cube {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Cube {
        let size = width * height;
        let mut faces = vec![];
        let default_chars = [b'@', b'$', b'~', b'#', b';', b'+'];
        for i in 0..6 {
            faces.push(FaceConfig {
                default_char: default_chars[i],
                color: 0x123456, // Sentinel for default theme color
                logo: None,
                logo_colors: None,
            });
        }

        let mut cube = Cube {
            buffer_chars: vec![b' '; size],
            buffer_colors: vec![0; size],
            height,
            width,
            points: vec![Point::new(0.0, 0.0, 0.0, b' ', 0); 60000], // 100x100x6
            z_buffer: vec![f32::NEG_INFINITY; size],
            distance_from_camera: 100.0,
            rotation: Quat::IDENTITY,
            delta_a: 0.0075,
            delta_b: 0.005,
            delta_c: 0.01,
            zoom: 1000.0,
            faces,
        };

        for i in 0..6 {
            cube.generate_points_for_face(i);
        }
        cube
    }

    #[wasm_bindgen]
    pub fn set_rotation(&mut self, a: f32, b: f32, c: f32) {
        self.rotation = Quat::from_euler(glam::EulerRot::XYZ, a, b, c);
    }

    #[wasm_bindgen]
    pub fn set_rotation_speed(&mut self, da: f32, db: f32, dc: f32) {
        self.delta_a = da;
        self.delta_b = db;
        self.delta_c = dc;
    }

    #[wasm_bindgen]
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    #[wasm_bindgen]
    pub fn set_face_color(&mut self, face_index: usize, hex_color: &str) {
        if face_index < 6 {
            let hex = hex_color.trim_start_matches('#');
            if let Ok(color) = u32::from_str_radix(hex, 16) {
                self.faces[face_index].color = color;
                self.generate_points_for_face(face_index);
            }
        }
    }

    #[wasm_bindgen]
    pub fn set_face_logo(&mut self, face_index: usize, logo: &str) {
        if face_index < 6 {
            let mut logo_grid = Vec::new();
            for line in logo.lines() {
                logo_grid.push(line.chars().map(|c| {
                    if c == '█' { 255 }
                    else if c == '▒' { 254 }
                    else if c.is_ascii() { c as u8 }
                    else { b' ' }
                }).collect::<Vec<u8>>());
            }
            self.faces[face_index].logo = Some(logo_grid);
            self.faces[face_index].logo_colors = None;
            self.generate_points_for_face(face_index);
        }
    }

    #[wasm_bindgen]
    pub fn set_face_colored_logo(&mut self, face_index: usize, logo_chars: &str, logo_colors: &[u32]) {
        if face_index < 6 {
            let mut logo_grid = Vec::new();
            let mut color_grid = Vec::new();
            let mut color_index = 0;
            
            for line in logo_chars.lines() {
                let mut char_row = Vec::new();
                let mut color_row = Vec::new();
                for c in line.chars() {
                    let u = if c == '█' { 255 } else if c == '▒' { 254 } else if c.is_ascii() { c as u8 } else { b' ' };
                    char_row.push(u);
                    if color_index < logo_colors.len() {
                        color_row.push(logo_colors[color_index]);
                    } else {
                        color_row.push(self.faces[face_index].color);
                    }
                    color_index += 1;
                }
                logo_grid.push(char_row);
                color_grid.push(color_row);
            }
            
            self.faces[face_index].logo = Some(logo_grid);
            self.faces[face_index].logo_colors = Some(color_grid);
            self.generate_points_for_face(face_index);
        }
    }

    #[wasm_bindgen]
    pub fn update_face_fast(&mut self, face_index: usize, logo_chars: &[u8], logo_colors: &[u32]) {
        if face_index >= 6 || self.points.len() < 60000 {
            return;
        }
        
        let start_idx = face_index * 10000;
        let default_char = self.faces[face_index].default_char;
        let default_color = self.faces[face_index].color;
        
        // Must match generate_points_for_face exactly:
        //   outer loop i → logo_x (image column)
        //   inner loop j → logo_y (image row)
        //   point_idx = start + i * 100 + j
        let w = 100_usize;
        let h = 100_usize;
        for i in 0..w {          // i = image column (x)
            for j in 0..h {      // j = image row (y)
                let flat_idx = j * w + i; // row-major pixel at (col=i, row=j)
                let c = if flat_idx < logo_chars.len() { logo_chars[flat_idx] } else { b' ' };
                let color = if flat_idx < logo_colors.len() { logo_colors[flat_idx] } else { default_color };

                let point_idx = start_idx + i * w + j; // matches generate: i * 100 + j
                let pt = &mut self.points[point_idx];
                if c != b' ' {
                    pt.character = c;
                    pt.color = color;
                } else {
                    pt.character = default_char;
                    pt.color = default_color;
                }
            }
        }
    }

    fn generate_points_for_face(&mut self, face_index: usize) {
        let dots_per_face: u64 = 100;
        let step = 2.0 / (dots_per_face as f32);

        let axes = [
            (1, 1.0),  // 0: Top
            (1, -1.0), // 1: Bottom
            (0, -1.0), // 2: Left
            (0, 1.0),  // 3: Right
            (2, 1.0),  // 4: Front
            (2, -1.0), // 5: Back
        ];

        let (fixed_axis, fixed_val) = axes[face_index];
        let face = &self.faces[face_index];
        let logo_height = face.logo.as_ref().map_or(0, |l| l.len());
        let logo_width = face.logo.as_ref().and_then(|l| l.first()).map_or(0, |r| r.len());
        
        let start_idx = face_index * 10000;

        for i in 0..dots_per_face {
            for j in 0..dots_per_face {
                let u = -1.0 + (i as f32) * step;
                let v = -1.0 + (j as f32) * step;

                let mut character = face.default_char;
                let mut current_color = face.color;

                if logo_height > 0 && logo_width > 0 {
                    let logo_x = (i * logo_width as u64) / dots_per_face;
                    let logo_y = (j * logo_height as u64) / dots_per_face;

                    if let Some(row) = face.logo.as_ref().unwrap().get(logo_y as usize) {
                        if let Some(&c) = row.get(logo_x as usize) {
                            if c != b' ' { // treat spaces as transparent
                                character = c;
                                if let Some(colors) = &face.logo_colors {
                                    if let Some(color_row) = colors.get(logo_y as usize) {
                                        if let Some(&color) = color_row.get(logo_x as usize) {
                                            current_color = color;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let (x, y, z) = match fixed_axis {
                    0 => (fixed_val, u, v),
                    1 => (u, fixed_val, v),
                    _ => (u, v, fixed_val),
                };
                
                let pt = Point::new(x, y, z, character, current_color);
                
                let point_idx = start_idx + (i * dots_per_face + j) as usize;
                if point_idx < self.points.len() {
                    self.points[point_idx] = pt;
                } else {
                    self.points.push(pt);
                }
            }
        }
    }

    #[wasm_bindgen]
    pub fn is_face_visible(&self, face_index: usize) -> bool {
        let (nx, ny, nz) = match face_index {
            0 => (0.0, 1.0, 0.0),  // Top
            1 => (0.0, -1.0, 0.0), // Bottom
            2 => (-1.0, 0.0, 0.0), // Left
            3 => (1.0, 0.0, 0.0),  // Right
            4 => (0.0, 0.0, 1.0),  // Front
            5 => (0.0, 0.0, -1.0), // Back
            _ => return false,
        };

        let rot_normal = self.rotation * glam::vec3(nx, ny, nz);
        rot_normal.z < 0.0
    }

    #[wasm_bindgen]
    pub fn next_frame(&mut self) {
        let rot_x = Quat::from_rotation_x(self.delta_a);
        let rot_y = Quat::from_rotation_y(self.delta_b);
        let rot_z = Quat::from_rotation_z(self.delta_c);

        self.rotation = rot_x * rot_y * rot_z * self.rotation;
        self.rotation = self.rotation.normalize();

        self.buffer_chars.fill(b' ');
        self.buffer_colors.fill(0);
        self.z_buffer.fill(f32::NEG_INFINITY);

        for face_idx in 0..6 {
            if !self.is_face_visible(face_idx) {
                continue;
            }
            
            let start = face_idx * 10000;
            let end = start + 10000;

            for p in &self.points[start..end] {
                let rot = self.rotation * p.pos;
                let ooz = 1.0 / (rot.z + self.distance_from_camera);

                let xp = (self.width as f32 / 2.0 + self.zoom * ooz * rot.x * 2.0) as i32;
                let yp = (self.height as f32 / 2.0 + self.zoom * ooz * rot.y) as i32;

                if xp >= 0 && xp < self.width as i32 && yp >= 0 && yp < self.height as i32 {
                    let idx = (xp + yp * self.width as i32) as usize;

                    if idx < self.z_buffer.len() {
                        if ooz > self.z_buffer[idx] {
                            self.z_buffer[idx] = ooz;
                            self.buffer_chars[idx] = p.character;
                            self.buffer_colors[idx] = p.color;
                        }
                    }
                }
            }
        }
    }

    #[wasm_bindgen]
    pub fn chars_ptr(&self) -> *const u8 {
        self.buffer_chars.as_ptr()
    }

    #[wasm_bindgen]
    pub fn colors_ptr(&self) -> *const u32 {
        self.buffer_colors.as_ptr()
    }
}
