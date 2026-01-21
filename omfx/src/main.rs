//! OMFX - OMOBA Fyrox Debug Frontend
//!
//! A 2D top-down visualization tool for debugging MOBA game state.

use fyrox::engine::executor::Executor;
use log::info;

mod game;
mod renderer;

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting OMFX - OMOBA Fyrox Debug Frontend");

    // Create and run the game
    let mut executor = Executor::new();
    executor.add_plugin(game::Game::new());
    executor.run()
}
