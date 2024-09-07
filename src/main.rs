pub mod tilemap;

use log::{error, info};
use raylib::prelude::*;

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 608; // 600 does not divide by 8

fn main() {
    // Init the pretty_env_logger
    unsafe { std::env::set_var("RUST_APP_LOG", "info") };
    pretty_env_logger::init_custom_env("RUST_APP_LOG");

    // Initialize raylib staff
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Raylib Dual Grid")
        .build();
    rl.set_target_fps(60);
    info!("Raylib initialized");

    // Load the water.png texture
    let water: Texture2D = match rl.load_texture(&thread, "resources/water.png") {
        Ok(texture) => texture,
        Err(e) => {
            error!("Failed to load water.png texture: {:?}", e);
            std::process::exit(1);
        }
    };
    info!("Texture \"water.png\" loaded");

    // Load the tilemap
    let mut tilemap =
        tilemap::TileMap::new(&mut rl, &thread, "tile_rules.yaml", "resources/grass.png");

    tilemap.add_chunk(0, 0, f32::ceil(SCREEN_WIDTH as f32 / 8.0 / 4.0) as i32, f32::ceil(SCREEN_HEIGHT as f32 / 8.0 / 4.0) as i32);

    // Enter the game loop
    while !rl.window_should_close() {
        let mouse_pos = &rl.get_mouse_position();

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        // Draw the water texture as the background
        for i in 0..SCREEN_WIDTH / &water.width() {
            for j in 0..SCREEN_HEIGHT / &water.height() {
                d.draw_texture(
                    &water,
                    i * &water.width(),
                    j * &water.height(),
                    Color::WHITE,
                );
            }
        }

        // Draw the tilemap
        tilemap.draw(&mut d);

        // Draw a squeare at the mouse position
        d.draw_rectangle(
            f32::floor(mouse_pos.x / 8.0 / 4.0) as i32 * 8 * 4,
            f32::floor(mouse_pos.y / 8.0 / 4.0) as i32 * 8 * 4,
            8 * 4,
            8 * 4,
            Color::new(255, 0, 0, 128),
        );
        
        // If the mouse is pressed, add a tile to the tilemap
        if d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            tilemap.set(
                f32::floor(mouse_pos.x / 8.0 / 4.0) as i32,
                f32::floor(mouse_pos.y / 8.0 / 4.0) as i32,
                true,
            );
        } else if d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            tilemap.set(
                f32::floor(mouse_pos.x / 8.0 / 4.0) as i32,
                f32::floor(mouse_pos.y / 8.0 / 4.0) as i32,
                false,
            );
        }
    }
}
