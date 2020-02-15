use crate::geometry::{
    min_distance_point_region, min_distance_region, Point, Shape, Shapelike, ShapelikeError,
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
        Point::new(self.coordinates_iter().map(|(x, y)| (x + y) / 2.0).collect())
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

    fn get_min_distance(&self, other: &Shape) -> Result<f64, ShapelikeError> {
        self.check_dimensions_match(other)?;

        match other {
            Shape::Point(point) => Ok(min_distance_point_region(point, self)),
            Shape::LineSegment(_) => Err(ShapelikeError::UnsupportedOperation),
            Shape::Region(region) => Ok(min_distance_region(region, self)),
        }
    }
}
