use std::collections::VecDeque;
use std::str::FromStr;

use error::{Error, Result};
use maze::{Coord, Direction, Maze};

#[derive(Debug, Clone, Copy)]
pub enum SolverType {
    DFS,
    BFS,
    Dijkstra,
    AStar,
}

impl SolverType {
    /// A list of possible variants in `&'static str` form
    pub fn variants() -> [&'static str; 4] {
        ["dfs", "bfs", "dijkstra", "astar"]
    }

    pub fn init(&self, maze: &Maze) -> Box<Solver> {
        match *self {
            SolverType::DFS => Box::new(DFS::new(maze)),
            SolverType::BFS => Box::new(BFS::new(maze)),
            SolverType::Dijkstra => Box::new(Dijkstra::new(maze)),
            SolverType::AStar => Box::new(AStar::new(maze)),
        }
    }
}

impl FromStr for SolverType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "dfs" => Ok(SolverType::DFS),
            "bfs" => Ok(SolverType::BFS),
            "dijkstra" => Ok(SolverType::Dijkstra),
            "astar" => Ok(SolverType::AStar),
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
struct BFSNode {
    coord: Coord,
    previous: Option<Box<BFSNode>>,
}

pub struct BFS {
    current: BFSNode,
    goal: Coord,
    queue: VecDeque<BFSNode>,
}

impl BFS {
    fn new(maze: &Maze) -> BFS {
        BFS {
            current: BFSNode {
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

    fn highlight_path_from_cell(hc: &BFSNode, maze: &mut Maze) {
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
            self.queue.push_back(BFSNode {
                coord: neighbour,
                previous: Some(Box::new(self.current.clone())),
            });
            maze.highlight_dark.insert(neighbour);
        }

        self.current = self.queue.pop_front().ok_or(Error::ImpossibleMaze)?;
        maze.highlight_dark.remove(&self.current.coord);

        maze.explored.insert(self.current.coord);

        maze.highlight_bright.insert(self.current.coord);
        self.highlight_path(maze);

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct DijkstraNode {
    coord: Coord,
    dist: u32,
    previous: Option<Box<DijkstraNode>>,
}

#[derive(Debug)]
pub struct Dijkstra {
    current: DijkstraNode,
    goal: Coord,
    queue: Vec<DijkstraNode>,
}

impl Dijkstra {
    fn new(maze: &Maze) -> Dijkstra {
        Dijkstra {
            current: DijkstraNode {
                coord: maze.start,
                dist: 0,
                previous: None,
            },
            goal: maze.end,
            queue: vec![],
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

    fn highlight_path_from_cell(hc: &DijkstraNode, maze: &mut Maze) {
        maze.highlight_medium.insert(hc.coord);
        if let Some(ref previous) = hc.previous {
            Dijkstra::highlight_path_from_cell(&previous, maze);
        }
    }

    fn highlight_path(&self, maze: &mut Maze) {
        maze.highlight_medium.clear();
        Dijkstra::highlight_path_from_cell(&self.current, maze);
    }
}

impl Solver for Dijkstra {
    fn is_done(&self) -> bool {
        self.goal == self.current.coord
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();

        maze.explored.insert(self.current.coord);
        for (neighbour, _) in self.available_neighbours(maze).into_iter() {
            let dist_to_neighbour = self.current.dist + 1;

            let new_neighbour = DijkstraNode {
                coord: neighbour,
                dist: dist_to_neighbour,
                previous: Some(Box::new(self.current.clone())),
            };

            maze.highlight_dark.insert(neighbour);
            match self.queue
                .binary_search_by_key(&new_neighbour.dist, |n| n.dist)
            {
                Ok(pos) | Err(pos) => {
                    self.queue.insert(pos, new_neighbour);
                }
            }
        }

        if self.queue.is_empty() {
            return Err(Error::ImpossibleMaze);
        }
        self.current = self.queue.remove(0);
        maze.highlight_dark.remove(&self.current.coord);

        maze.highlight_bright.insert(self.current.coord);
        self.highlight_path(maze);

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct AStarNode {
    coord: Coord,
    dist: u32,
    score: u32,
    previous: Option<Box<AStarNode>>,
}

#[derive(Debug)]
pub struct AStar {
    current: AStarNode,
    goal: Coord,
    queue: Vec<AStarNode>,
}

impl AStar {
    fn new(maze: &Maze) -> AStar {
        AStar {
            current: AStarNode {
                coord: maze.start,
                dist: 0,
                score: 0,
                previous: None,
            },
            goal: maze.end,
            queue: vec![],
        }
    }

    fn heuristic(&self, coord: &Coord) -> u32 {
        self.goal.manhattan_dist(coord)
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

    fn highlight_path_from_cell(hc: &AStarNode, maze: &mut Maze) {
        maze.highlight_medium.insert(hc.coord);
        if let Some(ref previous) = hc.previous {
            AStar::highlight_path_from_cell(&previous, maze);
        }
    }

    fn highlight_path(&self, maze: &mut Maze) {
        maze.highlight_medium.clear();
        AStar::highlight_path_from_cell(&self.current, maze);
    }
}

impl Solver for AStar {
    fn is_done(&self) -> bool {
        self.goal == self.current.coord
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();

        maze.explored.insert(self.current.coord);
        for (neighbour, _) in self.available_neighbours(maze).into_iter() {
            let dist_to_neighbour = self.current.dist + 1;

            let new_neighbour = AStarNode {
                coord: neighbour,
                dist: dist_to_neighbour,
                score: dist_to_neighbour + self.heuristic(&neighbour),
                previous: Some(Box::new(self.current.clone())),
            };

            maze.highlight_dark.insert(neighbour);
            match self.queue
                .binary_search_by_key(&new_neighbour.score, |n| n.score)
            {
                Ok(pos) | Err(pos) => {
                    self.queue.insert(pos, new_neighbour);
                }
            }
        }

        if self.queue.is_empty() {
            return Err(Error::ImpossibleMaze);
        }
        self.current = self.queue.remove(0);
        maze.highlight_dark.remove(&self.current.coord);

        maze.highlight_bright.insert(self.current.coord);
        self.highlight_path(maze);

        Ok(())
    }
}
