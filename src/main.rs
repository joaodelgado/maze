#![feature(option_filter)]
#![feature(drain_filter)]
#![feature(vec_remove_item)]

#[macro_use]
extern crate structopt;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

mod config;
mod error;
mod generator;
mod maze;
mod solver;

use structopt::StructOpt;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;

use config::Config;
use error::Result;
use generator::Generator;
use maze::Maze;
use solver::Solver;

enum AppMode {
    Generating,
    Solving,
}

struct App<'a> {
    maze: Maze<'a>,
    mode: AppMode,
    generator: Box<Generator>,
    solver: Box<Solver>,
    config: &'a Config,
}

impl<'a> App<'a> {
    fn new(config: &'a Config) -> Result<App<'a>> {
        let maze = Maze::new(&config);
        let generator = config.generator().init(&maze);
        let solver = config.solver().init(&maze);

        Ok(App {
            maze,
            mode: AppMode::Generating,
            generator,
            solver,
            config,
        })
    }

    fn render(&self, args: &RenderArgs, gl: &mut GlGraphics) {
        self.maze.render(args, gl);
    }

    fn tick(&mut self) -> Result<()> {
        match self.mode {
            AppMode::Generating => self.tick_gen()?,
            AppMode::Solving => self.tick_solve()?,
        }
        Ok(())
    }

    fn tick_gen(&mut self) -> Result<()> {
        if !self.config.interactive_gen() {
            while !self.generator.is_done() {
                self.generator.tick(&mut self.maze)?;
            }
        }

        if self.generator.is_done() {
            self.maze.highlight_bright.clear();
            self.maze.highlight_medium.clear();
            self.maze.highlight_dark.clear();
            self.maze.explored.clear();
            self.mode = AppMode::Solving;
        } else {
            self.generator.tick(&mut self.maze)?;
        }
        Ok(())
    }

    fn tick_solve(&mut self) -> Result<()> {
        if !self.config.interactive_solve() {
            while !self.solver.is_done() {
                self.solver.tick(&mut self.maze)?;
            }
        }

        if !self.solver.is_done() {
            self.solver.tick(&mut self.maze)?;
        }

        Ok(())
    }
}

fn main() {
    let config = Config::from_args();
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new(
        format!(
            "Mazes! Generator: {:?} Solver: {:?}",
            config.generator(),
            config.solver()
        ),
        [config.window_width(), config.window_height()],
    ).opengl(opengl)
        .exit_on_esc(false)
        .build()
        .expect("Error creating window");

    let mut gl = GlGraphics::new(opengl);
    let mut app = match App::new(&config) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            return;
        }
    };

    let mut updating = true;
    let mut event_settings = EventSettings::new();
    event_settings.ups = config.ups();
    event_settings.max_fps = config.fps();

    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if !updating {
            continue;
        }

        if let Some(r) = e.render_args() {
            app.render(&r, &mut gl);
        }

        if e.update_args().is_some() {
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
