use graphics::types::Color;
use opengl_graphics::GlGraphics;
use piston::input::RenderArgs;

use config::*;

pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    pub fn flat_idx(&self) -> Option<usize> {
        let width = MAZE_WIDTH as i32;
        let height = MAZE_HEIGHT as i32;
        let result = match *self {
            _ if self.x < 0 || self.x >= width || self.y < 0 || self.y >= height => None,
            _ => Some((self.x + self.y * width) as usize),
        };

        result
    }

    fn into_point(&self, width: f64, height: f64) -> Point {
        [
            self.x as f64 * width + width / 2.0,
            self.y as f64 * height + height / 2.0,
        ].into()
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

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl From<[f64; 2]> for Point {
    fn from(a: [f64; 2]) -> Point {
        Point { x: a[0], y: a[1] }
    }
}

#[derive(Debug, Clone)]
struct Wall {
    start: Point,
    end: Point,
}

impl Wall {
    fn render(&self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::line::Line;

        gl.draw(args.viewport(), |c, gl| {
            Line::new(COLOR_WALL, CELL_WALL_WIDTH / 2.0).draw(
                [self.start.x, self.start.y, self.end.x, self.end.y],
                &c.draw_state,
                c.transform,
                gl,
            );
        });
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub coord: Coord,
    center: Point,

    north: Option<Wall>,
    east: Option<Wall>,
    south: Option<Wall>,
    west: Option<Wall>,
}

impl Cell {
    pub fn new(coord: Coord) -> Cell {
        Cell {
            center: coord.into_point(CELL_WIDTH, CELL_HEIGHT),
            coord: coord,

            north: None,
            east: None,
            south: None,
            west: None,
        }
    }

    pub fn north(&mut self, set: bool) -> &Cell {
        if set {
            self.north = Some(Wall {
                start: self.north_east(),
                end: self.north_west(),
            });
        } else {
            self.north = None;
        }
        self
    }

    pub fn east(&mut self, set: bool) -> &Cell {
        if set {
            self.east = Some(Wall {
                start: self.north_east(),
                end: self.south_east(),
            });
        } else {
            self.east = None;
        }
        self
    }

    pub fn south(&mut self, set: bool) -> &Cell {
        if set {
            self.south = Some(Wall {
                start: self.south_west(),
                end: self.south_east(),
            });
        } else {
            self.south = None;
        }
        self
    }

    pub fn west(&mut self, set: bool) -> &Cell {
        if set {
            self.west = Some(Wall {
                start: self.north_west(),
                end: self.south_west(),
            });
        } else {
            self.west = None;
        }
        self
    }

    fn north_west(&self) -> Point {
        [
            self.center.x - CELL_WIDTH / 2.0,
            self.center.y - CELL_HEIGHT / 2.0,
        ].into()
    }

    fn north_east(&self) -> Point {
        [
            self.center.x + CELL_WIDTH / 2.0,
            self.center.y - CELL_HEIGHT / 2.0,
        ].into()
    }

    fn south_west(&self) -> Point {
        [
            self.center.x - CELL_WIDTH / 2.0,
            self.center.y + CELL_HEIGHT / 2.0,
        ].into()
    }

    fn south_east(&self) -> Point {
        [
            self.center.x + CELL_WIDTH / 2.0,
            self.center.y + CELL_HEIGHT / 2.0,
        ].into()
    }

    pub fn render(&self, color: Option<Color>, args: &RenderArgs, gl: &mut GlGraphics) {
        gl.draw(args.viewport(), |c, gl| {
            use graphics::rectangle::{centered, Rectangle};

            if let Some(color) = color {
                Rectangle::new(color).draw(
                    centered([
                        self.center.x,
                        self.center.y,
                        CELL_WIDTH / 2.0,
                        CELL_HEIGHT / 2.0,
                    ]),
                    &c.draw_state,
                    c.transform,
                    gl,
                );
            }
        });

        if let Some(ref wall) = self.north {
            wall.render(args, gl);
        }
        if let Some(ref wall) = self.east {
            wall.render(args, gl);
        }
        if let Some(ref wall) = self.south {
            wall.render(args, gl);
        }
        if let Some(ref wall) = self.west {
            wall.render(args, gl);
        }
    }
}

#[derive(Debug)]
pub struct Maze {
    pub cells: Vec<Cell>,
    pub start: Coord,
    pub end: Coord,
    pub explored: Vec<Coord>,
    pub highlighted: Vec<Coord>,
}

impl Maze {
    pub fn new() -> Maze {
        let mut cells = Vec::new();
        for y in 0..MAZE_HEIGHT {
            for x in 0..MAZE_WIDTH {
                let mut cell = Cell::new([x, y].into());

                cell.north(true);
                cell.south(true);
                cell.west(true);
                cell.east(true);

                cells.push(cell);
            }
        }

        Maze {
            cells: cells,
            start: [0, 0].into(),
            end: [MAZE_WIDTH - 1, MAZE_HEIGHT - 1].into(),
            explored: vec![[0, 0].into()],
            highlighted: Vec::new(),
        }
    }

    pub fn render(&self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::clear;

        clear(COLOR_BACKGROUND, gl);

        for cell in self.cells.iter() {
            let color;
            if self.highlighted.contains(&cell.coord) {
                color = Some(COLOR_HIGHLIGHT);
            } else if cell.coord == self.start {
                color = Some(COLOR_START);
            } else if cell.coord == self.end {
                color = Some(COLOR_END);
            } else if self.explored.contains(&cell.coord) {
                color = Some(COLOR_EXPLORED);
            } else {
                color = None;
            }

            cell.render(color, args, gl);
        }
    }

    pub fn cell_at(&self, coord: &Coord) -> Cell {
        self.cells[coord.flat_idx().expect("Invalid coord")].clone()
    }

    pub fn update_cell(&mut self, cell: Cell) {
        let idx = cell.coord.flat_idx().expect("Invalid cell");
        self.cells[idx] = cell;
    }
}
