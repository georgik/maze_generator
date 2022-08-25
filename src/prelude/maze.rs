#[cfg(feature = "std")]
use std::fmt::Write;

#[cfg(feature = "std")]
use anyhow::{anyhow, Result};
#[cfg(feature = "std")]
use petgraph::algo::is_isomorphic;
use petgraph::graphmap::GraphMap;
#[cfg(feature = "std")]
use petgraph::stable_graph::DefaultIx;
use petgraph::Undirected;
use petgraph::lib::Vec;

use crate::prelude::*;

pub(crate) type MazeGraph = GraphMap<Coordinates, (), Undirected>;

/// A collection of [`Field`]s with passages between them.
///
/// Use one of the provided [`Generator`]s to create an instance of this type.
#[derive(Clone)]
pub struct Maze {
    pub(crate) graph: MazeGraph,
    /// At which coordinates the start field lies
    pub start: Coordinates,
    /// At which coordinates the goal field lies
    pub goal: Coordinates,
    /// How large the maze is in (width, height) format
    pub size: (i32, i32),
}

impl Maze {
    pub(crate) fn new(width: i32, height: i32, start: Coordinates, goal: Coordinates) -> Self {
        debug_assert!(width > 0, "maze width should be >0");
        debug_assert!(height > 0, "maze height should be >0");

        Maze {
            graph: GraphMap::with_capacity((width * height) as usize, 0),
            size: (width, height),
            start,
            goal,
        }
    }

    /// Retrieve the [`Field`] which is located at `coordinates`
    pub fn get_field(&self, coordinates: &Coordinates) -> Option<Field> {
        if self.are_coordinates_inside(coordinates) {
            // figure out in which directions passages exist
            let passages: Vec<_> = Direction::all()
                .iter()
                .filter(|dir| {
                    self.graph
                        .contains_edge(*coordinates, coordinates.next(dir))
                })
                .copied()
                .collect();

            let field_type = if &self.start == coordinates {
                FieldType::Start
            } else if &self.goal == coordinates {
                FieldType::Goal
            } else {
                FieldType::Normal
            };

            Some(Field::new(field_type, *coordinates, passages))
        } else {
            None
        }
    }

    pub(crate) fn are_coordinates_inside(&self, coordinates: &Coordinates) -> bool {
        coordinates.x >= 0
            && coordinates.x < self.size.0
            && coordinates.y >= 0
            && coordinates.y < self.size.1
    }
}

#[cfg(feature = "std")]
impl std::fmt::Debug for Maze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for iy in 0..self.size.1 {
            // print top passage
            for ix in 0..self.size.0 {
                f.write_str("·")?;
                if self
                    .get_field(&(ix, iy).into())
                    .ok_or(std::fmt::Error {})?
                    .has_passage(&Direction::North)
                {
                    f.write_str(" ")?;
                } else {
                    f.write_str("-")?;
                }
            }
            f.write_str("·\n")?;

            // print left passage and room icon
            for ix in 0..self.size.0 {
                let field = self.get_field(&(ix, iy).into()).ok_or(std::fmt::Error {})?;
                if field.has_passage(&Direction::West) {
                    f.write_str(" ")?;
                } else {
                    f.write_str("|")?;
                }

                f.write_str(match field.field_type {
                    FieldType::Start => "S",
                    FieldType::Goal => "G",
                    _ => " ",
                })?;
            }
            f.write_str("|\n")?;

            // print bottom line
            if iy == self.size.1 - 1 {
                for _ix in 0..self.size.0 {
                    f.write_str("·-")?;
                }
                f.write_str("·\n")?;
            }
        }

        Ok(())
    }
}

