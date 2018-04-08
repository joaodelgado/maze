use std::collections::HashSet;
use std::str::FromStr;

use rand::{thread_rng as random, Rng};

use errors::{Error, Result};
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
    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        let current = match self.current {
            Some(ref current) => current.clone(),
            None => return Ok(()),
        };

        match self.available_neighbour(&maze) {
            Some((neighbour, _)) => {
                maze.explored.insert(neighbour);
                maze.link(&current, &neighbour)?;
                self.stack.push(current);
                self.current = Some(neighbour);
            }
            None => self.current = self.stack.pop(),
        }

        maze.highlighted.clear();
        if let Some(current) = self.current {
            maze.highlighted.insert(current);
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
    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlighted.clear();
        if self.walls.is_empty() {
            return Ok(());
        }

        if let Some(wall) = self.walls.pop() {
            let (c1, c2) = maze.divided_coords(&wall);

            maze.explored.insert(c1);
            maze.explored.insert(c2);
            maze.highlighted.insert(c1);
            maze.highlighted.insert(c2);

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
    walls: HashSet<Wall>,
}

impl Prim {
    pub fn new(maze: &Maze) -> Prim {
        let mut prim = Prim {
            walls: HashSet::new(),
        };

        prim.extend_walls(&maze.start, maze);

        prim
    }

    fn extend_walls(&mut self, c: &Coord, maze: &Maze) {
        for wall in maze.walls(c)
            .iter()
            .filter(|w| maze.walls.contains(w))
            .filter(|w| w.removable())
        {
            self.walls.insert(wall.clone());
        }
    }

    fn random_wall(&self) -> Option<Wall> {
        if self.walls.is_empty() {
            return None;
        }
        let wall_list: Vec<_> = self.walls.iter().collect();
        // Unwrap is safe here because of the is_empty check above
        let wall = random().choose(&wall_list).unwrap();

        Some(*wall.clone())
    }
}

impl Generator for Prim {
    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlighted.clear();

        // Unwrap is safe here because of the is_empty check above
        if let Some(wall) = self.random_wall() {
            let (c1, c2) = maze.divided_coords(&wall);
            maze.highlighted.insert(c1);
            maze.highlighted.insert(c2);

            if !maze.explored.contains(&c1) {
                maze.explored.insert(c1);
                self.extend_walls(&c1, &maze);
                maze.walls.remove(&wall);
            } else if !maze.explored.contains(&c2) {
                maze.explored.insert(c2);
                self.extend_walls(&c2, &maze);
                maze.walls.remove(&wall);
            }
            self.walls.remove(&wall);
        }

        Ok(())
    }
}
