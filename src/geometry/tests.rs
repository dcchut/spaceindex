use crate::geometry::{Point, LineSegment, Shapelike, Region};

#[test]
fn test_line_intersections() {
    let p1 = Point::new(vec![1.0, 0.0]);
    let p2 = Point::new(vec![3.0, 2.0]);
    let p3 = Point::new(vec![2.0, 0.0]);
    let p3a = Point::new(vec![2.0, 3.0]);
    let p4= Point::new(vec![2.0, 4.0]);
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