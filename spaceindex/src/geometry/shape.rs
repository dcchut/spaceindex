use crate::geometry::{LineSegment, Point, Region, Shapelike, ShapelikeError};

#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Point(Point),
    Region(Region),
    LineSegment(LineSegment),
}

// TODO: write a derive macro to write out this boilerplate
impl Shapelike for Shape {
    fn get_center(&self) -> Point {
        match self {
            Shape::Point(point) => point.get_center(),
            Shape::LineSegment(line) => line.get_center(),
            Shape::Region(region) => region.get_center(),
        }
    }

    fn get_dimension(&self) -> usize {
        match self {
            Shape::Point(point) => point.get_dimension(),
            Shape::LineSegment(line) => line.get_dimension(),
            Shape::Region(region) => region.get_dimension(),
        }
    }

    fn get_min_bounding_region(&self) -> Region {
        match self {
            Shape::Point(point) => point.get_min_bounding_region(),
            Shape::LineSegment(line) => line.get_min_bounding_region(),
            Shape::Region(region) => region.get_min_bounding_region(),
        }
    }

    fn get_area(&self) -> f64 {
        match self {
            Shape::Point(point) => point.get_area(),
            Shape::LineSegment(line) => line.get_area(),
            Shape::Region(region) => region.get_area(),
        }
    }

    fn get_min_distance(&self, other: &Shape) -> Result<f64, ShapelikeError> {
        match self {
            Shape::Point(point) => point.get_min_distance(other),
            Shape::LineSegment(line) => line.get_min_distance(other),
            Shape::Region(region) => region.get_min_distance(other),
        }
    }
}

impl Into<Shape> for Point {
    fn into(self) -> Shape {
        Shape::Point(self)
    }
}

impl Into<Shape> for LineSegment {
    fn into(self) -> Shape {
        Shape::LineSegment(self)
    }
}

impl Into<Shape> for Region {
    fn into(self) -> Shape {
        Shape::Region(self)
    }
}
