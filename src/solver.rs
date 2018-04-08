use std::collections::VecDeque;
use std::str::FromStr;

use error::{Error, Result};
use maze::{Coord, Direction, Maze};

#[derive(Debug, Clone, Copy)]
pub enum SolverType {
    DFS,
    BFS,
}

impl SolverType {
    /// A list of possible variants in `&'static str` form
    pub fn variants() -> [&'static str; 2] {
        ["dfs", "bfs"]
    }

    pub fn init(&self, maze: &Maze) -> Box<Solver> {
        match *self {
            SolverType::DFS => Box::new(DFS::new(maze)),
            SolverType::BFS => Box::new(BFS::new(maze)),
        }
    }
}

impl FromStr for SolverType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "dfs" => Ok(SolverType::DFS),
            "bfs" => Ok(SolverType::BFS),
            _ => Err(Error::UnsupportedSolver(s.to_string())),
        }
    }
}

pub trait Solver {
    fn is_done(&self) -> bool;
    fn tick(&mut self, maze: &mut Maze) -> Result<()>;
}

pub struct DFS {
    current: Coord,
    goal: Coord,
    stack: Vec<Coord>,
}

impl DFS {
    fn new(maze: &Maze) -> DFS {
        DFS {
            current: maze.start,
            goal: maze.end,
            stack: vec![],
        }
    }

    fn available_neighbour(&self, maze: &Maze) -> Option<(Coord, Direction)> {
        if maze.end == self.current {
            return None;
        }

        maze.connected_neighbours(&self.current)
            .into_iter()
            .filter(|(c, _)| !maze.explored.contains(&c))
            .filter(|(c, _)| !self.stack.contains(&c))
            .next()
    }
}

impl Solver for DFS {
    fn is_done(&self) -> bool {
        self.goal == self.current
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();

        match self.available_neighbour(&maze) {
            Some((neighbour, _)) => {
                maze.explored.insert(neighbour);

                maze.highlight_medium.insert(neighbour);

                self.stack.push(self.current);
                self.current = neighbour;
            }
            None => {
                maze.highlight_medium.remove(&self.current);
                self.current = self.stack.pop().ok_or(Error::ImpossibleMaze)?;
            }
        }

        maze.highlight_bright.insert(self.current);

        Ok(())
    }
}

#[derive(Clone)]
struct HierarchicalCoord {
    coord: Coord,
    previous: Option<Box<HierarchicalCoord>>,
}

pub struct BFS {
    current: HierarchicalCoord,
    goal: Coord,
    queue: VecDeque<HierarchicalCoord>,
}

impl BFS {
    fn new(maze: &Maze) -> BFS {
        BFS {
            current: HierarchicalCoord {
                coord: maze.start,
                previous: None,
            },
            goal: maze.end,
            queue: VecDeque::new(),
        }
    }

    fn available_neighbours(&self, maze: &Maze) -> Vec<(Coord, Direction)> {
        if maze.end == self.current.coord {
            return Vec::new();
        }

        maze.connected_neighbours(&self.current.coord)
            .into_iter()
            .filter(|(c, _)| !maze.explored.contains(&c))
            .filter(|(c, _)| {
                !self.queue
                    .iter()
                    .filter(|hc| hc.coord == *c)
                    .next()
                    .is_some()
            })
            .collect()
    }

    fn highlight_path_from_cell(hc: &HierarchicalCoord, maze: &mut Maze) {
        maze.highlight_medium.insert(hc.coord);
        if let Some(ref previous) = hc.previous {
            BFS::highlight_path_from_cell(&previous, maze);
        }
    }

    fn highlight_path(&self, maze: &mut Maze) {
        maze.highlight_medium.clear();
        BFS::highlight_path_from_cell(&self.current, maze);
    }
}

impl Solver for BFS {
    fn is_done(&self) -> bool {
        self.goal == self.current.coord
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();

        for (neighbour, _) in self.available_neighbours(maze).into_iter() {
            self.queue.push_back(HierarchicalCoord {
                coord: neighbour,
                previous: Some(Box::new(self.current.clone())),
            });
        }

        self.current = self.queue.pop_front().ok_or(Error::ImpossibleMaze)?;

        maze.explored.insert(self.current.coord);

        maze.highlight_bright.insert(self.current.coord);
        self.highlight_path(maze);

        Ok(())
    }
}
