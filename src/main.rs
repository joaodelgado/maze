#![feature(option_filter)]
#![feature(drain_filter)]
#![feature(vec_remove_item)]

#[macro_use]
extern crate structopt;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_app;
extern crate rand;

mod config;
mod error;
mod generator;
mod maze;
mod solver;

use structopt::StructOpt;

use piston_app::prelude::*;

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
    paused: bool,
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
            paused: false,
        })
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

impl<'a> Controller for App<'a> {
    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) {
        self.maze.render(args, gl);
    }

    fn tick(&mut self, _args: &UpdateArgs) {
        if self.paused {
            return;
        }
        match self.mode {
            AppMode::Generating => match self.tick_gen() {
                Ok(()) => {}
                Err(e) => eprintln!("[ERROR] {}", e),
            },
            AppMode::Solving => match self.tick_solve() {
                Ok(()) => {}
                Err(e) => eprintln!("[ERROR] {}", e),
            },
        }
    }
}

fn main() {
    let config = Config::from_args();

    let app = match App::new(&config) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            return;
        }
    };

    let mut piston_app = AppBuilder::new(app, [config.window_width(), config.window_height()])
        .title(format!(
            "Mazes! Generator: {:?} Solver: {:?}",
            config.generator(),
            config.solver()
        ))
        .ups(config.ups())
        .fps(config.fps())
        .build()
        .expect("Error creating window");

    piston_app.run();
}
