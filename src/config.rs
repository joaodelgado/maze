use generator::GeneratorType;
use maze::Coord;
use solver::SolverType;

pub const COLOR_BACKGROUND: (u8, u8, u8) = (7, 16, 19);
pub const COLOR_START: (u8, u8, u8) = (149, 198, 35);
pub const COLOR_END: (u8, u8, u8) = (229, 88, 18);

pub const COLOR_WALL: (u8, u8, u8) = (239, 231, 218);

pub const COLOR_EXPLORED: (u8, u8, u8) = (14, 71, 73);
pub const COLOR_HIGHLIGHT_BRIGHT: (u8, u8, u8) = (163, 187, 173);
pub const COLOR_HIGHLIGHT_MEDIUM: (u8, u8, u8) = (53, 114, 102);
pub const COLOR_HIGHLIGHT_DARK: (u8, u8, u8) = (57, 104, 106);

pub const CELL_WALL_WIDTH: f32 = 1.0;

#[derive(StructOpt, Debug)]
#[structopt(name = "populate", about = "Generate packages")]
pub struct Config {
    /// The algorithm to use when generating the maze
    #[structopt(short = "g", long = "generator", default_value = "dfs",
                raw(possible_values = "&GeneratorType::variants()"))]
    generator: GeneratorType,

    /// The algorithm to use when solving the maze
    #[structopt(short = "s", long = "solver", default_value = "astar",
                raw(possible_values = "&SolverType::variants()"))]
    solver: SolverType,

    /// Updates per second
    #[structopt(long = "ups", default_value = "60")]
    ups: u32,

    /// The size of each sell in pixels
    #[structopt(long = "cell-size", default_value = "40")]
    cell_size: u32,

    /// The width of the maze in cells
    #[structopt(short = "w", long = "width", default_value = "32")]
    width: u32,

    /// The height of the maze in cells
    #[structopt(short = "h", long = "height", default_value = "18")]
    height: u32,

    /// The starting point of the maze
    #[structopt(long = "start")]
    start: Option<Coord>,

    /// The ending point of the maze
    #[structopt(long = "end")]
    end: Option<Coord>,

    /// If provided, the maze generation is done without any visualization
    #[structopt(long = "no-interactive-gen")]
    no_interactive_gen: bool,

    /// If provided, the maze solving is done without any visualization
    #[structopt(long = "no-interactive-solve")]
    no_interactive_solve: bool,

    /// If provided, average FPS are printed to the console
    #[structopt(long = "no-print-fps")]
    print_fps: bool,

    /// If provided, the maze will be generated using a static seed
    #[structopt(long = "seed")]
    seed: Option<u32>,
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
    pub fn ups(&self) -> u32 {
        self.ups
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

    #[inline]
    pub fn start(&self) -> Option<Coord> {
        self.start
    }

    #[inline]
    pub fn end(&self) -> Option<Coord> {
        self.end
    }

    #[inline]
    pub fn interactive_gen(&self) -> bool {
        !self.no_interactive_gen
    }

    #[inline]
    pub fn interactive_solve(&self) -> bool {
        !self.no_interactive_solve
    }

    #[inline]
    pub fn print_fps(&self) -> bool {
        !self.print_fps
    }

    #[inline]
    pub fn seed(&self) -> Option<[usize; 4]> {
        match self.seed {
            None => None,
            Some(seed) => {
                let b1 = ((seed >> 24) & 0xff) as usize;
                let b2 = ((seed >> 16) & 0xff) as usize;
                let b3 = ((seed >> 8) & 0xff) as usize;
                let b4 = (seed & 0xff) as usize;
                Some([b1, b2, b3, b4])
            }
        }
    }
}
