use test::Bencher;

use rand::Rng;

use crate::rtree::RTree;
use crate::{point, Coordinate, Rect};

#[bench]
fn bench_large_tree_lookups(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    // generate 20,000 regions and insert them into our tree
    let mut tree = RTree::new();

    for _ in 0..200_000 {
        let xmin = rng.gen_range(0.0..=10_000.0);
        let width = rng.gen_range(0.0..=5.0);
        let ymin = rng.gen_range(0.0..=10_000.0);
        let height = rng.gen_range(0.0..=5.0);

        let rect = Rect::new(
            Coordinate { x: xmin, y: ymin },
            Coordinate {
                x: xmin + width,
                y: ymin + height,
            },
        );
        tree.insert(rect, ()).unwrap();
    }

    // generate 500_000 lookup points
    let mut lookup_points = Vec::new();

    for _ in 0..500 {
        let x = rng.gen_range(0.0..=11_000.0);
        let y = rng.gen_range(0.0..=11_000.0);

        lookup_points.push(point! { x: x, y: y });
    }

    b.iter(|| {
        lookup_points
            .iter()
            .map(|point| tree.point_lookup(*point))
            .collect::<Vec<_>>()
    });
}
