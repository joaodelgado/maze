#![feature(option_filter)]
#![feature(drain_filter)]

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

mod config;
mod errors;
mod generator;
mod maze;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;

use config::*;
use errors::Result;
use generator::{Generator, Kruskal};
use maze::Maze;

struct App {
    maze: Maze,
    generator: Box<Generator>,
}

impl App {
    fn recursive_backtracker() -> App {
        let maze = Maze::new();
        App {
            generator: Box::new(Kruskal::new(&maze)),
            maze: maze,
        }
    }

    fn render(&self, args: &RenderArgs, gl: &mut GlGraphics) {
        self.maze.render(args, gl);
    }

    fn tick(&mut self) -> Result<()> {
        self.generator.tick(&mut self.maze)?;
        Ok(())
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window =
        WindowSettings::new("Space filling circles", [WINDOW_WIDTH, WINDOW_HEIGHT])
            .opengl(opengl)
            .exit_on_esc(false)
            .build()
            .expect("Error creating window");

    let mut gl = GlGraphics::new(opengl);
    let mut app = App::recursive_backtracker();

    let mut updating = true;
    let mut event_settings = EventSettings::new();
    event_settings.ups = UPS;

    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if !updating {
            return;
        }

        if let Some(r) = e.render_args() {
            app.render(&r, &mut gl);
        }

        if let Some(_) = e.update_args() {
            match app.tick() {
                Ok(()) => {}
                Err(e) => {
                    updating = false;
                    eprintln!("[ERROR] {}", e);
                }
            }
        }
    }
}
