mod lib;
use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

use cube::Engine;
use terminal_size::terminal_size;

fn main() {
    let mut engine = Engine::new(80, 24);

    engine.init_pyramid_points();

    print!("\x1B[2J");

    loop {
        if let Some((w, h)) = terminal_size() {
            let width = w.0 as usize;
            let height = h.0 as usize;

            if width != engine.width || height != engine.height {
                engine.resize(width, height);
                print!("\x1B[2J");
            }
        }

        print!("\x1B[H");

        print!("{}", engine.render_frame());
        io::stdout().flush().unwrap();
        engine.rotate();
        thread::sleep(Duration::from_millis(16));
    }
}
