#![feature(option_filter)]

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use rand::{thread_rng as random, Rng};

use graphics::types::Color;

const COLOR_BACKGROUND: Color = [7.0 / 255.0, 16.0 / 255.0, 19.0 / 255.0, 1.0];
const COLOR_EXPLORED: Color = [14.0 / 255.0, 71.0 / 255.0, 73.0 / 255.0, 1.0];
const COLOR_START: Color = [149.0 / 255.0, 198.0 / 255.0, 35.0 / 255.0, 1.0];
const COLOR_END: Color = [229.0 / 255.0, 88.0 / 255.0, 18.0 / 255.0, 1.0];
const COLOR_WALL: Color = [239.0 / 255.0, 231.0 / 255.0, 218.0 / 255.0, 1.0];
const COLOR_HIGHLIGHT: Color = [57.0 / 255.0, 104.0 / 255.0, 106.0 / 255.0, 1.0];

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

const MAZE_WIDTH: u32 = WINDOW_WIDTH / 40;
const MAZE_HEIGHT: u32 = WINDOW_HEIGHT / 40;

const CELL_WIDTH: f64 = WINDOW_WIDTH as f64 / MAZE_WIDTH as f64;
const CELL_HEIGHT: f64 = WINDOW_HEIGHT as f64 / MAZE_HEIGHT as f64;
const CELL_WALL_WIDTH: f64 = 1.5;

const UPS: u64 = 60;

enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, PartialEq)]
struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    fn flat_idx(&self) -> Option<usize> {
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

    fn neighbour(&self, direction: Direction) -> Coord {
        match direction {
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
        }
    }

    fn neighbours(&self) -> [Coord; 4] {
        [
            self.neighbour(Direction::North),
            self.neighbour(Direction::East),
            self.neighbour(Direction::South),
            self.neighbour(Direction::West),
        ]
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

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl From<[f64; 2]> for Point {
    fn from(a: [f64; 2]) -> Point {
        Point { x: a[0], y: a[1] }
    }
}

#[derive(Clone)]
struct Wall {
    start: Point,
    end: Point,
}

impl Wall {
    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) {
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

#[derive(Clone)]
struct Cell {
    coord: Coord,
    center: Point,

    start_cell: bool,
    end_cell: bool,
    visited: bool,

    north: Option<Wall>,
    east: Option<Wall>,
    south: Option<Wall>,
    west: Option<Wall>,
}

impl Cell {
    fn new(coord: Coord) -> Cell {
        Cell {
            center: coord.into_point(CELL_WIDTH, CELL_HEIGHT),
            coord: coord,

            start_cell: false,
            end_cell: false,
            visited: false,

            north: None,
            east: None,
            south: None,
            west: None,
        }
    }

    fn start_cell(&mut self, set: bool) -> &Cell {
        self.start_cell = set;
        self
    }

    fn end_cell(&mut self, set: bool) -> &Cell {
        self.end_cell = set;
        self
    }

    fn north(&mut self, set: bool) -> &Cell {
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

    fn east(&mut self, set: bool) -> &Cell {
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

    fn south(&mut self, set: bool) -> &Cell {
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

    fn west(&mut self, set: bool) -> &Cell {
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

    fn render(&mut self, highlight: bool, args: &RenderArgs, gl: &mut GlGraphics) {
        gl.draw(args.viewport(), |c, gl| {
            use graphics::rectangle::{centered, Rectangle};

            let color;
            if highlight {
                color = Some(COLOR_HIGHLIGHT)
            } else if self.start_cell {
                color = Some(COLOR_START);
            } else if self.end_cell {
                color = Some(COLOR_END);
            } else if self.visited {
                color = Some(COLOR_EXPLORED);
            } else {
                color = None;
            }

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

        if let Some(ref mut wall) = self.north {
            wall.render(args, gl);
        }
        if let Some(ref mut wall) = self.east {
            wall.render(args, gl);
        }
        if let Some(ref mut wall) = self.south {
            wall.render(args, gl);
        }
        if let Some(ref mut wall) = self.west {
            wall.render(args, gl);
        }
    }
}

struct App {
    maze: Vec<Cell>,
    current: Option<Coord>,
    stack: Vec<Coord>,
}

impl App {
    fn new() -> App {
        let mut maze = Vec::new();
        for y in 0..MAZE_HEIGHT {
            for x in 0..MAZE_WIDTH {
                let mut cell = Cell::new([x, y].into());

                cell.start_cell(x == 0 && y == 0);
                cell.end_cell(x == MAZE_WIDTH - 1 && y == MAZE_HEIGHT - 1);
                cell.north(true);
                cell.south(true);
                cell.west(true);
                cell.east(true);

                maze.push(cell);
            }
        }

        maze[0].visited = true;

        App {
            current: Some([0, 0].into()),
            maze: maze,
            stack: Vec::new(),
        }
    }

    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::clear;

        clear(COLOR_BACKGROUND, gl);

        for cell in self.maze.iter_mut() {
            let highlight = match self.current {
                Some(ref c) => cell.coord == *c,
                None => false,
            };
            cell.render(highlight, args, gl);
        }
    }

    fn update(&mut self, _args: &UpdateArgs) {
        let current = match self.current {
            Some(ref current) => current.clone(),
            None => return,
        };

        match self.available_neighbour() {
            Some(neighbour) => {
                let mut current_cell = self.cell_at(&current);
                let mut neighbour_cell = self.cell_at(&neighbour);

                neighbour_cell.visited = true;

                if neighbour == current.neighbour(Direction::North) {
                    current_cell.north(false);
                    neighbour_cell.south(false);
                } else if neighbour == current.neighbour(Direction::East) {
                    current_cell.east(false);
                    neighbour_cell.west(false);
                } else if neighbour == current.neighbour(Direction::South) {
                    current_cell.south(false);
                    neighbour_cell.north(false);
                } else if neighbour == current.neighbour(Direction::West) {
                    current_cell.west(false);
                    neighbour_cell.east(false);
                }

                self.update_cell(current_cell);
                self.update_cell(neighbour_cell);

                self.stack.push(current);
                self.current = Some(neighbour);
            }
            None => self.current = self.stack.pop(),
        }
    }

    fn available_neighbour(&self) -> Option<Coord> {
        let current = match self.current {
            Some(ref current) => current,
            None => return None,
        };

        let mut neighbours = current.neighbours();
        random().shuffle(&mut neighbours);

        neighbours
            .iter()
            .filter(|c| {
                c.flat_idx()
                    .and_then(|i| self.maze.get(i))
                    .filter(|c| !c.visited)
                    .is_some()
            })
            .map(|c| c.clone())
            .next()
    }

    fn cell_at(&self, coord: &Coord) -> Cell {
        self.maze[coord.flat_idx().expect("Invalid coord")].clone()
    }

    fn update_cell(&mut self, cell: Cell) {
        let idx = cell.coord.flat_idx().expect("Invalid cell");
        self.maze[idx] = cell;
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window =
        WindowSettings::new("Space filling circles", [WINDOW_WIDTH, WINDOW_HEIGHT])
            .opengl(opengl)
            .exit_on_esc(false)
            .build()
            .expect("Error creating window");

    let mut gl = GlGraphics::new(opengl);
    let mut app = App::new();

    let mut event_settings = EventSettings::new();
    event_settings.ups = UPS;
    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r, &mut gl);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}
