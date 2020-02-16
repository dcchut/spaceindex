use thiserror::Error;

mod line_segment;
mod point;
mod region;
mod shape;
#[cfg(test)]
mod tests;

pub use line_segment::LineSegment;
pub use point::Point;
pub use region::Region;
pub use shape::Shape;

#[derive(Error, Debug, PartialEq)]
pub enum ShapelikeError {
    #[error("the current operation is unsupported")]
    UnsupportedOperation,
    #[error("shapes have unmatched dimensions {0} and {1}")]
    UnmatchedDimensions(usize, usize),
    #[error("unexpected dimension {0} (expected: {1})")]
    UnexpectedDimension(usize, usize),
}

pub trait Shapelike {
    fn get_center(&self) -> Point;
    fn get_dimension(&self) -> usize;
    fn get_min_bounding_region(&self) -> Region;
    fn get_area(&self) -> f64;
    fn get_min_distance(&self, other: &Shape) -> Result<f64, ShapelikeError>;

    fn intersects_shape(&self, other: &Shape) -> Result<bool, ShapelikeError>
    where
        Self: Sized,
    {
        check_dimensions_match(self, other)?;

        match other {
            Shape::Point(point) => self.contains_point(point),
            Shape::LineSegment(line) => self.intersects_line_segment(line),
            Shape::Region(region) => self.intersects_region(region),
        }
    }

    fn contains_point(&self, _point: &Point) -> Result<bool, ShapelikeError> {
        Err(ShapelikeError::UnsupportedOperation)
    }

    fn intersects_line_segment(&self, _line: &LineSegment) -> Result<bool, ShapelikeError> {
        Err(ShapelikeError::UnsupportedOperation)
    }

    fn intersects_region(&self, _region: &Region) -> Result<bool, ShapelikeError> {
        Err(ShapelikeError::UnsupportedOperation)
    }
}

/// Returns the minimum distance between two points.
fn min_distance_point(s: &Point, t: &Point) -> f64 {
    s.coordinate_iter()
        .zip(t.coordinate_iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// Returns the minimum distance between a point and a line.
fn min_distance_point_line(s: &Point, t: &LineSegment) -> Result<f64, ShapelikeError> {
    // only supported for 2D objects
    if s.get_dimension() != 2 {
        return Err(ShapelikeError::UnsupportedOperation);
    }

    let (x1, x2) = t.get_coordinate(0);
    let (y1, y2) = t.get_coordinate(1);

    let x0 = s.get_coordinate(0);
    let y0 = s.get_coordinate(1);

    // avoid /0
    if x2 >= x1 - std::f64::EPSILON && x2 <= x1 + std::f64::EPSILON {
        return Ok((x0 - x1).abs());
    }

    // avoid /0
    if y2 >= y1 - std::f64::EPSILON && y2 <= y1 + std::f64::EPSILON {
        return Ok((y0 - y1).abs());
    }

    Ok(((x2 - x1) * (y1 - y0) - (x1 - x0) * (y2 - y1)).abs()
        / ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt())
}

/// Returns the minimum distance between a point and a region.
fn min_distance_point_region(s: &Point, t: &Region) -> f64 {
    let mut distance = 0.0;

    for (coordinate, (low, high)) in s.coordinate_iter().zip(t.coordinates_iter()) {
        distance += if coordinate < low {
            (low - coordinate).powi(2)
        } else if coordinate > high {
            (coordinate - high).powi(2)
        } else {
            0.0
        };
    }

    distance
}

/// Returns the minimum distance between two regions.
fn min_distance_region(s: &Region, t: &Region) -> f64 {
    let mut distance = 0.0;

    for ((s_low, s_high), (t_low, t_high)) in s.coordinates_iter().zip(t.coordinates_iter()) {
        let x = {
            if t_high < s_low {
                (t_high - s_low).abs()
            } else if s_high < t_low {
                (t_low - s_high).abs()
            } else {
                0.0
            }
        };

        distance += x * x;
    }

    distance
}

/// Helper function to check that two shapelike's have the same dimension.
fn check_dimensions_match<S: Shapelike, T: Shapelike>(
    s1: &S,
    s2: &T,
) -> Result<(), ShapelikeError> {
    let d1 = s1.get_dimension();
    let d2 = s2.get_dimension();

    if d1 != d2 {
        Err(ShapelikeError::UnmatchedDimensions(d1, d2))
    } else {
        Ok(())
    }
}
