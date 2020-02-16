use crate::geometry::point::IntoPoint;
use crate::geometry::region::IntoRegion;
use crate::geometry::{LineSegment, Point, Region, Shape, Shapelike, ShapelikeError};
use crate::rtree::RTree;
use rand::Rng;

#[test]
fn test_line_intersections() {
    let p1 = Point::new(vec![1.0, 0.0]);
    let p2 = Point::new(vec![3.0, 2.0]);
    let p3 = Point::new(vec![2.0, 0.0]);
    let p3a = Point::new(vec![2.0, 3.0]);
    let p4 = Point::new(vec![2.0, 4.0]);
    let p5 = Point::new(vec![1.0, 1.0]);
    let p6 = Point::new(vec![2.5, 3.0]);
    let p7 = Point::new(vec![1.0, 2.0]);
    let p8 = Point::new(vec![0.0, -1.0]);
    let p9 = Point::new(vec![4.0, 3.0]);

    let ls1 = LineSegment::new(p1, p2);
    let ls2 = LineSegment::new(p3, p4.clone());
    let ls3 = LineSegment::new(p3a, p4);

    assert_eq!(ls1.intersects_line_segment(&ls2), Ok(true));
    assert_eq!(ls1.intersects_line_segment(&ls3), Ok(false));

    let r1 = Region::from_points(&p5, &p6);
    let r2 = Region::from_points(&p7, &p6);
    let r3 = Region::from_points(&p8, &p9);

    assert_eq!(r1.intersects_line_segment(&ls1), Ok(true));
    assert_eq!(ls1.intersects_region(&r1), Ok(true));

    assert_eq!(r2.intersects_line_segment(&ls1), Ok(false));
    assert_eq!(ls1.intersects_region(&r2), Ok(false));

    assert_eq!(r3.intersects_line_segment(&ls1), Ok(true));
    assert_eq!(ls1.intersects_region(&r3), Ok(true));
}

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

#[test]
fn test_region_area() {
    let ll = (0.0, 0.0).into_pt();
    let ur = (2.0, 2.0).into_pt();

    let r = Region::from_points(&ll, &ur);
    assert_eq!(r.get_area(), 4.0);
}

#[test]
fn test_combine_regions() {
    // Make the region going from (0.0, 0.0) -> (2.0, 2.0)
    let b = ((0.0, 0.0), (2.0, 2.0)).into_region();

    // Make the region going from (0.5, 0.5) -> (1.5, 3)
    let c = ((0.5, 0.5), (1.5, 3.0)).into_region();

    let combined_region = b
        .combine_region(&c)
        .expect("Failed to combine regions `b` and `c`");

    // The combined region should go from (0.0)-> (2, 3)
    assert_eq!(combined_region, ((0.0, 0.0), (2.0, 3.0)).into_region());
}

#[test]
fn test_rtree_insert() -> Result<(), ShapelikeError> {
    let (mut tree, root) = RTree::new(2);

    // insert 50 random positions
    let mut rng = rand::thread_rng();

    for _ in 0..50 {
        let xmin = rng.gen_range(0.0, 50.0);
        let ymin = rng.gen_range(0.0, 50.0);
        let height = rng.gen_range(0.0, 50.0);
        let width = rng.gen_range(0.0, 50.0);

        let r = ((xmin, ymin), (xmin + width, ymin + height)).into_region();
        tree.insert(r, 11)?;
    }

    tree.validate_consistency(root);

    dbg!(&tree);

    Ok(())
}
