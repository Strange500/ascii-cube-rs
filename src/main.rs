use cube::{Engine, get_bitcoin_price};
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::{
    io::{self, Write},
    thread,
};
#[cfg(not(target_arch = "wasm32"))]
use terminal_size::terminal_size;

#[cfg(not(target_arch = "wasm32"))]
fn calculate_speed_from_price(price: f32) -> f32 {
    let target_price = 100_000.0;
    let min_price = 0.0;
    let min_speed = 0.5;
    let max_speed = 50.0;

    let factor = (price - min_price) / (target_price - min_price);
    let factor = factor.clamp(0.0, 1.0);
    min_speed + (factor * factor * (max_speed - min_speed))
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let mut engine = Engine::new(80, 24);

    println!("Fetching Bitcoin price...");

    // We await the native implementation of get_bitcoin_price
    let price = match get_bitcoin_price().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error fetching Bitcoin price: {}", e);
            30_000.0 // Fallback price
        }
    };

    println!("Price: ${}", price);
    thread::sleep(Duration::from_secs(1));

    let speed = calculate_speed_from_price(price);

    engine.init_pyramid_points();

    print!("\x1B[2J");

    let engine = std::sync::Arc::new(std::sync::Mutex::new(engine));
    let engine_render = engine.clone();
    let engine_rotate = engine.clone();
    
    // Shared speed state that can be updated by the price fetcher thread
    let speed_shared = std::sync::Arc::new(std::sync::Mutex::new(speed));
    let speed_for_rotate = speed_shared.clone();
    let speed_for_updater = speed_shared.clone();

    // Price Update Loop - Periodically fetch Bitcoin price and update speed
    let _price_updater_thread = thread::spawn(move || {
        // Create runtime once outside the loop
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        loop {
            // Sleep for 60 seconds before fetching the price again
            thread::sleep(Duration::from_secs(60));
            
            // Fetch the new price
            let new_price = runtime.block_on(async {
                match get_bitcoin_price().await {
                    Ok(p) => Some(p),
                    Err(e) => {
                        eprintln!("Error fetching Bitcoin price: {}", e);
                        None
                    }
                }
            });
            
            if let Some(price) = new_price {
                // Calculate new speed based on the updated price
                let new_speed = calculate_speed_from_price(price);
                
                // Update the shared speed
                let mut speed = speed_for_updater.lock().unwrap();
                *speed = new_speed;
            }
        }
    });

    // Render Loop
    let render_thread = thread::spawn(move || {
        loop {
            if let Some((w, h)) = terminal_size() {
                let width = w.0 as usize;
                let height = h.0 as usize;

                let mut eng = engine_render.lock().unwrap();
                if width != eng.width || height != eng.height {
                    eng.resize(width, height);
                    print!("\x1B[2J");
                }

                print!("\x1B[H");
                print!("{}", eng.render_frame());
            }
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(16));
        }
    });

    // Rotation Loop
    let rotate_thread = thread::spawn(move || {
        loop {
            {
                let mut eng = engine_rotate.lock().unwrap();
                let current_speed = *speed_for_rotate.lock().unwrap();
                eng.rotate(current_speed);
            }
            thread::sleep(Duration::from_millis(16));
        }
    });

    render_thread.join().unwrap();
    rotate_thread.join().unwrap();
}

// Dummy main for Wasm to satisfy the compiler if building as a binary
#[cfg(target_arch = "wasm32")]
fn main() {}
