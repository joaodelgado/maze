use rand::{thread_rng as random, Rng};

use maze::{Coord, Direction, Maze};

pub trait Generator {
    fn tick(&mut self, maze: &mut Maze);
}

pub struct RecursiveBacktracker {
    pub current: Option<Coord>,
    pub stack: Vec<Coord>,
}

impl RecursiveBacktracker {
    fn available_neighbour(&self, maze: &Maze) -> Option<(Coord, Direction)> {
        let current = match self.current {
            Some(ref current) => current,
            None => return None,
        };

        if maze.end == *current {
            return None;
        }

        let mut neighbours = current.neighbours();
        random().shuffle(&mut neighbours);

        neighbours
            .into_iter()
            .filter(|(c, _)| !maze.explored.contains(&c))
            .next()
    }
}

impl Generator for RecursiveBacktracker {
    fn tick(&mut self, maze: &mut Maze) {
        let current = match self.current {
            Some(ref current) => current.clone(),
            None => return,
        };

        match self.available_neighbour(&maze) {
            Some((neighbour, direction)) => {
                let mut current_cell = maze.cell_at(&current);
                let mut neighbour_cell = maze.cell_at(&neighbour);

                maze.explored.push(neighbour.clone());

                match direction {
                    Direction::North => {
                        current_cell.north(false);
                        neighbour_cell.south(false);
                    }
                    Direction::East => {
                        current_cell.east(false);
                        neighbour_cell.west(false);
                    }
                    Direction::South => {
                        current_cell.south(false);
                        neighbour_cell.north(false);
                    }
                    Direction::West => {
                        current_cell.west(false);
                        neighbour_cell.east(false);
                    }
                }

                maze.update_cell(current_cell);
                maze.update_cell(neighbour_cell);

                self.stack.push(current);
                self.current = Some(neighbour);
            }
            None => self.current = self.stack.pop(),
        }

        match self.current {
            Some(ref current) => maze.highlighted = vec![current.clone()],
            None => maze.highlighted = vec![],
        }
    }
}
