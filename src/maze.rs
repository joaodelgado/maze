use std;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use ggez::graphics;
use ggez::graphics::Color;
use ggez::graphics::MeshBuilder;
use ggez::mint::Point2;
use ggez::{Context, GameResult};
use rand::{Rng, StdRng};

use config::{
    Config, CELL_WALL_WIDTH, COLOR_END, COLOR_EXPLORED, COLOR_HIGHLIGHT_BRIGHT,
    COLOR_HIGHLIGHT_DARK, COLOR_HIGHLIGHT_MEDIUM, COLOR_START, COLOR_WALL,
};
use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Orientation::Horizontal => write!(f, "Horizontal"),
            Orientation::Vertical => write!(f, "Vertical"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    fn into_coord(self, cell_width: u32, cell_height: u32) -> Coord {
        Coord {
            x: (self.x - cell_width as i32 / 2) / cell_width as i32,
            y: (self.y - cell_height as i32 / 2) / cell_height as i32,
        }
    }
}

impl From<[i32; 2]> for Point {
    fn from(a: [i32; 2]) -> Point {
        Point { x: a[0], y: a[1] }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn random(max_x: u32, max_y: u32, random: &mut StdRng) -> Coord {
        Coord {
            x: random.gen_range(0, max_x) as i32,
            y: random.gen_range(0, max_y) as i32,
        }
    }

    pub fn manhattan_dist(&self, other: &Coord) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
    }

    pub fn walls(&self, config: &Config) -> [Wall; 4] {
        [
            self.north_wall(config),
            self.east_wall(config),
            self.south_wall(config),
            self.west_wall(config),
        ]
    }

    pub fn wall(&self, direction: &Direction, config: &Config) -> Wall {
        match direction {
            Direction::North => self.north_wall(config),
            Direction::East => self.east_wall(config),
            Direction::South => self.south_wall(config),
            Direction::West => self.west_wall(config),
        }
    }

    pub fn north_wall(&self, config: &Config) -> Wall {
        let as_point: Point = self.into_point(config.cell_width(), config.cell_height());
        let offset = if config.cell_height() % 2 == 0 {
            config.cell_height() as i32 / 2
        } else {
            (config.cell_height() + 1) as i32 / 2
        };
        Wall {
            center: [as_point.x, as_point.y - offset].into(),
            orientation: Orientation::Horizontal,
            border: self.y == 0,
            size: config.cell_height(),
        }
    }

    pub fn east_wall(&self, config: &Config) -> Wall {
        let as_point: Point = self.into_point(config.cell_width(), config.cell_height());
        Wall {
            center: [as_point.x + config.cell_width() as i32 / 2, as_point.y].into(),
            orientation: Orientation::Vertical,
            border: self.x == (config.maze_width() - 1) as i32,
            size: config.cell_width(),
        }
    }

    pub fn south_wall(&self, config: &Config) -> Wall {
        let as_point: Point = self.into_point(config.cell_width(), config.cell_height());
        Wall {
            center: [as_point.x, as_point.y + config.cell_height() as i32 / 2].into(),
            orientation: Orientation::Horizontal,
            border: self.y == (config.maze_height() - 1) as i32,
            size: config.cell_height(),
        }
    }

    pub fn west_wall(&self, config: &Config) -> Wall {
        let as_point: Point = self.into_point(config.cell_width(), config.cell_height());
        let offset = if config.cell_width() % 2 == 0 {
            config.cell_width() as i32 / 2
        } else {
            (config.cell_width() + 1) as i32 / 2
        };

        Wall {
            center: [as_point.x - offset, as_point.y].into(),
            orientation: Orientation::Vertical,
            border: self.x == 0,
            size: config.cell_width(),
        }
    }

    pub fn into_point(self, cell_width: u32, cell_height: u32) -> Point {
        Point {
            x: self.x * cell_width as i32 + cell_width as i32 / 2,
            y: self.y * cell_height as i32 + cell_height as i32 / 2,
        }
    }

    pub fn valid_coord(&self, maze_width: u32, maze_height: u32) -> bool {
        self.x >= 0
            && self.x <= (maze_width - 1) as i32
            && self.y >= 0
            && self.y <= (maze_height - 1) as i32
    }

