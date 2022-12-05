extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::OpenGL;
use piston::EventLoop;
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
mod app;
use app::{App, SoldierType::{*, self}, Team::{*, self}, SCREEN_HEIGHT, SCREEN_WIDTH};

fn add_block(app: &mut App, pos: [f64; 2], x: u32, y: u32, team: Team, legion: SoldierType) {
    for y in (0..y).map(|f| f as f64 * 8.0) {
        for x in (0..x).map(|f| f as f64 * 8.0) {
            app.add_soldier(team, legion, [x + pos[0], y + pos[1]])
        }
    }
}



fn main() {

    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("Roman Battle Simulation", [SCREEN_WIDTH, SCREEN_HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .decorated(true)
        .resizable(false)
        .build()
        .unwrap();

    let mut app = App::new(opengl);
    
    
    add_block(&mut app, [100.0, 100.0], 6, 10, Red, Triarii);

    

    let mut settings = EventSettings::new();
    settings.set_max_fps(60);
    settings.set_ups(60);


    let mut events = Events::new(settings);
    while let Some(event) = events.next(&mut window) {

        if let Some(args) = event.render_args() {
            app.render(&args);
        }
        if let Some(_) = event.update_args() {
            app.update();
        }
    }
}
