use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use rand::{thread_rng as random, Rng};

use error::{Error, Result};
use maze::{Coord, Direction, Maze, Wall};

#[derive(Debug, Clone, Copy)]
pub enum GeneratorType {
    DFS,
    Kruskal,
    Prim,
    Eller,
}

impl GeneratorType {
    /// A list of possible variants in `&'static str` form
    pub fn variants() -> [&'static str; 4] {
        ["dfs", "kruskal", "prim", "eller"]
    }

    pub fn init(&self, maze: &Maze) -> Box<Generator> {
        match *self {
            GeneratorType::DFS => Box::new(DFS::new(maze)),
            GeneratorType::Kruskal => Box::new(Kruskal::new(maze)),
            GeneratorType::Prim => Box::new(Prim::new(maze)),
            GeneratorType::Eller => Box::new(Eller::new(maze)),
        }
    }
}

impl FromStr for GeneratorType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "dfs" => Ok(GeneratorType::DFS),
            "kruskal" => Ok(GeneratorType::Kruskal),
            "prim" => Ok(GeneratorType::Prim),
            "eller" => Ok(GeneratorType::Eller),
            _ => Err(Error::UnsupportedGenerator(s.to_string())),
        }
    }
}

pub trait Generator {
    fn is_done(&self) -> bool;
    fn tick(&mut self, maze: &mut Maze) -> Result<()>;
}

pub struct DFS {
    pub current: Option<Coord>,
    pub stack: Vec<Coord>,
}

impl DFS {
    fn new(maze: &Maze) -> DFS {
        DFS {
            current: Some(maze.start),
            stack: vec![],
        }
    }

    fn available_neighbour(&self, maze: &Maze) -> Option<(Coord, Direction)> {
        let current = match self.current {
            Some(ref current) => current,
            None => return None,
        };

        if maze.end == *current {
            return None;
        }

        let mut neighbours = maze.neighbours(current);
        random().shuffle(&mut neighbours);

        neighbours
            .into_iter()
            .find(|(c, _)| !maze.explored.contains(&c))
    }
}

impl Generator for DFS {
    fn is_done(&self) -> bool {
        self.current.is_none()
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();

        let current = match self.current {
            Some(ref current) => *current,
            None => return Ok(()),
        };

        maze.highlight_bright.insert(current);
        maze.highlight_medium.insert(current);
        maze.explored.insert(current);

        match self.available_neighbour(&maze) {
            Some((neighbour, _)) => {
                maze.link(&current, &neighbour)?;
                self.stack.push(current);
                self.current = Some(neighbour);
            }
            None => {
                maze.highlight_medium.remove(&current);
                self.current = self.stack.pop();
            }
        }

        Ok(())
    }
}

enum JoinResult {
    Joined,
    Nop,
}

pub struct Kruskal {
    walls: Vec<Wall>,
    sets: Vec<HashSet<Coord>>,
}

impl Kruskal {
    pub fn new(maze: &Maze) -> Kruskal {
        let mut walls = maze.walls
            .iter()
            .filter(|w| w.removable())
            .cloned()
            .collect::<Vec<_>>();

        random().shuffle(&mut walls);

        Kruskal {
            walls,
            sets: maze.cells
                .keys()
                .map(|c| {
                    let mut set = HashSet::new();
                    set.insert(*c);

                    set
                })
                .collect(),
        }
    }

    fn join(&mut self, c1: Coord, c2: Coord) -> Result<JoinResult> {
        let mut c1_set = self.sets
            .drain_filter(|s| s.contains(&c1))
            .next()
            .ok_or_else(|| Error::MissingSet(c1))?;

        if c1_set.contains(&c2) {
            self.sets.push(c1_set);
            return Ok(JoinResult::Nop);
        }

        let c2_set = self.sets
            .drain_filter(|s| s.contains(&c2))
            .next()
            .ok_or_else(|| Error::MissingSet(c2))?;

        for c in c2_set {
            c1_set.insert(c);
        }

        self.sets.push(c1_set);

        Ok(JoinResult::Joined)
    }
}