    pub fn neighbour(
        &self,
        direction: &Direction,
        maze_width: u32,
        maze_height: u32,
    ) -> Option<Coord> {
        let candidate = match direction {
            Direction::North => Coord {
                x: self.x,
                y: self.y - 1,
            },
            Direction::East => Coord {
                x: self.x + 1,
                y: self.y,
            },
            Direction::South => Coord {
                x: self.x,
                y: self.y + 1,
            },
            Direction::West => Coord {
                x: self.x - 1,
                y: self.y,
            },
        };

        if candidate.valid_coord(maze_width, maze_height) {
            Some(candidate)
        } else {
            None
        }
    }

    pub fn neighbours(&self, maze_width: u32, maze_height: u32) -> Vec<(Coord, Direction)> {
        vec![
            (
                self.neighbour(&Direction::North, maze_width, maze_height),
                Direction::North,
            ),
            (
                self.neighbour(&Direction::East, maze_width, maze_height),
                Direction::East,
            ),
            (
                self.neighbour(&Direction::South, maze_width, maze_height),
                Direction::South,
            ),
            (
                self.neighbour(&Direction::West, maze_width, maze_height),
                Direction::West,
            ),
        ]
        .into_iter()
        .filter(|(coord, _)| coord.is_some())
        .map(|(c, d)| (c.unwrap(), d))
        .collect()
    }
}

impl From<[i32; 2]> for Coord {
    fn from(a: [i32; 2]) -> Coord {
        Coord { x: a[0], y: a[1] }
    }
}

impl From<[u32; 2]> for Coord {
    fn from(a: [u32; 2]) -> Coord {
        Coord {
            x: a[0] as i32,
            y: a[1] as i32,
        }
    }
}

impl FromStr for Coord {
    type Err = ParseIntError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let coords: Vec<&str> = s
            .trim_matches(|p| p == '(' || p == ')')
            .split(',')
            .collect();

        let x_fromstr = coords[0].parse::<i32>()?;
        let y_fromstr = coords[1].parse::<i32>()?;

        Ok(Coord {
            x: x_fromstr,
            y: y_fromstr,
        })
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Wall {
    center: Point,
    orientation: Orientation,
    border: bool,
    size: u32,
}

impl Wall {
    pub fn removable(&self) -> bool {
        !self.border
    }

    fn build_mesh(&self, mb: &mut MeshBuilder, color: Color) {
        let offset = if self.size % 2 == 0 { 0.0 } else { 1.0 };

        let (start, end) = match self.orientation {
            Orientation::Horizontal => (
                Point2 {
                    x: self.center.x as f32 - (self.size as f32 + offset) / 2.0,
                    y: self.center.y as f32,
                },
                Point2 {
                    x: self.center.x as f32 + self.size as f32 / 2.0,
                    y: self.center.y as f32,
                },
            ),
            Orientation::Vertical => (
                Point2 {
                    x: self.center.x as f32,
                    y: self.center.y as f32 - (self.size as f32 + offset) / 2.0,
                },
                Point2 {
                    x: self.center.x as f32,
                    y: self.center.y as f32 + self.size as f32 / 2.0,
                },
            ),
        };

        mb.line(&[start, end], CELL_WALL_WIDTH, color).unwrap();
    }
}

impl fmt::Display for Wall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Wall {} @ {}]", self.orientation, self.center)
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    center: Point,
    width: f32,
    height: f32,
}

impl Cell {
    pub fn new(center: Point, width: u32, height: u32) -> Cell {
        Cell {
            center,
            width: width as f32,
            height: height as f32,
        }
    }

    pub fn build_mesh(&self, mb: &mut MeshBuilder, color: Color) {
        use graphics::DrawMode;
        let x1 = self.center.x as f32 - self.width / 2.0;
        let x2 = self.center.x as f32 + self.width / 2.0;
        let y1 = self.center.y as f32 - self.height / 2.0;
        let y2 = self.center.y as f32 + self.height / 2.0;
        mb.polygon(
            DrawMode::fill(),
            &[
                Point2 { x: x1, y: y1 },
                Point2 { x: x2, y: y1 },
                Point2 { x: x2, y: y2 },
                Point2 { x: x1, y: y2 },
            ],
            color,
        )
        .unwrap();
    }
}

