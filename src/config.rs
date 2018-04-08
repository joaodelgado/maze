use graphics::types::Color;

use generator::GeneratorType;

pub const COLOR_BACKGROUND: Color = [7.0 / 255.0, 16.0 / 255.0, 19.0 / 255.0, 1.0];
pub const COLOR_EXPLORED: Color = [14.0 / 255.0, 71.0 / 255.0, 73.0 / 255.0, 1.0];
pub const COLOR_START: Color = [149.0 / 255.0, 198.0 / 255.0, 35.0 / 255.0, 1.0];
pub const COLOR_END: Color = [229.0 / 255.0, 88.0 / 255.0, 18.0 / 255.0, 1.0];
pub const COLOR_WALL: Color = [239.0 / 255.0, 231.0 / 255.0, 218.0 / 255.0, 1.0];
pub const COLOR_HIGHLIGHT: Color = [57.0 / 255.0, 104.0 / 255.0, 106.0 / 255.0, 1.0];

pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;

pub const MAZE_WIDTH: u32 = WINDOW_WIDTH / 40;
pub const MAZE_HEIGHT: u32 = WINDOW_HEIGHT / 40;

pub const CELL_WIDTH: u32 = WINDOW_WIDTH / MAZE_WIDTH;
pub const CELL_HEIGHT: u32 = WINDOW_HEIGHT / MAZE_HEIGHT;
pub const CELL_WALL_WIDTH: f64 = 1.5;

#[derive(StructOpt, Debug)]
#[structopt(name = "populate", about = "Generate packages")]
pub struct Config {
    /// The algorithm to used when generating the maze
    #[structopt(short = "g", long = "generator", default_value = "dfs",
                raw(possible_values = "&GeneratorType::variants()"))]
    pub generator: GeneratorType,

    /// Updates per second
    #[structopt(long = "ups", default_value = "60")]
    pub ups: u64,
}
