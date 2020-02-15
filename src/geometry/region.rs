use crate::geometry::{
    check_dimensions_match, min_distance_point_region, min_distance_region, LineSegment, Point,
    Shape, Shapelike, ShapelikeError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    coordinates: Vec<(f64, f64)>,
}

impl Region {
    pub fn new(coordinates: Vec<(f64, f64)>) -> Self {
        Self { coordinates }
    }

    pub fn coordinates_iter(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.coordinates.iter().cloned()
    }
}

impl Shapelike for Region {
    fn get_center(&self) -> Point {
        // take the average of high + low coordinates
        Point::new(
            self.coordinates_iter()
                .map(|(x, y)| (x + y) / 2.0)
                .collect(),
        )
    }

    fn get_dimension(&self) -> usize {
        self.coordinates.len()
    }

    fn get_min_bounding_region(&self) -> Region {
        self.clone()
    }

    fn get_area(&self) -> f64 {
        let mut area = 1.0;

        for (low, high) in self.coordinates_iter() {
            area *= high - low;
        }

        area
    }

    fn contains_point(&self, other: &Point) -> Result<bool, ShapelikeError> {
        check_dimensions_match(self, other)?;

        Ok(!other
            .coordinate_iter()
            .zip(self.coordinates_iter())
            .any(|(pc, (low, high))| low > pc || high < pc))
    }

    fn intersects_line_segment(&self, other: &LineSegment) -> Result<bool, ShapelikeError> {
        if self.get_dimension() != 2 {
            return Err(ShapelikeError::UnexpectedDimension(self.get_dimension(), 2));
        }

        check_dimensions_match(self, other)?;

        let (low0, high0) = self.coordinates[0];
        let (low1, high1) = self.coordinates[1];

        let ll = Point::new(vec![low0, high0]);
        let ur = Point::new(vec![low1, high1]);
        let ul = Point::new(vec![low0, high1]);
        let lr = Point::new(vec![high0, low1]);

        let (start, end) = other.get_points();

        // Check whether the endpoints are within the region, or whether any of the bounding
        // segments of the region intersect the segment.
        Ok(self.contains_point(start)?
            || self.contains_point(end)?
            || other.intersects_line_segment(&LineSegment::new(ll.clone(), ul.clone()))?
            || other.intersects_line_segment(&LineSegment::new(ul, ur.clone()))?
            || other.intersects_line_segment(&LineSegment::new(ur, lr.clone()))?
            || other.intersects_line_segment(&LineSegment::new(lr, ll))?)
    }

    fn intersects_region(&self, other: &Region) -> Result<bool, ShapelikeError> {
        check_dimensions_match(self, other)?;

        Ok(!self
            .coordinates_iter()
            .zip(other.coordinates_iter())
            .any(|((s_low, s_high), (o_low, o_high))| s_low > o_high || s_high < o_low))
    }

    fn get_min_distance(&self, other: &Shape) -> Result<f64, ShapelikeError> {
        check_dimensions_match(self, other)?;

        match other {
            Shape::Point(point) => Ok(min_distance_point_region(point, self)),
            Shape::LineSegment(_) => Err(ShapelikeError::UnsupportedOperation),
            Shape::Region(region) => Ok(min_distance_region(region, self)),
        }
    }
}
