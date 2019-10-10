use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use rand::{Rng, StdRng};

use error::{Error, Result};
use maze::{Coord, Direction, Maze, Wall};

#[derive(Debug, Clone, Copy)]
pub enum GeneratorType {
    DFS,
    Kruskal,
    Prim,
    Eller,
    HuntKill,
}

impl GeneratorType {
    /// A list of possible variants in `&'static str` form
    pub fn variants() -> [&'static str; 5] {
        ["dfs", "kruskal", "prim", "eller", "hunt-kill"]
    }

    pub fn init(&self, maze: &Maze, random: &mut StdRng) -> Box<dyn Generator> {
        match *self {
            GeneratorType::DFS => Box::new(DFS::new(maze)),
            GeneratorType::Kruskal => Box::new(Kruskal::new(maze, random)),
            GeneratorType::Prim => Box::new(Prim::new(maze)),
            GeneratorType::Eller => Box::new(Eller::new(maze)),
            GeneratorType::HuntKill => Box::new(HuntKill::new(maze)),
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
            "hunt-kill" => Ok(GeneratorType::HuntKill),
            _ => Err(Error::UnsupportedGenerator(s.to_string())),
        }
    }
}

pub trait Generator {
    fn is_done(&self) -> bool;
    fn tick(&mut self, maze: &mut Maze, random: &mut StdRng) -> Result<()>;
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

    fn available_neighbour(&self, maze: &Maze, random: &mut StdRng) -> Option<(Coord, Direction)> {
        let current = match self.current {
            Some(ref current) => current,
            None => return None,
        };

        if maze.end == *current {
            return None;
        }

        let mut neighbours = maze.neighbours(current);
        random.shuffle(&mut neighbours);

        neighbours
            .into_iter()
            .find(|(c, _)| !maze.explored.contains(&c))
    }
}

impl Generator for DFS {
    fn is_done(&self) -> bool {
        self.current.is_none()
    }

