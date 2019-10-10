use std::collections::VecDeque;
use std::rc::Rc;
use std::str::FromStr;

use error::{Error, Result};
use maze::{Coord, Direction, Maze};

#[derive(Debug, Clone, Copy)]
pub enum SolverType {
    DFS,
    BFS,
    Dijkstra,
    Greedy,
    AStar,
}

impl SolverType {
    /// A list of possible variants in `&'static str` form
    pub fn variants() -> [&'static str; 5] {
        ["dfs", "bfs", "dijkstra", "greedy", "astar"]
    }

    pub fn init(&self, maze: &Maze) -> Box<dyn Solver> {
        match *self {
            SolverType::DFS => Box::new(DFS::new(maze)),
            SolverType::BFS => Box::new(BFS::new(maze)),
            SolverType::Dijkstra => Box::new(Dijkstra::new(maze)),
            SolverType::Greedy => Box::new(Greedy::new(maze)),
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
            "greedy" => Ok(SolverType::Greedy),
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
            .find(|(c, _)| !self.stack.contains(&c))
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

struct BFSNode {
    coord: Coord,
    previous: Option<Rc<BFSNode>>,
}

pub struct BFS {
    current: Rc<BFSNode>,
    goal: Coord,
    queue: VecDeque<Rc<BFSNode>>,
}

impl BFS {
    fn new(maze: &Maze) -> BFS {
        BFS {
            current: Rc::new(BFSNode {
                coord: maze.start,
                previous: None,
            }),
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
            .filter(|(c, _)| self.queue.iter().find(|hc| hc.coord == *c).is_none())
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

impl<'a> Solver for BFS {
    fn is_done(&self) -> bool {
        self.goal == self.current.coord
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();

        for (neighbour, _) in self.available_neighbours(maze) {
            self.queue.push_back(Rc::new(BFSNode {
                coord: neighbour,
                previous: Some(self.current.clone()),
            }));
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

#[derive(Debug)]
struct DijkstraNode {
    coord: Coord,
    dist: u32,
    previous: Option<Rc<DijkstraNode>>,
}

#[derive(Debug)]
pub struct Dijkstra {
    current: Rc<DijkstraNode>,
    goal: Coord,
    queue: Vec<DijkstraNode>,
}

impl Dijkstra {
    fn new(maze: &Maze) -> Dijkstra {
        Dijkstra {
            current: Rc::new(DijkstraNode {
                coord: maze.start,
                dist: 0,
                previous: None,
            }),
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
            .filter(|(c, _)| self.queue.iter().find(|hc| hc.coord == *c).is_none())
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
        for (neighbour, _) in self.available_neighbours(maze) {
            let dist_to_neighbour = self.current.dist + 1;

            let new_neighbour = DijkstraNode {
                coord: neighbour,
                dist: dist_to_neighbour,
                previous: Some(self.current.clone()),
            };

            maze.highlight_dark.insert(neighbour);
            match self
                .queue
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
        self.current = Rc::new(self.queue.remove(0));
        maze.highlight_dark.remove(&self.current.coord);

        maze.highlight_bright.insert(self.current.coord);
        self.highlight_path(maze);

        Ok(())
    }
}

#[derive(Debug)]
struct GreedyNode {
    coord: Coord,
    score: u32,
    previous: Option<Rc<GreedyNode>>,
}

#[derive(Debug)]
pub struct Greedy {
    current: Rc<GreedyNode>,
    goal: Coord,
    queue: Vec<Rc<GreedyNode>>,
}

impl Greedy {
    fn new(maze: &Maze) -> Greedy {
        Greedy {
            current: Rc::new(GreedyNode {
                coord: maze.start,
                score: 0,
                previous: None,
            }),
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
            .filter(|(c, _)| self.queue.iter().find(|hc| hc.coord == *c).is_none())
            .collect()
    }

    fn highlight_path_from_cell(hc: &GreedyNode, maze: &mut Maze) {
        maze.highlight_medium.insert(hc.coord);
        if let Some(ref previous) = hc.previous {
            Greedy::highlight_path_from_cell(&previous, maze);
        }
    }

    fn highlight_path(&self, maze: &mut Maze) {
        maze.highlight_medium.clear();
        Greedy::highlight_path_from_cell(&self.current, maze);
    }
}

impl Solver for Greedy {
    fn is_done(&self) -> bool {
        self.goal == self.current.coord
    }

    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        maze.highlight_bright.clear();

        maze.explored.insert(self.current.coord);
        for (neighbour, _) in self.available_neighbours(maze) {
            let new_neighbour = GreedyNode {
                coord: neighbour,
                score: self.heuristic(&neighbour),
                previous: Some(self.current.clone()),
            };

            maze.highlight_dark.insert(neighbour);
            match self
                .queue
                .binary_search_by_key(&new_neighbour.score, |n| n.score)
            {
                Ok(pos) | Err(pos) => {
                    self.queue.insert(pos, Rc::new(new_neighbour));
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

#[derive(Debug)]
struct AStarNode {
    coord: Coord,
    dist: u32,
    score: u32,
    previous: Option<Rc<AStarNode>>,
}

#[derive(Debug)]
pub struct AStar {
    current: Rc<AStarNode>,
    goal: Coord,
    queue: Vec<Rc<AStarNode>>,
}

impl AStar {
    fn new(maze: &Maze) -> AStar {
        AStar {
            current: Rc::new(AStarNode {
                coord: maze.start,
                dist: 0,
                score: 0,
                previous: None,
            }),
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
            .filter(|(c, _)| self.queue.iter().find(|hc| hc.coord == *c).is_none())
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
        for (neighbour, _) in self.available_neighbours(maze) {
            let dist_to_neighbour = self.current.dist + 1;

            let new_neighbour = AStarNode {
                coord: neighbour,
                dist: dist_to_neighbour,
                score: dist_to_neighbour + self.heuristic(&neighbour),
                previous: Some(self.current.clone()),
            };

            maze.highlight_dark.insert(neighbour);
            match self
                .queue
                .binary_search_by_key(&new_neighbour.score, |n| n.score)
            {
                Ok(pos) | Err(pos) => {
                    self.queue.insert(pos, Rc::new(new_neighbour));
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
