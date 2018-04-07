use rand::{thread_rng as random, Rng};

use errors::Result;
use maze::{Coord, Direction, Maze};

pub trait Generator {
    fn tick(&mut self, maze: &mut Maze) -> Result<()>;
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
    fn tick(&mut self, maze: &mut Maze) -> Result<()> {
        let current = match self.current {
            Some(ref current) => current.clone(),
            None => return Ok(()),
        };

        match self.available_neighbour(&maze) {
            Some((neighbour, _)) => {
                maze.explored.push(neighbour.clone());
                maze.link(current, neighbour)?;
                self.stack.push(current);
                self.current = Some(neighbour);
            }
            None => self.current = self.stack.pop(),
        }

        match self.current {
            Some(ref current) => maze.highlighted = vec![current.clone()],
            None => maze.highlighted = vec![],
        };

        Ok(())
    }
}
