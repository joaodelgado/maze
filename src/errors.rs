use std::result::Result as StdResult;
use std::{error, fmt};

use maze::{Coord, Wall};

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    NotNeighbours(Coord, Coord),
    BorderWall(Wall),
    MissingSet(Coord),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NotNeighbours(ref c1, ref c2) => {
                write!(f, "{} and {} are not neighbours", c1, c2)
            }
            Error::BorderWall(ref wall) => write!(f, "Tried to remove non border wall {}", wall),
            Error::MissingSet(ref coord) => write!(f, "Missing set for coord {}", coord),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NotNeighbours(_, _) => "Two provided coordinates are not neighbours",
            Error::BorderWall(_) => "Tried to remove border wall",
            Error::MissingSet(_) => "Missing set for a given coord",
        }
    }
}
