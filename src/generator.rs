use std::collections::HashSet;
use std::str::FromStr;

use rand::{thread_rng as random, Rng};

use error::{Error, Result};
use maze::{Coord, Direction, Maze, Wall};

#[derive(Debug, Clone, Copy)]
pub enum GeneratorType {
    DFS,
    Kruskal,
    Prim,
}

impl GeneratorType {
    /// A list of possible variants in `&'static str` form
    pub fn variants() -> [&'static str; 3] {
        ["dfs", "kruskal", "prim"]
    }

    pub fn init(&self, maze: &Maze) -> Box<Generator> {
        match *self {
            GeneratorType::DFS => Box::new(DFS::new(maze)),
            GeneratorType::Kruskal => Box::new(Kruskal::new(maze)),
            GeneratorType::Prim => Box::new(Prim::new(maze)),
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
            .filter(|(c, _)| !maze.explored.contains(&c))
            .next()
    }
}

impl Generator for DFS {
    fn is_done(&self) -> bool {
        self.current.is_none()
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();

        let current = match self.current {
            Some(ref current) => current.clone(),
            None => return Ok(()),
        };
        maze.explored.insert(current);

        match self.available_neighbour(&maze) {
            Some((neighbour, _)) => {
                maze.link(&current, &neighbour)?;
                self.stack.push(current);
                self.current = Some(neighbour);
            }
            None => {
                maze.highlight_medium.insert(current);
                self.current = self.stack.pop();
            }
        }

        maze.highlight_bright.insert(current);
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
            .map(|w| w.clone())
            .collect::<Vec<_>>();

        random().shuffle(&mut walls);

        Kruskal {
            walls: walls,
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
            .ok_or(Error::MissingSet(c1))?;

        if c1_set.contains(&c2) {
            self.sets.push(c1_set);
            return Ok(JoinResult::Nop);
        }

        let c2_set = self.sets
            .drain_filter(|s| s.contains(&c2))
            .next()
            .ok_or(Error::MissingSet(c2))?;

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

        Prim { cells: cells }
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
            maze.explored.insert(cell);
            maze.highlight_medium.remove(&cell);
            maze.highlight_bright.insert(cell);
            self.cells.remove(&cell);

            let explored_neighbours: Vec<_> = maze.neighbours(&cell)
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
