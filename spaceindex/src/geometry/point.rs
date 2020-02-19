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

    fn get_min_distance(&self, other: &Shape) -> Result<f64, ShapelikeError> {
        check_dimensions_match(self, other)?;

        match other {
            Shape::Point(point) => Ok(min_distance_point(self, point)),
            Shape::LineSegment(line) => min_distance_point_line(self, line),
            Shape::Region(region) => Ok(min_distance_point_region(self, region)),
        }
    }

    fn intersects_region(&self, region: &Region) -> Result<bool, ShapelikeError> {
        region.contains_point(self)
    }
}

// Convenience traits for converting into points
pub trait IntoPoint {
    fn into_pt(self) -> Point;
}

impl IntoPoint for Point {
    fn into_pt(self) -> Point {
        self
    }
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
