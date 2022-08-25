#[cfg(feature = "std")]
use std::fmt::{Debug, Display, Formatter};

use crate::prelude::*;

/// Two-Dimensional coordinates used for addressing fields in a maze.
#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Coordinates {
    /// X component
    pub x: i32,
    /// Y component
    pub y: i32,
}

impl Coordinates {
    /// Create a new instance from the specified coordinate components
    pub fn new(x: i32, y: i32) -> Self {
        Coordinates { x, y }
    }

    /// Returns the next neighboring coordinates in a specific direction
    pub fn next(&self, direction: &Direction) -> Self {
        Self {
            x: self.x
                + match direction {
                    Direction::East => 1,
                    Direction::West => -1,
                    _ => 0,
                },
            y: self.y
                + match direction {
                    Direction::North => -1,
                    Direction::South => 1,
                    _ => 0,
                },
        }
    }
}

impl From<Coordinates> for (i32, i32) {
    fn from(c: Coordinates) -> Self {
        (c.x, c.y)
    }
}

impl From<(i32, i32)> for Coordinates {
    fn from(source: (i32, i32)) -> Self {
        Self {
            x: source.0,
            y: source.1,
        }
    }
}

#[cfg(feature = "std")]
impl Display for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("({}, {})", self.x, self.y))
    }
}
