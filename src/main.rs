#![feature(option_filter)]
#![feature(drain_filter)]
#![feature(vec_remove_item)]

#[macro_use]
extern crate structopt;
extern crate ggez;
extern crate rand;

mod config;
mod error;
mod generator;
mod maze;
mod solver;

use std::time::{Duration, Instant};

use rand::{SeedableRng, StdRng};
use structopt::StructOpt;

use ggez::*;

use config::{Config, COLOR_BACKGROUND};
use error::Result;
use generator::Generator;
use maze::Maze;
use solver::Solver;

#[derive(Default)]
struct Timer {
    start: Option<Instant>,
    end: Option<Instant>,
}

impl Timer {
    fn is_running(&self) -> bool {
        self.start.is_some() && self.end.is_none()
    }

    fn is_stopped(&self) -> bool {
        !self.is_running()
    }

    fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    fn stop(&mut self) {
        self.end = Some(Instant::now());
    }

    fn restart(&mut self) {
        self.reset();
        self.start();
    }

    fn reset(&mut self) {
        self.start = None;
        self.end = None;
    }

    fn duration(&self) -> Duration {
        let start = self.start.expect("Timer not running");

        match self.end {
            None => start.elapsed(),
            Some(end) => end.duration_since(start),
        }
    }
}

enum AppMode {
    Generating,
    Solving,
}

struct MainState<'a> {
    maze: Maze<'a>,
    mode: AppMode,

    generator: Box<Generator>,
    solver: Box<Solver>,

    fps_timer: Timer,
    gen_timer: Timer,
    solve_timer: Timer,

    config: &'a Config,
    random: StdRng,
    paused: bool,
}

impl<'a> MainState<'a> {
    fn new(config: &'a Config) -> Result<MainState<'a>> {
        let mut random = if let Some(seed) = config.seed() {
            StdRng::from_seed(&seed)
        } else {
            StdRng::new().unwrap()
        };
        let maze = Maze::new(&config, &mut random);
        let generator = config.generator().init(&maze, &mut random);
        let solver = config.solver().init(&maze);

        let mut fps_timer = Timer::default();
        fps_timer.start();

        Ok(MainState {
            maze,
            mode: AppMode::Generating,

            generator,
            solver,

            fps_timer,
            gen_timer: Timer::default(),
            solve_timer: Timer::default(),

            config,
            random,
            paused: false,
        })
    }

    fn tick_gen(&mut self) -> Result<()> {
        if self.gen_timer.is_stopped() {
            self.gen_timer.start();
        }

        if !self.config.interactive_gen() {
            while !self.generator.is_done() {
                self.generator.tick(&mut self.maze, &mut self.random)?;
            }
            self.gen_timer.stop();
        }

        if self.generator.is_done() {
            if self.gen_timer.is_running() {
                self.gen_timer.stop();
                println!("Gen time: {} seconds", self.gen_timer.duration().as_secs());
            }

            self.maze.highlight_bright.clear();
            self.maze.highlight_medium.clear();
            self.maze.highlight_dark.clear();
            self.maze.explored.clear();
            self.mode = AppMode::Solving;
        } else {
            self.generator.tick(&mut self.maze, &mut self.random)?;
        }
        Ok(())
    }

    fn tick_solve(&mut self) -> Result<()> {
        if self.solve_timer.is_stopped() {
            self.solve_timer.start();
        }

        if !self.config.interactive_solve() {
            while !self.solver.is_done() {
                self.solver.tick(&mut self.maze)?;
            }
            self.solve_timer.stop();
        }

        if !self.solver.is_done() {
            self.solver.tick(&mut self.maze)?;
        } else if self.solve_timer.is_running() {
            self.solve_timer.stop();
            println!(
                "Solve time: {} seconds",
                self.solve_timer.duration().as_secs()
            );
        }

        Ok(())
    }
}

impl<'a> event::EventHandler for MainState<'a> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.config.print_fps() && self.fps_timer.duration() > Duration::from_secs(1) {
            println!("FPS: {}", timer::get_fps(ctx));
            self.fps_timer.restart();
        }
        while timer::check_update_time(ctx, self.config.ups()) {
            if self.paused {
                return Ok(());
            }

            match self.mode {
                AppMode::Generating => match self.tick_gen() {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("[ERROR] {}", e);
                        self.paused = true;
                    }
                },
                AppMode::Solving => match self.tick_solve() {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("[ERROR] {}", e);
                        self.paused = true;
                    }
                },
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        self.maze.render(ctx)?;

        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }
}

fn main() {
    let config = Config::from_args();

    let mut state = match MainState::new(&config) {
        Ok(state) => state,
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            return;
        }
    };

    let title = format!(
        "Mazes! Generator: {:?} Solver: {:?}",
        config.generator(),
        config.solver()
    );
    let mut ctx = ContextBuilder::new("maze", "João Delgado")
        .window_setup(conf::WindowSetup::default().title(&title))
        .window_mode(
            conf::WindowMode::default().dimensions(config.window_width(), config.window_height()),
        )
        .build()
        .expect("Error building context");

    graphics::set_background_color(&mut ctx, COLOR_BACKGROUND.into());

    if let Err(e) = event::run(&mut ctx, &mut state) {
        println!("[ERROR] {}", e);
    }
}