impl Maze {
    /// Generate an SVG version of the maze, returned as a String which you can then write to a file or use directly
    #[cfg(feature = "std")]
    pub fn to_svg(&self, svgoptions: SvgOptions) -> Result<String> {
        // Get the options for convenience
        let padding = svgoptions.padding; // Pad the maze all around by this amount.
        let markersize = svgoptions.markersize; // Size of the Start and Goal markers
        let mut height = match svgoptions.height {
            // Height and width of the maze image (excluding padding), in pixels
            None => (2 + self.size.1) * padding,
            Some(h) => h,
        };
        let mut width = height * self.size.0 / self.size.1; // Derive width based on height

        // Scaling factors mapping maze coordinates to image/svg coordinates
        let scx = width / self.size.0;
        let scx2 = scx / 2;
        let scy = height / self.size.1;
        let scy2 = scy / 2;
        // Recalculate integer width, height now that we have the actual elements
        width = scx * self.size.0;
        height = scy * self.size.1;
        let mut x1;
        let mut x2;
        let mut y1;
        let mut y2;

        // Write the SVG to the return String
        let mut svg = String::new();
        writeln!(svg, "<?xml version=\"1.0\" encoding=\"utf-8\"?>")?;
        writeln!(svg, "<svg xmlns=\"http://www.w3.org/2000/svg\"")?;
        writeln!(svg, "    xmlns:xlink=\"http://www.w3.org/1999/xlink\"")?;
        writeln!(
            svg,
            "    width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\">",
            width + 2 * padding,
            height + 2 * padding,
            -padding,
            -padding,
            width + 2 * padding,
            height + 2 * padding
        )?;

        writeln!(svg, "<defs>\n<style type=\"text/css\"><![CDATA[")?;
        writeln!(svg, "line {{")?;
        writeln!(
            svg,
            "    stroke: {};\n    stroke-linecap: square;",
            svgoptions.strokecol
        )?;
        writeln!(svg, "    stroke-width: {};\n}}", svgoptions.strokewidth)?;
        writeln!(svg, "]]></style>\n</defs>")?;

        for iy in 0..self.size.1 {
            // print top passage
            for ix in 0..self.size.0 {
                if self
                    .get_field(&(ix, iy).into())
                    .ok_or_else(|| {
                        anyhow!("Could not get maze field at coordinates {},{}", ix, iy)
                    })?
                    .has_passage(&Direction::North)
                {
                    // Do nothing. This code structure keeps the SVG output aligned with the original text debug output
                } else {
                    x1 = ix * scx;
                    y1 = iy * scy;
                    x2 = (ix + 1) * scx;
                    y2 = iy * scy;
                    writeln!(
                        svg,
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                        x1, y1, x2, y2
                    )?;
                }
            }

            // print left passage and room markers
            for ix in 0..self.size.0 {
                let field = self.get_field(&(ix, iy).into()).ok_or_else(|| {
                    anyhow!("Could not get maze field at coordinates {},{}", ix, iy)
                })?;
                if field.has_passage(&Direction::West) {
                    // Do nothing
                } else {
                    x1 = ix * scx;
                    y1 = iy * scy;
                    x2 = ix * scx;
                    y2 = (iy + 1) * scy;
                    writeln!(
                        svg,
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                        x1, y1, x2, y2
                    )?;
                }
                // Special cells
                match field.field_type {
                    FieldType::Start => {
                        x1 = ix * scx + scx2;
                        y1 = iy * scy + scy2;
                        writeln!(svg, "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" stroke=\"{}\" stroke-width=\"{}\" fill=\"{}\" />", x1, y1, markersize, svgoptions.startcol, markersize + 1, svgoptions.startcol)?;
                    }
                    FieldType::Goal => {
                        x1 = ix * scx + scx2;
                        y1 = iy * scy + scy2;
                        writeln!(svg, "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" stroke=\"{}\" stroke-width=\"{}\" fill=\"{}\" />", x1, y1, markersize, svgoptions.goalcol, markersize + 1, svgoptions.goalcol)?;
                    }
                    _ => continue,
                };
            }

            // print bottom border line
            x1 = 0;
            y1 = (self.size.1) * scy;
            x2 = (self.size.0) * scx;
            y2 = (self.size.1) * scy;
            writeln!(
                svg,
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                x1, y1, x2, y2
            )?;

            // print right border line
            x1 = (self.size.0) * scx;
            y1 = 0;
            x2 = (self.size.0) * scx;
            y2 = (self.size.1) * scy;
            writeln!(
                svg,
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                x1, y1, x2, y2
            )?;
        }
        writeln!(svg, "</svg>")?;

        Ok(svg)
    }
}

// implemented as into and not accessor because after exposing the internal graph, data integrity
// can not be guaranteed (size, start, goal could be made invalid).
impl From<Maze> for MazeGraph {
    fn from(m: Maze) -> Self {
        m.graph
    }
}

#[cfg(feature = "std")]
impl PartialEq for Maze {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.goal == other.goal
            && self.size == other.size
            && is_isomorphic(
                &self.graph.clone().into_graph::<DefaultIx>(),
                &other.graph.clone().into_graph::<DefaultIx>(),
            )
    }
}

#[cfg(feature = "std")]
impl Eq for Maze {}
