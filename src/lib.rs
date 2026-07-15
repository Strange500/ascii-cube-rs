use glam::{Vec3, Quat};
use wasm_bindgen::prelude::*;

#[derive(Clone)]
struct Point {
    pos: Vec3,
    character: char,
    color: u32,
}

impl Point {
    fn new(x: f32, y: f32, z: f32, chr: char, color: u32) -> Point {
        Point {
            pos: glam::vec3(x, y, z),
            character: chr,
            color,
        }
    }
}

#[derive(Clone)]
struct FaceConfig {
    default_char: char,
    color: u32,
    logo: Option<Vec<Vec<char>>>,
}

#[wasm_bindgen]
pub struct Cube {
    buffer_chars: Vec<char>,
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
        let default_chars = ['@', '$', '~', '#', ';', '+'];
        for i in 0..6 {
            faces.push(FaceConfig {
                default_char: default_chars[i],
                color: 0xffffff, // Default color white
                logo: None,
            });
        }

        let mut cube = Cube {
            buffer_chars: vec![' '; size],
            buffer_colors: vec![0; size],
            height,
            width,
            points: Vec::new(),
            z_buffer: vec![f32::NEG_INFINITY; size],
            distance_from_camera: 100.0,
            rotation: Quat::IDENTITY,
            delta_a: 0.0075,
            delta_b: 0.005,
            delta_c: 0.01,
            zoom: 1000.0,
            faces,
        };

        cube.generate_points();
        cube
    }

    /// Override the current rotation angles instantly
    #[wasm_bindgen]
    pub fn set_rotation(&mut self, a: f32, b: f32, c: f32) {
        self.rotation = Quat::from_euler(glam::EulerRot::XYZ, a, b, c);
    }

    /// Override the rotation speed (how much it turns per frame)
    #[wasm_bindgen]
    pub fn set_rotation_speed(&mut self, da: f32, db: f32, dc: f32) {
        self.delta_a = da;
        self.delta_b = db;
        self.delta_c = dc;
    }

    /// Sets the zoom level (projection scale factor, default 1000.0)
    #[wasm_bindgen]
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    /// Sets the color of a specific face (0 to 5) using a hex color like "#ff0000" or "ff0000"
    #[wasm_bindgen]
    pub fn set_face_color(&mut self, face_index: usize, hex_color: &str) {
        if face_index < 6 {
            let hex = hex_color.trim_start_matches('#');
            if let Ok(color) = u32::from_str_radix(hex, 16) {
                self.faces[face_index].color = color;
                self.generate_points();
            }
        }
    }

    /// Sets a multiline string (ASCII art) to be mapped onto a specific face.
    #[wasm_bindgen]
    pub fn set_face_logo(&mut self, face_index: usize, logo: &str) {
        if face_index < 6 {
            let mut logo_grid = Vec::new();
            for line in logo.lines() {
                logo_grid.push(line.chars().collect::<Vec<char>>());
            }
            self.faces[face_index].logo = Some(logo_grid);
            self.generate_points();
        }
    }

    fn generate_points(&mut self) {
        self.points.clear();
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

        for (idx, &(fixed_axis, fixed_val)) in axes.iter().enumerate() {
            let face = &self.faces[idx];
            let logo_height = face.logo.as_ref().map_or(0, |l| l.len());
            let logo_width = face.logo.as_ref().and_then(|l| l.first()).map_or(0, |r| r.len());

            for i in 0..dots_per_face {
                for j in 0..dots_per_face {
                    let u = -1.0 + (i as f32) * step;
                    let v = -1.0 + (j as f32) * step;

                    let mut character = face.default_char;

                    if logo_height > 0 && logo_width > 0 {
                        // Map grid to logo coordinates (i is horizontal, j is vertical)
                        let logo_x = (i * logo_width as u64) / dots_per_face;
                        let logo_y = (j * logo_height as u64) / dots_per_face;

                        if let Some(row) = face.logo.as_ref().unwrap().get(logo_y as usize) {
                            if let Some(&c) = row.get(logo_x as usize) {
                                if c != ' ' { // treat spaces as transparent
                                    character = c;
                                }
                            }
                        }
                    }

                    let (x, y, z) = match fixed_axis {
                        0 => (fixed_val, u, v),
                        1 => (u, fixed_val, v),
                        _ => (u, v, fixed_val),
                    };
                    self.points.push(Point::new(x, y, z, character, face.color));
                }
            }
        }
    }

    #[wasm_bindgen]
    pub fn next_frame(&mut self) -> String {
        // Create global rotation deltas for X, Y, Z axes
        let rot_x = Quat::from_rotation_x(self.delta_a);
        let rot_y = Quat::from_rotation_y(self.delta_b);
        let rot_z = Quat::from_rotation_z(self.delta_c);

        // Left-multiply to apply rotations in the global (camera) coordinate system
        self.rotation = rot_x * rot_y * rot_z * self.rotation;
        self.rotation = self.rotation.normalize();

        self.buffer_chars.fill(' ');
        self.buffer_colors.fill(0);
        self.z_buffer.fill(f32::NEG_INFINITY);

        for p in self.points.iter() {
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

        self.render_buffer_to_string()
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
        
        // Since the camera is conceptually located at z = -distance_from_camera 
        // looking towards the +z axis, faces with a negative rotated Z normal 
        // are pointing towards the camera.
        rot_normal.z < 0.0
    }

    fn render_buffer_to_string(&self) -> String {
        let mut output = String::with_capacity((self.width * self.height * 20) as usize);
        let mut current_color: Option<u32> = None;

        for row in 0..self.height {
            let start = (row * self.width) as usize;
            let end = start + self.width as usize;

            for idx in start..end {
                let c = self.buffer_chars[idx];
                let color = self.buffer_colors[idx];

                if c == ' ' {
                    if current_color.is_some() {
                        output.push_str("</span>");
                        current_color = None;
                    }
                    output.push(' ');
                } else {
                    if current_color != Some(color) {
                        if current_color.is_some() {
                            output.push_str("</span>");
                        }
                        // Format the color to exactly 6 hex characters (e.g., #00ff00)
                        output.push_str(&format!("<span style=\"color:#{:06x}\">", color));
                        current_color = Some(color);
                    }
                    // HTML escape for safety since it goes straight to innerHTML
                    match c {
                        '<' => output.push_str("&lt;"),
                        '>' => output.push_str("&gt;"),
                        '&' => output.push_str("&amp;"),
                        _ => output.push(c),
                    }
                }
            }
            if current_color.is_some() {
                output.push_str("</span>");
                current_color = None;
            }
            output.push('\n');
        }
        output
    }
}