    fn tick(&mut self, maze: &mut Maze, random: &mut StdRng) -> Result<()> {
        maze.highlight_bright.clear();

        let current = match self.current {
            Some(ref current) => *current,
            None => return Ok(()),
        };

        maze.highlight_bright.insert(current);
        maze.highlight_medium.insert(current);
        maze.explored.insert(current);

        match self.available_neighbour(&maze, random) {
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
    pub fn new(maze: &Maze, random: &mut StdRng) -> Kruskal {
        let mut walls = maze
            .walls
            .iter()
            .filter(|w| w.removable())
            .cloned()
            .collect::<Vec<_>>();

        random.shuffle(&mut walls);

        Kruskal {
            walls,
            sets: maze
                .cells
                .keys()
                .map(|c| {
                    let mut set = HashSet::new();
                    set.insert(*c);

                    set
                })
                .collect(),
        }
    }

    fn join(&mut self, c1: Coord, c2: Coord, maze: &Maze) -> Result<JoinResult> {
        let mut c1_set = self
            .sets
            .drain_filter(|s| s.contains(&c1))
            .next()
            .ok_or_else(|| Error::MissingSet(c1))?;

        if c1_set.contains(&c2) {
            self.sets.push(c1_set);
            return Ok(JoinResult::Nop);
        }

        let c2_set = self
            .sets
            .drain_filter(|s| s.contains(&c2))
            .next()
            .ok_or_else(|| Error::MissingSet(c2))?;

        for c in c2_set {
            for (neighbour, direction) in maze.neighbours(&c) {
                if c1_set.contains(&neighbour) {
                    self.walls.remove_item(&maze.wall(&c, &direction));
                }
            }
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

    fn tick(&mut self, maze: &mut Maze, _random: &mut StdRng) -> Result<()> {
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

            match self.join(c1, c2, maze) {
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

    fn random_cell(&mut self, random: &mut StdRng) -> Option<Coord> {
        if self.cells.is_empty() {
            return None;
        }
        let cell_list: Vec<_> = self.cells.iter().collect();
        // Unwrap is safe here because of the is_empty check above
        let cell = random.choose(&cell_list).unwrap();

        Some(**cell)
    }
}

impl Generator for Prim {
    fn is_done(&self) -> bool {
        self.cells.is_empty()
    }

    fn tick(&mut self, maze: &mut Maze, random: &mut StdRng) -> Result<()> {
        maze.highlight_bright.clear();

        if let Some(cell) = self.random_cell(random) {
            if (cell == maze.start || cell == maze.end) && maze.explored.contains(&cell) {
                return Ok(());
            }

            maze.explored.insert(cell);
            maze.highlight_medium.remove(&cell);
            maze.highlight_bright.insert(cell);
            self.cells.remove(&cell);

            let explored_neighbours: Vec<_> = maze
                .neighbours(&cell)
                .into_iter()
                .filter(|(n, _)| maze.explored.contains(n))
                .collect();

            let unknown_neighbours: Vec<_> = maze
                .neighbours(&cell)
                .into_iter()
                .filter(|(n, _)| !maze.explored.contains(n))
                .collect();

            if let Some((_, direction)) = random.choose(&explored_neighbours) {
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
enum EllerMode {
    Horizontal,
    Vertical,
}

pub struct Eller {
    current: Coord,
    last_row: i32,
    mode: EllerMode,
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
            mode: EllerMode::Horizontal,
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

    #[allow(clippy::map_entry)]
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
        self.mode == EllerMode::Vertical && self.current.y == self.last_row
    }

    fn tick(&mut self, maze: &mut Maze, random: &mut StdRng) -> Result<()> {
        maze.highlight_bright.clear();
        maze.highlight_medium.clear();
        maze.highlight_dark.clear();

        for x in 0..maze.maze_width() as i32 {
            maze.highlight_dark.insert([x, self.current.y].into());
        }

        maze.highlight_bright.insert(self.current);
        maze.explored.insert(self.current);

        match self.mode {
            EllerMode::Horizontal => {
                let current = self.current;
                let last_row = current.y == self.last_row;
                if let Some(neighbour) = maze.neighbour(&current, &Direction::East) {
                    if !self.same_set(&current, &neighbour) && (last_row || random.gen()) {
                        self.join(&current, neighbour);

                        let wall = maze.east_wall(&current);
                        maze.walls.remove(&wall);

                        maze.highlight_medium.insert(neighbour);
                    } else if !self.coord_to_set.contains_key(&neighbour) {
                        self.new_set(neighbour);
                    }
                    self.current = neighbour;
                } else {
                    self.mode = EllerMode::Vertical;
                }
            }
            EllerMode::Vertical => {
                let current_set = self.coord_to_set[&self.current];
                let last_in_set = maze
                    .neighbour(&self.current, &Direction::West)
                    .filter(|c| self.coord_to_set[&c] == current_set)
                    .is_none();
                let connected = self.connected_vertically(current_set, maze);
                let force_join = last_in_set && !connected;

                let current = self.current;
                if force_join || random.gen() {
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
                    self.mode = EllerMode::Horizontal;
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

#[derive(PartialEq, Eq)]
enum HuntKillMode {
    Hunt,
    Kill,
}

pub struct HuntKill {
    current: Option<Coord>,
    last_completed_column: i32,
    last_completed_row: i32,
    mode: HuntKillMode,
}

impl HuntKill {
    pub fn new(_maze: &Maze) -> HuntKill {
        HuntKill {
            current: Some(Coord { x: 0, y: 0 }),
            last_completed_column: 0,
            last_completed_row: 0,
            mode: HuntKillMode::Kill,
        }
    }

    fn available_neighbour(&self, maze: &Maze, random: &mut StdRng) -> Option<(Coord, Direction)> {
        let current = match self.current {
            Some(ref current) => current,
            None => return None,
        };

        let mut neighbours = maze.neighbours(current);
        random.shuffle(&mut neighbours);

        neighbours
            .into_iter()
            .find(|(c, _)| !maze.explored.contains(&c))
    }

    fn current_row(&self, maze: &Maze) -> Vec<Coord> {
        let current = match self.current {
            Some(ref current) => current,
            None => return vec![],
        };

        (0..maze.maze_width() as i32)
            .into_iter()
            .map(|x| [x, current.y].into())
            .collect()
    }

    fn visited_neighbour(&self, maze: &Maze, random: &mut StdRng) -> Option<(Coord, Direction)> {
        let current = match self.current {
            Some(ref current) => current,
            None => return None,
        };

        if maze.explored.contains(&current) {
            return None;
        }

        let mut neighbours = maze.neighbours(current);
        random.shuffle(&mut neighbours);

        neighbours
            .into_iter()
            .find(|(c, _)| maze.explored.contains(&c))
    }

    fn tick_kill(&mut self, maze: &mut Maze, random: &mut StdRng) -> Result<()> {
        let current = match self.current {
            Some(ref current) => *current,
            None => return Ok(()),
        };

        maze.highlight_bright.insert(current);
        maze.highlight_medium.insert(current);
        maze.explored.insert(current);

        match self.available_neighbour(&maze, random) {
            Some((neighbour, _)) => {
                maze.link(&current, &neighbour)?;
                self.current = Some(neighbour);
            }
            None => {
                self.mode = HuntKillMode::Hunt;
                self.current = Some([self.last_completed_column, self.last_completed_row].into());
            }
        };
        Ok(())
    }

    fn tick_hunt(&mut self, maze: &mut Maze, random: &mut StdRng) -> Result<()> {
        maze.highlight_medium.clear();

        let current = match self.current {
            Some(ref current) => *current,
            None => return Ok(()),
        };

        for c in self.current_row(maze) {
            maze.highlight_medium.insert(c);
        }

        maze.highlight_bright.insert(current);

        match self.visited_neighbour(&maze, random) {
            Some((neighbour, _)) => {
                maze.highlight_medium.clear();
                maze.highlight_bright.insert(neighbour);
                maze.link(&current, &neighbour)?;
                self.last_completed_column = current.x;
                self.mode = HuntKillMode::Kill;
            }
            None => {
                if let Some(neighbour) = maze.neighbour(&current, &Direction::East) {
                    self.current = Some(neighbour);
                } else if current.y < maze.maze_height() as i32 - 1 {
                    if self
                        .current_row(maze)
                        .iter()
                        .all(|c| maze.explored.contains(c))
                    {
                        self.last_completed_row = current.y + 1;
                    }
                    self.current = Some([0, current.y + 1].into());
                } else {
                    self.current = None;
                }
            }
        };
        Ok(())
    }
}

impl Generator for HuntKill {
    fn is_done(&self) -> bool {
        self.current.is_none()
    }

    fn tick(&mut self, maze: &mut Maze, random: &mut StdRng) -> Result<()> {
        maze.highlight_bright.clear();

        match self.mode {
            HuntKillMode::Hunt => self.tick_hunt(maze, random),
            HuntKillMode::Kill => self.tick_kill(maze, random),
        }
    }
}
