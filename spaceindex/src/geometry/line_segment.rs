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

    /// Returns double the area of the triangle created by points `a`, `b`, and `c`.
    #[inline(always)]
    fn double_area_triangle(a: &Point, b: &Point, c: &Point) -> f64 {
        ((b.get_coordinate(0) - a.get_coordinate(0)) * (c.get_coordinate(1) - a.get_coordinate(1)))
            - ((c.get_coordinate(0) - a.get_coordinate(0))
                * (b.get_coordinate(1) - a.get_coordinate(1)))
    }

    /// Returns `true` if [`Point`] `self` is to the left of the segment (a, b).
    #[inline(always)]
    fn left_of(a: &Point, b: &Point, c: &Point) -> bool {
        Self::double_area_triangle(a, b, c) > 0.0
    }

    /// Returns `true` if `a`, `b`, and `c` are collinear.
    #[inline(always)]
    fn collinear(a: &Point, b: &Point, c: &Point) -> bool {
        Self::double_area_triangle(a, b, c) == 0.0
    }

    /// Determine whether the segments (a, b) and (c, d) intersect (excluding endpoints).
    #[inline(always)]
    fn intersects_proper(a: &Point, b: &Point, c: &Point, d: &Point) -> bool {
        if Self::collinear(a, b, c)
            || Self::collinear(a, b, d)
            || Self::collinear(c, d, a)
            || Self::collinear(c, d, b)
        {
            return false;
        }

        (Self::left_of(a, b, c) ^ Self::left_of(a, b, d))
            && (Self::left_of(c, d, a) ^ Self::left_of(c, d, b))
    }

    /// Assuming `a`, `b`, and `c` are collinear, determine whether `c` is between `a` and `b`.
    fn between(a: &Point, b: &Point, c: &Point) -> bool {
        fn _between(x1: f64, x2: f64, x3: f64) -> bool {
            ((x1 <= x3) && (x3 <= x2)) || ((x1 >= x3) && (x3 >= x2))
        }

        if !Self::collinear(a, b, c) {
            return false;
        }

        if (a.get_coordinate(0) - b.get_coordinate(0)).abs() > std::f64::EPSILON {
            // If `a` and `b` are not on the same vertical, compare along the x-axis.
            _between(
                a.get_coordinate(0),
                b.get_coordinate(0),
                c.get_coordinate(0),
            )
        } else {
            // other we have a vertical segment, so compare along the y-axis.
            _between(
                a.get_coordinate(1),
                b.get_coordinate(1),
                c.get_coordinate(1),
            )
        }
    }

    #[inline(always)]
    fn intersects(a: &Point, b: &Point, c: &Point, d: &Point) -> bool {
        Self::intersects_proper(a, b, c, d)
            || Self::between(a, b, c)
            || Self::between(a, b, d)
            || Self::between(c, d, a)
            || Self::between(c, d, b)
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
        check_dimensions_match(self, other)?;

        match other {
            Shape::Point(point) => min_distance_point_line(point, self),
            Shape::LineSegment(_) => Err(ShapelikeError::UnsupportedOperation),
            Shape::Region(_) => Err(ShapelikeError::UnsupportedOperation),
        }
    }

    fn intersects_line_segment(&self, line: &LineSegment) -> Result<bool, ShapelikeError> {
        if self.get_dimension() != 2 {
            return Err(ShapelikeError::UnexpectedDimension(self.get_dimension(), 2));
        }

        check_dimensions_match(self, line)?;

        Ok(Self::intersects(
            &self.start,
            &self.end,
            &line.start,
            &line.end,
        ))
    }

    fn intersects_region(&self, region: &Region) -> Result<bool, ShapelikeError> {
        if self.get_dimension() != 2 {
            return Err(ShapelikeError::UnexpectedDimension(self.get_dimension(), 2));
        }

        check_dimensions_match(self, region)?;

        // defer to the `Region` implementation
        region.intersects_line_segment(self)
    }
}
