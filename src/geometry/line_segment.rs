use crate::geometry::{min_distance_point_line, Point, Region, Shape, Shapelike, ShapelikeError};

#[derive(Debug, Clone, PartialEq)]
pub struct LineSegment {
    start: Point,
    end: Point,
}

impl LineSegment {
    pub fn new(start: Point, end: Point) -> Self {
        // TODO: consider making this function return a result and not panicking
        assert_eq!(start.get_dimension(), end.get_dimension());

        Self { start, end }
    }

    #[inline(always)]
    pub fn get_coordinate(&self, index: usize) -> (f64, f64) {
        (self.start.get_coordinate(index), self.end.get_coordinate(index))
    }

    pub fn coordinate_iter(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.start
            .coordinate_iter()
            .zip(self.end.coordinate_iter())
    }
}

impl Shapelike for LineSegment {
    fn get_center(&self) -> Point {
        let mut coordinates = Vec::with_capacity(self.get_dimension());

        for (start_coord, end_coord) in self.coordinate_iter() {
            coordinates
                .push(((start_coord - end_coord).abs() / 2.0) + f64::min(start_coord, end_coord));
        }

        Point::new(coordinates)
    }

    fn get_dimension(&self) -> usize {
        self.start.get_dimension()
    }

    fn get_min_bounding_region(&self) -> Region {
        let mut coordinates = Vec::with_capacity(self.get_dimension());

        for (start_coord, end_coord) in self.coordinate_iter() {
            coordinates.push((
                f64::min(start_coord, end_coord),
                f64::max(start_coord, end_coord),
            ));
        }

        Region::new(coordinates)
    }

    fn get_area(&self) -> f64 {
        0.0
    }

    fn get_min_distance(&self, other: &Shape) -> Result<f64, ShapelikeError> {
        self.check_dimensions_match(other)?;

        match other {
            Shape::Point(point) => min_distance_point_line(point, self),
            Shape::LineSegment(_) => Err(ShapelikeError::UnsupportedOperation),
            Shape::Region(_) => Err(ShapelikeError::UnsupportedOperation),
        }
    }
}