#[derive(Debug)]
pub struct Maze<'a> {
    config: &'a Config,
    pub walls: HashSet<Wall>,
    pub cells: HashMap<Coord, Cell>,
    pub start: Coord,
    pub end: Coord,
    pub explored: HashSet<Coord>,
    pub highlight_bright: HashSet<Coord>,
    pub highlight_medium: HashSet<Coord>,
    pub highlight_dark: HashSet<Coord>,
}

impl<'a> Maze<'a> {
    pub fn new<'b>(config: &'a Config, random: &'b mut StdRng) -> Maze<'a> {
        let mut walls = HashSet::new();
        let mut cells = HashMap::new();

        for y in 0..config.maze_height() {
            for x in 0..config.maze_width() {
                let coord: Coord = [x, y].into();

                for wall in &coord.walls(config) {
                    walls.insert(*wall);
                }

                let cell = Cell::new(
                    coord.into_point(config.cell_width(), config.cell_height()),
                    config.cell_width(),
                    config.cell_height(),
                );

                cells.insert(coord, cell);
            }
        }

        let start = match config.start() {
            Some(coord) => coord,
            None => Coord::random(config.maze_width(), config.maze_height(), random),
        };
        let mut end = match config.end() {
            Some(coord) => coord,
            None => Coord::random(config.maze_width(), config.maze_height(), random),
        };

        while start == end {
            end = Coord::random(config.maze_width(), config.maze_height(), random);
        }

        let explored = HashSet::new();

        Maze {
            config,
            walls,
            cells,
            start,
            end,
            explored,
            highlight_bright: HashSet::new(),
            highlight_medium: HashSet::new(),
            highlight_dark: HashSet::new(),
        }
    }

    pub fn render(&self, ctx: &mut Context) -> GameResult<()> {
        let mut start_mb = MeshBuilder::new();
        let mut end_mb = MeshBuilder::new();
        let mut highlight_bright_mb = MeshBuilder::new();
        let mut highlight_medium_mb = MeshBuilder::new();
        let mut highlight_dark_mb = MeshBuilder::new();
        let mut explored_mb = MeshBuilder::new();

        for (coord, cell) in &self.cells {
            if *coord == self.start {
                cell.build_mesh(&mut start_mb, COLOR_START.into());
            } else if *coord == self.end {
                cell.build_mesh(&mut end_mb, COLOR_END.into());
            } else if self.highlight_bright.contains(&coord) {
                cell.build_mesh(&mut highlight_bright_mb, COLOR_HIGHLIGHT_BRIGHT.into());
            } else if self.highlight_medium.contains(&coord) {
                cell.build_mesh(&mut highlight_medium_mb, COLOR_HIGHLIGHT_MEDIUM.into());
            } else if self.highlight_dark.contains(&coord) {
                cell.build_mesh(&mut highlight_dark_mb, COLOR_HIGHLIGHT_DARK.into());
            } else if self.explored.contains(&coord) {
                cell.build_mesh(&mut explored_mb, COLOR_EXPLORED.into());
            }
        }

        if let Ok(m) = start_mb.build(ctx) {
            graphics::draw(ctx, &m, graphics::DrawParam::default())?;
        }
        if let Ok(m) = end_mb.build(ctx) {
            graphics::draw(ctx, &m, graphics::DrawParam::default())?;
        }
        if let Ok(m) = highlight_bright_mb.build(ctx) {
            graphics::draw(ctx, &m, graphics::DrawParam::default())?;
        }
        if let Ok(m) = highlight_medium_mb.build(ctx) {
            graphics::draw(ctx, &m, graphics::DrawParam::default())?;
        }
        if let Ok(m) = highlight_dark_mb.build(ctx) {
            graphics::draw(ctx, &m, graphics::DrawParam::default())?;
        }
        if let Ok(m) = explored_mb.build(ctx) {
            graphics::draw(ctx, &m, graphics::DrawParam::default())?;
        }

        // Use a list of mesh builder because we might run into vertex buffer limits
        let mut walls_mbs = self
            .walls
            .iter()
            .collect::<Vec<_>>()
            .chunks(10_000)
            .map(|walls| {
                let mut mb = MeshBuilder::new();
                for wall in walls {
                    wall.build_mesh(&mut mb, COLOR_WALL.into());
                }
                mb
            })
            .collect::<Vec<_>>();

        for mb in &mut walls_mbs {
            let mesh = mb.build(ctx)?;
            graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
        }

        Ok(())
    }

