use crate::geometry::{
    check_dimensions_match, min_distance_point, min_distance_point_line, min_distance_point_region,
    Region, Shape, Shapelike, ShapelikeError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    coordinates: Vec<f64>,
}

impl Point {
    pub fn new(coordinates: Vec<f64>) -> Self {
        Self { coordinates }
    }

    #[inline(always)]
    pub fn get_coordinate(&self, index: usize) -> f64 {
        self.coordinates[index]
    }

    pub fn coordinate_iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.coordinates.iter().cloned()
    }
}

impl Shapelike for Point {
    fn get_center(&self) -> Point {
        self.clone()
    }

    fn get_dimension(&self) -> usize {
        self.coordinates.len()
    }

    fn get_min_bounding_region(&self) -> Region {
        Region::new(
            self.coordinates
                .iter()
                .zip(self.coordinates.iter())
                .map(|(x, y)| (*x, *y))
                .collect(),
        )
    }

    fn get_area(&self) -> f64 {
        0.0
    }

    fn intersects_region(&self, other: &Region) -> Result<bool, ShapelikeError> {
        other.contains_point(self)
    }

    fn get_min_distance(&self, other: &Shape) -> Result<f64, ShapelikeError> {
        check_dimensions_match(self, other)?;

        match other {
            Shape::Point(point) => Ok(min_distance_point(self, point)),
            Shape::LineSegment(line) => min_distance_point_line(self, line),
            Shape::Region(region) => Ok(min_distance_point_region(self, region)),
        }
    }
}

// Convenience traits for converting into points
pub trait IntoPoint {
    fn into_pt(self) -> Point;
}

impl IntoPoint for f32 {
    fn into_pt(self) -> Point {
        Point::new(vec![self as f64])
    }
}

impl IntoPoint for f64 {
    fn into_pt(self) -> Point {
        Point::new(vec![self])
    }
}

impl IntoPoint for (f64, f64) {
    fn into_pt(self) -> Point {
        Point::new(vec![self.0, self.1])
    }
}

impl IntoPoint for (f64, f64, f64) {
    fn into_pt(self) -> Point {
        Point::new(vec![self.0, self.1, self.2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_point_impl() {
        let _pt: Point = 0.1_f32.into_pt();
        let _pt: Point = 0.1_f64.into_pt();
        let _pt: Point = (0.5, 0.3).into_pt();
        let _pt: Point = (1.0, -1.0).into_pt();
        let _pt: Point = (1.0, 2.0, 3.0).into_pt();
    }

    #[test]
    fn test_point_shapelike_impl() {
        let p = (1.0, 2.0, 3.0).into_pt();

        // check our basic functions work
        assert_eq!(p.get_dimension(), 3);
        assert_eq!(p.get_area(), 0.0);
        assert_eq!(p.get_center(), p);

        let q = Shape::Point((2.0, 3.0, 4.0).into_pt());

        // the (minimum) distance between p and q is the square root of 3
        assert_eq!(p.get_min_distance(&q), Ok(3.0_f64.sqrt()));
    }
}
