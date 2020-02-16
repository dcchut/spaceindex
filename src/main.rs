use image::Rgb;
use image::RgbImage;
use imageproc::drawing::{draw_line_segment_mut, Canvas};
use rand::Rng;
use spaceindex::geometry::{IntoRegion, Region};
use spaceindex::rtree::{Node, RTree};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let (mut tree, root) = RTree::new(2);

    // insert 50 random positions
    let mut rng = rand::thread_rng();

    let mut size_f = 600.0;
    for z in 0..40 {
        let xmin = rng.gen_range(0.0, 1400.0);
        let ymin = rng.gen_range(0.0, 500.0);
        let height = rng.gen_range(15.0, size_f);
        let width = rng.gen_range(15.0, size_f);

        // gradually get smaller
        if z >= 7 {
            size_f = 100.0;
        } else if z >= 20 {
            size_f = 40.0;
        }

        let r = ((xmin, ymin), (xmin + width, ymin + height)).into_region();
        tree.insert(r, 11)?;
    }

    // draw it all out
    let mut img = RgbImage::new(1920, 1080);

    render_node(&mut img, &tree, &tree.nodes[tree.root], 0);

    println!("{} root children", &tree.nodes[tree.root].child_count());

    img.save("output.png").unwrap();

    Ok(())
}

fn draw_line<C: Canvas<Pixel = Rgb<u8>>>(
    canvas: &mut C,
    level: usize,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
) {
    let colors = vec![
        Rgb([128u8, 21u8, 21u8]),
        Rgb([86u8, 119u8, 20u8]),
        Rgb([170u8, 108u8, 57u8]),
        Rgb([170u8, 151u8, 57u8]),
        Rgb([59u8, 49u8, 118u8]),
    ];

    draw_line_segment_mut(
        canvas,
        (x0 as f32, y0 as f32),
        (x1 as f32, y1 as f32),
        colors[level % colors.len()].clone(),
    );
}

const BUFFER_WIDTH: f64 = 1.0;

fn draw_mbr<C: Canvas<Pixel = Rgb<u8>>>(canvas: &mut C, mbr: &Region, level: usize) {
    let (x0, x1) = mbr.coordinates[0].clone();
    let (y0, y1) = mbr.coordinates[1].clone();

    draw_line(
        canvas,
        level,
        x0 + (BUFFER_WIDTH * level as f64),
        y0 + (BUFFER_WIDTH * level as f64),
        x0 + (BUFFER_WIDTH * level as f64),
        y1 - (BUFFER_WIDTH * level as f64),
    );
    draw_line(
        canvas,
        level,
        x0 + (BUFFER_WIDTH * level as f64),
        y1 - (BUFFER_WIDTH * level as f64),
        x1 - (BUFFER_WIDTH * level as f64),
        y1 - (BUFFER_WIDTH * level as f64),
    );
    draw_line(
        canvas,
        level,
        x1 - (BUFFER_WIDTH * level as f64),
        y1 - (BUFFER_WIDTH * level as f64),
        x1 - (BUFFER_WIDTH * level as f64),
        y0 + (BUFFER_WIDTH * level as f64),
    );
    draw_line(
        canvas,
        level,
        x1 - (BUFFER_WIDTH * level as f64),
        y0 + (BUFFER_WIDTH * level as f64),
        x0 + (BUFFER_WIDTH * level as f64),
        y0 + (BUFFER_WIDTH * level as f64),
    );
}

fn render_node(
    canvas: &mut RgbImage,
    tree: &RTree,
    node: &Node,
    level: usize,
) {
    // and all leaves
    for leaf in node.leaves.iter() {
        draw_mbr(canvas, &leaf.region, level + 1);
    }

    // now do all children
    for (_, child_node) in node.child_iter(tree) {
        render_node(canvas, tree, child_node, level + 1);
    }

    draw_mbr(canvas, &node.minimum_bounding_region, level);
}