    pub fn divided_coords(&self, wall: &Wall) -> (Coord, Coord) {
        match wall.orientation {
            Orientation::Horizontal => {
                let up: Coord = Point {
                    y: wall.center.y - self.config.cell_height() as i32 / 2,
                    ..wall.center
                }
                .into_coord(self.config.cell_width(), self.config.cell_height());
                let down: Coord = Point {
                    y: wall.center.y + self.config.cell_height() as i32 / 2,
                    ..wall.center
                }
                .into_coord(self.config.cell_width(), self.config.cell_height());

                (up, down)
            }
            Orientation::Vertical => {
                let left: Coord = Point {
                    x: wall.center.x - self.config.cell_width() as i32 / 2,
                    ..wall.center
                }
                .into_coord(self.config.cell_width(), self.config.cell_height());
                let right: Coord = Point {
                    x: wall.center.x + self.config.cell_width() as i32 / 2,
                    ..wall.center
                }
                .into_coord(self.config.cell_width(), self.config.cell_height());

                (left, right)
            }
        }
    }

    pub fn link(&mut self, c1: &Coord, c2: &Coord) -> Result<()> {
        match c1
            .neighbours(self.config.maze_width(), self.config.maze_height())
            .iter()
            .find(|n| n.0 == *c2)
        {
            Some((_, direction)) => {
                let wall = match direction {
                    Direction::North => self.north_wall(c1),
                    Direction::East => self.east_wall(c1),
                    Direction::South => self.south_wall(c1),
                    Direction::West => self.west_wall(c1),
                };

                if !wall.removable() {
                    return Err(Error::BorderWall(wall));
                }

                self.walls.remove(&wall);

                Ok(())
            }
            None => Err(Error::NotNeighbours(*c1, *c2)),
        }
    }

    /*
     * Coords
     */

    #[allow(unused)]
    pub fn walls(&self, coord: &Coord) -> [Wall; 4] {
        coord.walls(self.config)
    }

    pub fn wall(&self, coord: &Coord, direction: &Direction) -> Wall {
        coord.wall(direction, self.config)
    }

    pub fn north_wall(&self, coord: &Coord) -> Wall {
        coord.north_wall(self.config)
    }

    pub fn east_wall(&self, coord: &Coord) -> Wall {
        coord.east_wall(self.config)
    }

    pub fn south_wall(&self, coord: &Coord) -> Wall {
        coord.south_wall(self.config)
    }

    pub fn west_wall(&self, coord: &Coord) -> Wall {
        coord.west_wall(self.config)
    }

    pub fn neighbour(&self, coord: &Coord, direction: &Direction) -> Option<Coord> {
        coord.neighbour(
            direction,
            self.config.maze_width(),
            self.config.maze_height(),
        )
    }

    pub fn neighbours(&self, coord: &Coord) -> Vec<(Coord, Direction)> {
        coord.neighbours(self.config.maze_width(), self.config.maze_height())
    }

    pub fn connected_neighbours(&self, coord: &Coord) -> Vec<(Coord, Direction)> {
        coord
            .neighbours(self.config.maze_width(), self.config.maze_height())
            .into_iter()
            .filter(|(_, d)| {
                let wall = self.wall(coord, d);
                !self.walls.contains(&wall)
            })
            .collect()
    }

    /*
     * Maze config
     */

    pub fn maze_width(&self) -> u32 {
        self.config.maze_width()
    }

    pub fn maze_height(&self) -> u32 {
        self.config.maze_height()
    }
}