impl Generator for Kruskal {
    fn is_done(&self) -> bool {
        self.walls.is_empty() || self.sets.len() <= 1
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();
        if self.is_done() {
            return Ok(());
        }

        if let Some(wall) = self.walls.pop() {
            let (c1, c2) = maze.divided_coords(&wall);

            maze.explored.insert(c1);
            maze.explored.insert(c2);
            maze.highlight_bright.insert(c1);
            maze.highlight_bright.insert(c2);

            match self.join(c1, c2) {
                Err(e) => return Err(e),
                Ok(JoinResult::Joined) => {
                    maze.walls.remove(&wall);
                }
                Ok(JoinResult::Nop) => {}
            };
        }

        Ok(())
    }
}

pub struct Prim {
    cells: HashSet<Coord>,
}

impl Prim {
    pub fn new(maze: &Maze) -> Prim {
        let mut cells = HashSet::new();
        cells.insert(maze.start);

        Prim { cells }
    }

    fn random_cell(&mut self) -> Option<Coord> {
        if self.cells.is_empty() {
            return None;
        }
        let cell_list: Vec<_> = self.cells.iter().collect();
        // Unwrap is safe here because of the is_empty check above
        let cell = random().choose(&cell_list).unwrap();

        Some(**cell)
    }
}

impl Generator for Prim {
    fn is_done(&self) -> bool {
        self.cells.is_empty()
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();

        if let Some(cell) = self.random_cell() {
            if (cell == maze.start || cell == maze.end) && maze.explored.contains(&cell) {
                return Ok(());
            }

            maze.explored.insert(cell);
            maze.highlight_medium.remove(&cell);
            maze.highlight_bright.insert(cell);
            self.cells.remove(&cell);

            let mut explored_neighbours: Vec<_> = maze.neighbours(&cell)
                .into_iter()
                .filter(|(n, _)| maze.explored.contains(n))
                .collect();

            let unknown_neighbours: Vec<_> = maze.neighbours(&cell)
                .into_iter()
                .filter(|(n, _)| !maze.explored.contains(n))
                .collect();

            if let Some((_, direction)) = random().choose(&explored_neighbours) {
                let wall = match direction {
                    Direction::North => maze.north_wall(&cell),
                    Direction::East => maze.east_wall(&cell),
                    Direction::South => maze.south_wall(&cell),
                    Direction::West => maze.west_wall(&cell),
                };
                maze.walls.remove(&wall);
            }

            for (unknown_neighbour, _) in unknown_neighbours {
                maze.highlight_medium.insert(unknown_neighbour);
                self.cells.insert(unknown_neighbour);
            }
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq)]
enum EllerJoiningMode {
    Horizontal,
    Vertical,
}

pub struct Eller {
    current: Coord,
    last_row: i32,
    mode: EllerJoiningMode,
    coord_to_set: HashMap<Coord, usize>,
    set_to_coords: HashMap<usize, Vec<Coord>>,
    last_set: usize,
}

impl Eller {
    pub fn new(maze: &Maze) -> Eller {
        let current = [0, 0].into();
        let mut coord_to_set = HashMap::new();
        let mut set_to_coords = HashMap::new();

        set_to_coords.insert(0, vec![current]);
        coord_to_set.insert(current, 0);

        Eller {
            current,
            last_row: maze.maze_height() as i32 - 1,
            mode: EllerJoiningMode::Horizontal,
            coord_to_set,
            set_to_coords,
            last_set: 0,
        }
    }

    fn same_set(&mut self, c1: &Coord, c2: &Coord) -> bool {
        if let Some(c1_set_idx) = self.coord_to_set.get(c1) {
            if let Some(c2_set_idx) = self.coord_to_set.get(c2) {
                return c1_set_idx == c2_set_idx;
            }
        }

        false
    }

    fn new_set(&mut self, c: Coord) {
        let next_set = self.last_set + 1;
        self.add(c, next_set);
        self.last_set = next_set;
    }

