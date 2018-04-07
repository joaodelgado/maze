use std::collections::{HashMap, HashSet};
use std::fmt;

use graphics::types::Color;
use opengl_graphics::GlGraphics;
use piston::input::RenderArgs;

use config::*;
use errors::{Error, Result};

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

impl From<[i32; 2]> for Point {
    fn from(a: [i32; 2]) -> Point {
        Point { x: a[0], y: a[1] }
    }
}

impl Into<Coord> for Point {
    fn into(self) -> Coord {
        Coord {
            x: (self.x - CELL_WIDTH as i32 / 2) / CELL_WIDTH as i32,
            y: (self.y - CELL_HEIGHT as i32 / 2) / CELL_HEIGHT as i32,
        }
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
    pub fn walls(&self) -> [Wall; 4] {
        [
            self.north_wall(),
            self.east_wall(),
            self.south_wall(),
            self.west_wall(),
        ]
    }

    pub fn north_wall(&self) -> Wall {
        let as_point: Point = (*self).into();
        Wall {
            center: [as_point.x, as_point.y - CELL_HEIGHT as i32 / 2].into(),
            orientation: Orientation::Horizontal,
            border: self.y == 0,
        }
    }

    pub fn east_wall(&self) -> Wall {
        let as_point: Point = (*self).into();
        Wall {
            center: [as_point.x + CELL_WIDTH as i32 / 2, as_point.y].into(),
            orientation: Orientation::Vertical,
            border: self.x == (MAZE_WIDTH - 1) as i32,
        }
    }

    pub fn south_wall(&self) -> Wall {
        let as_point: Point = (*self).into();
        Wall {
            center: [as_point.x, as_point.y + CELL_HEIGHT as i32 / 2].into(),
            orientation: Orientation::Horizontal,
            border: self.y == (MAZE_HEIGHT - 1) as i32,
        }
    }

    pub fn west_wall(&self) -> Wall {
        let as_point: Point = (*self).into();
        Wall {
            center: [as_point.x - CELL_WIDTH as i32 / 2, as_point.y].into(),
            orientation: Orientation::Vertical,
            border: self.x == 0,
        }
    }

    pub fn valid_coord(&self) -> bool {
        self.x >= 0 && self.x <= (MAZE_WIDTH - 1) as i32 && self.y >= 0
            && self.y <= (MAZE_HEIGHT - 1) as i32
    }

    pub fn neighbour(&self, direction: Direction) -> Option<Coord> {
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

        if candidate.valid_coord() {
            Some(candidate)
        } else {
            None
        }
    }

    pub fn neighbours(&self) -> Vec<(Coord, Direction)> {
        vec![
            (self.neighbour(Direction::North), Direction::North),
            (self.neighbour(Direction::East), Direction::East),
            (self.neighbour(Direction::South), Direction::South),
            (self.neighbour(Direction::West), Direction::West),
        ].into_iter()
            .filter(|(coord, _)| coord.is_some())
            .map(|(c, d)| (c.unwrap(), d))
            .collect()
    }
}

impl Into<Point> for Coord {
    fn into(self) -> Point {
        Point {
            x: self.x * CELL_WIDTH as i32 + CELL_WIDTH as i32 / 2,
            y: self.y * CELL_HEIGHT as i32 + CELL_HEIGHT as i32 / 2,
        }
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
}

impl Wall {
    pub fn removable(&self) -> bool {
        !self.border
    }

    fn render(&self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::line::Line;

        let (start, end): (Point, Point) = match self.orientation {
            Orientation::Horizontal => (
                [self.center.x - CELL_WIDTH as i32 / 2, self.center.y].into(),
                [self.center.x + CELL_WIDTH as i32 / 2, self.center.y].into(),
            ),
            Orientation::Vertical => (
                [self.center.x, self.center.y - CELL_HEIGHT as i32 / 2].into(),
                [self.center.x, self.center.y + CELL_HEIGHT as i32 / 2].into(),
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
}

impl Cell {
    pub fn new(center: Point) -> Cell {
        Cell { center: center }
    }

    pub fn render(&self, color: Option<Color>, args: &RenderArgs, gl: &mut GlGraphics) {
        gl.draw(args.viewport(), |c, gl| {
            use graphics::rectangle::{centered, Rectangle};

            if let Some(color) = color {
                Rectangle::new(color).draw(
                    centered([
                        self.center.x as f64,
                        self.center.y as f64,
                        CELL_WIDTH as f64 / 2.0,
                        CELL_HEIGHT as f64 / 2.0,
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
pub struct Maze {
    pub walls: HashSet<Wall>,
    pub cells: HashMap<Coord, Cell>,
    pub start: Coord,
    pub end: Coord,
    pub explored: HashSet<Coord>,
    pub highlighted: HashSet<Coord>,
}

impl Maze {
    pub fn new() -> Maze {
        let mut walls = HashSet::new();
        let mut cells = HashMap::new();

        for y in 0..MAZE_HEIGHT {
            for x in 0..MAZE_WIDTH {
                let coord: Coord = [x, y].into();

                walls.insert(coord.north_wall());
                walls.insert(coord.east_wall());
                walls.insert(coord.south_wall());
                walls.insert(coord.west_wall());

                let mut cell = Cell::new(coord.into());

                cells.insert(coord, cell);
            }
        }

        let mut explored = HashSet::new();
        explored.insert([0, 0].into());

        Maze {
            walls: walls,
            cells: cells,
            start: [0, 0].into(),
            end: [MAZE_WIDTH - 1, MAZE_HEIGHT - 1].into(),
            explored: explored,
            highlighted: HashSet::new(),
        }
    }

    pub fn render(&self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::clear;

        clear(COLOR_BACKGROUND, gl);

        for (coord, cell) in &self.cells {
            let color;
            if self.highlighted.contains(&coord) {
                color = Some(COLOR_HIGHLIGHT);
            } else if *coord == self.start {
                color = Some(COLOR_START);
            } else if *coord == self.end {
                color = Some(COLOR_END);
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
        let result = match wall.orientation {
            Orientation::Horizontal => {
                let up: Coord = Point {
                    y: wall.center.y - CELL_HEIGHT as i32 / 2,
                    ..wall.center
                }.into();
                let down: Coord = Point {
                    y: wall.center.y + CELL_HEIGHT as i32 / 2,
                    ..wall.center
                }.into();

                (up, down)
            }
            Orientation::Vertical => {
                let left: Coord = Point {
                    x: wall.center.x - CELL_WIDTH as i32 / 2,
                    ..wall.center
                }.into();
                let right: Coord = Point {
                    x: wall.center.x + CELL_WIDTH as i32 / 2,
                    ..wall.center
                }.into();

                (left, right)
            }
        };

        result
    }

    pub fn link(&mut self, c1: &Coord, c2: &Coord) -> Result<()> {
        match c1.neighbours().iter().filter(|n| n.0 == *c2).next() {
            Some((_, direction)) => {
                let wall = match direction {
                    Direction::North => c1.north_wall(),
                    Direction::East => c1.east_wall(),
                    Direction::South => c1.south_wall(),
                    Direction::West => c1.west_wall(),
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
}
