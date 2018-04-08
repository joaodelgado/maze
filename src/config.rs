use graphics::types::Color;

use generator::GeneratorType;
use solver::SolverType;

pub const COLOR_BACKGROUND: Color = [7.0 / 255.0, 16.0 / 255.0, 19.0 / 255.0, 1.0];
pub const COLOR_START: Color = [149.0 / 255.0, 198.0 / 255.0, 35.0 / 255.0, 1.0];
pub const COLOR_END: Color = [229.0 / 255.0, 88.0 / 255.0, 18.0 / 255.0, 1.0];
pub const COLOR_WALL: Color = [239.0 / 255.0, 231.0 / 255.0, 218.0 / 255.0, 1.0];

pub const COLOR_EXPLORED: Color = [14.0 / 255.0, 71.0 / 255.0, 73.0 / 255.0, 1.0];

pub const COLOR_HIGHLIGHT_BRIGHT: Color = [163.0 / 255.0, 187.0 / 255.0, 173.0 / 255.0, 1.0];
pub const COLOR_HIGHLIGHT_MEDIUM: Color = [53.0 / 255.0, 114.0 / 255.0, 102.0 / 255.0, 1.0];
pub const COLOR_HIGHLIGHT_DARK: Color = [57.0 / 255.0, 104.0 / 255.0, 106.0 / 255.0, 1.0];

pub const CELL_WALL_WIDTH: f64 = 1.0;

#[derive(StructOpt, Debug)]
#[structopt(name = "populate", about = "Generate packages")]
pub struct Config {
    /// The algorithm to used when generating the maze
    #[structopt(short = "g", long = "generator", default_value = "dfs",
                raw(possible_values = "&GeneratorType::variants()"))]
    generator: GeneratorType,

    /// The algorithm to used when solving the maze
    #[structopt(short = "s", long = "solver", default_value = "dfs",
                raw(possible_values = "&SolverType::variants()"))]
    solver: SolverType,

    /// Updates per second
    #[structopt(long = "ups", default_value = "60")]
    ups: u64,

    /// Frames per second
    #[structopt(long = "fps", default_value = "60")]
    fps: u64,

    /// The size of each sell in pixels
    #[structopt(long = "cell-size", default_value = "40")]
    cell_size: u32,

    /// The width of the maze in cells
    #[structopt(short = "w", long = "width", default_value = "32")]
    width: u32,

    /// The height of the maze in cells
    #[structopt(short = "h", long = "height", default_value = "18")]
    height: u32,
}

impl Config {
    #[inline]
    pub fn generator(&self) -> GeneratorType {
        self.generator
    }

    #[inline]
    pub fn solver(&self) -> SolverType {
        self.solver
    }

    #[inline]
    pub fn ups(&self) -> u64 {
        self.ups
    }

    #[inline]
    pub fn fps(&self) -> u64 {
        self.fps
    }

    #[inline]
    pub fn cell_width(&self) -> u32 {
        self.cell_size
    }

    #[inline]
    pub fn cell_height(&self) -> u32 {
        self.cell_size
    }

    #[inline]
    pub fn maze_width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn maze_height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn window_width(&self) -> u32 {
        self.cell_width() * self.maze_width()
    }

    #[inline]
    pub fn window_height(&self) -> u32 {
        self.cell_height() * self.maze_height()
    }
}
