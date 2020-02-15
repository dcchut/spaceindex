use crate::geometry::{
    check_dimensions_match, min_distance_point_line, Point, Region, Shape, Shapelike,
    ShapelikeError,
};

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
        (
            self.start.get_coordinate(index),
            self.end.get_coordinate(index),
        )
    }

    pub fn get_points(&self) -> (&Point, &Point) {
        (&self.start, &self.end)
    }

    pub fn coordinate_iter(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.start.coordinate_iter().zip(self.end.coordinate_iter())
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

    fn intersects_line_segment(&self, other: &LineSegment) -> Result<bool, ShapelikeError> {
        if self.get_dimension() != 2 {
            return Err(ShapelikeError::UnexpectedDimension(self.get_dimension(), 2));
        }

        check_dimensions_match(self, other)?;

        // geometry::Intersects
        unimplemented!()
    }

    fn intersects_region(&self, other: &Region) -> Result<bool, ShapelikeError> {
        if self.get_dimension() != 2 {
            return Err(ShapelikeError::UnexpectedDimension(self.get_dimension(), 2));
        }

        check_dimensions_match(self, other)?;

        // defer to the `Region` implementation
        other.intersects_line_segment(self)
    }

    fn get_min_distance(&self, other: &Shape) -> Result<f64, ShapelikeError> {
        check_dimensions_match(self, other)?;

        match other {
            Shape::Point(point) => min_distance_point_line(point, self),
            Shape::LineSegment(_) => Err(ShapelikeError::UnsupportedOperation),
            Shape::Region(_) => Err(ShapelikeError::UnsupportedOperation),
        }
    }
}
