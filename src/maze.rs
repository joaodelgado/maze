use std::collections::{HashMap, HashSet};
use std::fmt;

use graphics::types::Color;
use opengl_graphics::GlGraphics;
use piston::input::RenderArgs;

use config::{Config, CELL_WALL_WIDTH, COLOR_BACKGROUND, COLOR_END, COLOR_EXPLORED,
             COLOR_HIGHLIGHT_BRIGHT, COLOR_HIGHLIGHT_DARK, COLOR_HIGHLIGHT_MEDIUM, COLOR_START,
             COLOR_WALL};
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

    pub fn north_wall(&self, config: &Config) -> Wall {
        let as_point: Point = self.into_point(config.cell_width(), config.cell_height());
        let offset = match config.cell_height() % 2 == 0 {
            true => config.cell_height() as i32 / 2,
            false => (config.cell_height() + 1) as i32 / 2,
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
        let offset = match config.cell_width() % 2 == 0 {
            true => config.cell_width() as i32 / 2,
            false => (config.cell_width() + 1) as i32 / 2,
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
        self.x >= 0 && self.x <= (maze_width - 1) as i32 && self.y >= 0
            && self.y <= (maze_height - 1) as i32
    }

    pub fn neighbour(
        &self,
        direction: Direction,
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
                self.neighbour(Direction::North, maze_width, maze_height),
                Direction::North,
            ),
            (
                self.neighbour(Direction::East, maze_width, maze_height),
                Direction::East,
            ),
            (
                self.neighbour(Direction::South, maze_width, maze_height),
                Direction::South,
            ),
            (
                self.neighbour(Direction::West, maze_width, maze_height),
                Direction::West,
            ),
        ].into_iter()
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

    fn render(&self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::line::Line;

        let offset = match self.size % 2 == 0 {
            true => 0,
            false => 1,
        };

        let (start, end): (Point, Point) = match self.orientation {
            Orientation::Horizontal => (
                [
                    self.center.x - (self.size + offset) as i32 / 2,
                    self.center.y,
                ].into(),
                [self.center.x + self.size as i32 / 2, self.center.y].into(),
            ),
            Orientation::Vertical => (
                [
                    self.center.x,
                    self.center.y - (self.size + offset) as i32 / 2,
                ].into(),
                [self.center.x, self.center.y + self.size as i32 / 2].into(),
            ),
        };

        gl.draw(args.viewport(), |c, gl| {
            Line::new(COLOR_WALL, CELL_WALL_WIDTH / 2.0).draw(
                [start.x as f64, start.y as f64, end.x as f64, end.y as f64],
                &c.draw_state,
                c.transform,
                gl,
            );
        });
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
    width: f64,
    height: f64,
}

impl Cell {
    pub fn new(center: Point, width: u32, height: u32) -> Cell {
        Cell {
            center: center,
            width: width as f64,
            height: height as f64,
        }
    }

    pub fn render(&self, color: Option<Color>, args: &RenderArgs, gl: &mut GlGraphics) {
        gl.draw(args.viewport(), |c, gl| {
            use graphics::rectangle::{centered, Rectangle};

            if let Some(color) = color {
                Rectangle::new(color).draw(
                    centered([
                        self.center.x as f64,
                        self.center.y as f64,
                        self.width / 2.0,
                        self.height / 2.0,
                    ]),
                    &c.draw_state,
                    c.transform,
                    gl,
                );
            }
        });
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
    pub fn new(config: &'a Config) -> Maze {
        let mut walls = HashSet::new();
        let mut cells = HashMap::new();

        for y in 0..config.maze_height() {
            for x in 0..config.maze_width() {
                let coord: Coord = [x, y].into();

                for wall in coord.walls(config).iter() {
                    walls.insert(*wall);
                }

                let mut cell = Cell::new(
                    coord.into_point(config.cell_width(), config.cell_height()),
                    config.cell_width(),
                    config.cell_height(),
                );

                cells.insert(coord, cell);
            }
        }

        let mut explored = HashSet::new();
        explored.insert([0, 0].into());

        Maze {
            config: config,
            walls: walls,
            cells: cells,
            start: [0, 0].into(),
            end: [config.maze_width() - 1, config.maze_height() - 1].into(),
            explored: explored,
            highlight_bright: HashSet::new(),
            highlight_medium: HashSet::new(),
            highlight_dark: HashSet::new(),
        }
    }

    pub fn render(&self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::clear;

        clear(COLOR_BACKGROUND, gl);

        for (coord, cell) in &self.cells {
            let color;
            if *coord == self.start {
                color = Some(COLOR_START);
            } else if *coord == self.end {
                color = Some(COLOR_END);
            } else if self.highlight_bright.contains(&coord) {
                color = Some(COLOR_HIGHLIGHT_BRIGHT);
            } else if self.highlight_medium.contains(&coord) {
                color = Some(COLOR_HIGHLIGHT_MEDIUM);
            } else if self.highlight_dark.contains(&coord) {
                color = Some(COLOR_HIGHLIGHT_DARK);
            } else if self.explored.contains(&coord) {
                color = Some(COLOR_EXPLORED);
            } else {
                color = None;
            }

            cell.render(color, args, gl);
        }

        for wall in &self.walls {
            wall.render(args, gl);
        }
    }

    pub fn divided_coords(&self, wall: &Wall) -> (Coord, Coord) {
        match wall.orientation {
            Orientation::Horizontal => {
                let up: Coord = Point {
                    y: wall.center.y - self.config.cell_height() as i32 / 2,
                    ..wall.center
                }.into_coord(self.config.cell_width(), self.config.cell_height());
                let down: Coord =
                    Point {
                        y: wall.center.y + self.config.cell_height() as i32 / 2,
                        ..wall.center
                    }.into_coord(self.config.cell_width(), self.config.cell_height());

                (up, down)
            }
            Orientation::Vertical => {
                let left: Coord =
                    Point {
                        x: wall.center.x - self.config.cell_width() as i32 / 2,
                        ..wall.center
                    }.into_coord(self.config.cell_width(), self.config.cell_height());
                let right: Coord =
                    Point {
                        x: wall.center.x + self.config.cell_width() as i32 / 2,
                        ..wall.center
                    }.into_coord(self.config.cell_width(), self.config.cell_height());

                (left, right)
            }
        }
    }

    pub fn link(&mut self, c1: &Coord, c2: &Coord) -> Result<()> {
        match c1.neighbours(self.config.maze_width(), self.config.maze_height())
            .iter()
            .filter(|n| n.0 == *c2)
            .next()
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
        match direction {
            Direction::North => self.north_wall(coord),
            Direction::East => self.east_wall(coord),
            Direction::South => self.south_wall(coord),
            Direction::West => self.west_wall(coord),
        }
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
}