    fn add(&mut self, c: Coord, set: usize) {
        if self.coord_to_set.contains_key(&c) {
            let current_set = self.coord_to_set[&c];

            if current_set == set {
                return;
            }

            self.coord_to_set.remove(&c);
            self.set_to_coords
                .get_mut(&current_set)
                .unwrap()
                .remove_item(&c);
            self.add(c, set);
            if self.set_to_coords[&current_set].is_empty() {
                self.set_to_coords.remove(&current_set);
            }
        } else {
            self.coord_to_set.insert(c, set);
            if self.set_to_coords.contains_key(&set) {
                self.set_to_coords.get_mut(&set).unwrap().push(c);
            } else {
                self.set_to_coords.insert(set, vec![c]);
            }
        }
    }

    fn join(&mut self, c1: &Coord, c2: Coord) {
        if self.same_set(c1, &c2) {
            unreachable!()
        }
        let c1_set_idx = self.coord_to_set[c1];

        if !self.coord_to_set.contains_key(&c2) {
            self.add(c2, c1_set_idx);
        } else {
            let c2_set_idx = self.coord_to_set[&c2];
            let c2_set: Vec<Coord> = self.set_to_coords[&c2_set_idx].to_vec();

            for &cell in &c2_set {
                self.add(cell, c1_set_idx);
            }
        }
    }

    fn connected_vertically(&self, set: usize, maze: &Maze) -> bool {
        self.set_to_coords[&set]
            .iter()
            .filter(|c| c.y == self.current.y)
            .any(|c| {
                let wall = maze.south_wall(c);
                !maze.walls.contains(&wall)
            })
    }
}

impl Generator for Eller {
    fn is_done(&self) -> bool {
        self.mode == EllerJoiningMode::Vertical && self.current.y == self.last_row
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();
        maze.highlight_medium.clear();
        maze.highlight_dark.clear();

        for x in 0..maze.maze_width() as i32 {
            maze.highlight_dark.insert([x, self.current.y].into());
        }

        maze.highlight_bright.insert(self.current);
        maze.explored.insert(self.current);

        match self.mode {
            EllerJoiningMode::Horizontal => {
                let current = self.current;
                let last_row = current.y == self.last_row;
                if let Some(neighbour) = maze.neighbour(&current, &Direction::East) {
                    if !self.same_set(&current, &neighbour) && (last_row || random().gen()) {
                        self.join(&current, neighbour);

                        let wall = maze.east_wall(&current);
                        maze.walls.remove(&wall);

                        maze.highlight_medium.insert(neighbour);
                    } else if !self.coord_to_set.contains_key(&neighbour) {
                        self.new_set(neighbour);
                    }
                    self.current = neighbour;
                } else {
                    self.mode = EllerJoiningMode::Vertical;
                }
            }
            EllerJoiningMode::Vertical => {
                let current_set = self.coord_to_set[&self.current];
                let last_in_set = maze.neighbour(&self.current, &Direction::West)
                    .filter(|c| self.coord_to_set[&c] == current_set)
                    .is_none();
                let connected = self.connected_vertically(current_set, maze);
                let force_join = last_in_set && !connected;

                let current = self.current;
                if force_join || random().gen() {
                    if let Some(neighbour) = maze.neighbour(&current, &Direction::South) {
                        self.join(&current, neighbour);

                        let wall = maze.south_wall(&current);
                        maze.walls.remove(&wall);

                        maze.explored.insert(neighbour);
                        maze.highlight_medium.insert(neighbour);
                    }
                }

                if let Some(neighbour) = maze.neighbour(&self.current, &Direction::West) {
                    self.current = neighbour;
                } else {
                    self.mode = EllerJoiningMode::Horizontal;
                    if let Some(neighbour) = maze.neighbour(&self.current, &Direction::South) {
                        if self.coord_to_set.get(&neighbour).is_none() {
                            self.new_set(neighbour);
                        }
                        self.current = neighbour;
                    }
                }
            }
        }

        Ok(())
    }
}
