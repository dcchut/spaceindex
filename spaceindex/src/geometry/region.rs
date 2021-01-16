use geo::bounding_rect::BoundingRect;
use std::borrow::Cow;

use crate::geometry::point::IntoPoint;
use crate::geometry::{
    check_dimensions_match, min_distance_point_region, min_distance_region, LineSegment, Point,
    Shape, Shapelike, ShapelikeError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub coordinates: Vec<(f64, f64)>,
}

impl Region {
    /// Creates a new [`Region`].
    pub fn new(coordinates: Vec<(f64, f64)>) -> Self {
        Self { coordinates }
    }

    /// Creates an infinite [`Region']
    pub fn infinite(dimension: usize) -> Self {
        let coordinates = vec![(std::f64::MIN, std::f64::MAX); dimension];

        Self::new(coordinates)
    }

    /// Returns an iterator over coordinates in this region.
    pub fn coordinates_iter(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.coordinates.iter().cloned()
    }

    /// Constructs a region from a pair of points.
    #[inline(always)]
    pub fn from_points(a: &Point, b: &Point) -> Self {
        Self::new(a.coordinate_iter().zip(b.coordinate_iter()).collect())
    }

    /// Determines whether this region contains another region `other`.
    pub fn contains_region(&self, other: &Region) -> Result<bool, ShapelikeError> {
        check_dimensions_match(self, other)?;

        Ok(!self
            .coordinates_iter()
            .zip(other.coordinates_iter())
            .any(|((s_low, s_high), (o_low, o_high))| s_low > o_low || s_high < o_high))
    }

    /// Combines this region with another region `other`.
    #[inline(always)]
    pub fn combine_region(&self, other: &Region) -> Result<Region, ShapelikeError> {
        check_dimensions_match(self, other)?;

        Ok(Region::new(
            self.coordinates_iter()
                .zip(other.coordinates_iter())
                .map(|((s_low, s_high), (o_low, o_high))| {
                    (f64::min(s_low, o_low), f64::max(s_high, o_high))
                })
                .collect(),
        ))
    }

    /// Combines this region with another region `other` in place.
    #[inline(always)]
    pub fn combine_region_in_place(&mut self, other: &Region) {
        check_dimensions_match(self, other).unwrap();

        for ((s_low, s_high), (o_low, o_high)) in
            self.coordinates.iter_mut().zip(other.coordinates_iter())
        {
            *s_low = f64::min(*s_low, o_low);
            *s_high = f64::max(*s_high, o_high);
        }
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

    #[inline(always)]
    fn get_area(&self) -> f64 {
        let mut area = 1.0;

        for (low, high) in self.coordinates_iter() {
            area *= high - low;
        }

        area
    }

    fn get_min_distance(&self, other: &Shape) -> Result<f64, ShapelikeError> {
        check_dimensions_match(self, other)?;

        match other {
            Shape::Point(point) => Ok(min_distance_point_region(point, self)),
            Shape::LineSegment(_) => Err(ShapelikeError::UnsupportedOperation),
            Shape::Region(region) => Ok(min_distance_region(region, self)),
        }
    }

    fn contains_point(&self, point: &Point) -> Result<bool, ShapelikeError> {
        check_dimensions_match(self, point)?;

        Ok(!point
            .coordinate_iter()
            .zip(self.coordinates_iter())
            .any(|(pc, (low, high))| low > pc || high < pc))
    }

    fn intersects_line_segment(&self, line: &LineSegment) -> Result<bool, ShapelikeError> {
        if self.get_dimension() != 2 {
            return Err(ShapelikeError::UnexpectedDimension(self.get_dimension(), 2));
        }

        check_dimensions_match(self, line)?;

        let (low0, high0) = self.coordinates[0];
        let (low1, high1) = self.coordinates[1];

        let ll = Point::new(vec![low0, high0]);
        let ur = Point::new(vec![low1, high1]);
        let ul = Point::new(vec![low0, high1]);
        let lr = Point::new(vec![high0, low1]);

        let (start, end) = line.get_points();

        Ok(self.contains_point(start)?
            || self.contains_point(end)?
            || line.intersects_line_segment(&LineSegment::new(ll.clone(), ul.clone()))?
            || line.intersects_line_segment(&LineSegment::new(ul, ur.clone()))?
            || line.intersects_line_segment(&LineSegment::new(ur, lr.clone()))?
            || line.intersects_line_segment(&LineSegment::new(lr, ll))?)
    }

    fn intersects_region(&self, region: &Region) -> Result<bool, ShapelikeError> {
        check_dimensions_match(self, region)?;

        Ok(!self
            .coordinates_iter()
            .zip(region.coordinates_iter())
            .any(|((s_low, s_high), (o_low, o_high))| s_low > o_high || s_high < o_low))
    }
}

/// We can't implement Into<Cow<'a, Region>> for types such as (f64, f64) or ((f64, f64), (f64, f64)),
/// so we have the [`IntoRegion<'a>]` trait which is essentially identical.  This makes many of our
/// internal API's much nicer to work with.
pub trait IntoRegion<'a> {
    fn into_region(self) -> Cow<'a, Region>;
}

impl<'a> IntoRegion<'a> for Region {
    fn into_region(self) -> Cow<'a, Region> {
        Cow::Owned(self)
    }
}

impl<'a> IntoRegion<'a> for Cow<'a, Region> {
    fn into_region(self) -> Cow<'a, Region> {
        self
    }
}

impl<'a> IntoRegion<'a> for (f64, f64) {
    fn into_region(self) -> Cow<'a, Region> {
        Cow::Owned(Region::new(vec![(self.0, self.1)]))
    }
}

impl<'a> IntoRegion<'a> for ((f64, f64), (f64, f64)) {
    fn into_region(self) -> Cow<'a, Region> {
        Cow::Owned(Region::from_points(
            &(self.0).into_pt(),
            &(self.1).into_pt(),
        ))
    }
}

impl<'a> IntoRegion<'a> for &geo_types::LineString<f64> {
    fn into_region(self) -> Cow<'a, Region> {
        let bounding_rect = self.bounding_rect().expect("failed to get bounding rect");
        (
            (bounding_rect.min().x, bounding_rect.min().y),
            (bounding_rect.max().x, bounding_rect.max().y),
        )
            .into_region()
    }
}
