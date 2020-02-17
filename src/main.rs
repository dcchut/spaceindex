use std::error::Error;

use generational_arena::Index;
use image::Rgb;
use image::RgbImage;
use imageproc::drawing::{draw_line_segment_mut, Canvas};
use rand::Rng;

use spaceindex::geometry::{IntoRegion, Region};
use spaceindex::rtree::{Node, RTree};

fn main() -> Result<(), Box<dyn Error>> {
    let mut tree = RTree::new(2);

    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let xmin = rng.gen_range(0.0, 1890.0);
        let ymin = rng.gen_range(0.0, 1000.0);
        let height = rng.gen_range(0.1, 300.0);
        let width = rng.gen_range(0.1, 300.0);

        let r = ((xmin, ymin), (xmin + width, ymin + height)).into_region();
        tree.insert(r, 11)?;
    }

    tree.validate_consistency();

    let gviz = render_gviz(&tree);

    // write the graphviz representation of the tree to file
    std::fs::write("tree.dot", gviz).expect("failed to write to tree.dot");

    // draw out each child of the root node layer-by-layer
    for (ix, (_, child_node)) in tree.nodes[tree.root].child_iter(&tree).enumerate() {
        // draw it all out
        for threshold in 0..5 {
            draw(
                &tree,
                child_node,
                threshold,
                &format!("tree_c{}_t{}.png", ix, threshold),
                true,
            );
        }
    }

    // do a global rendering
    draw(&tree, &tree.nodes[tree.root], 1_000_000, "tree.png", false);

    println!(
        "{} children of the root node",
        tree.nodes[tree.root].child_count()
    );

    Ok(())
}

fn draw(tree: &RTree, node: &Node, threshold: usize, filename: &str, hard: bool) {
    let mut img = RgbImage::new(1920, 1080);
    render_node(&mut img, tree, node, 0, threshold, hard);
    img.save(filename).unwrap();
}

fn render_gviz(tree: &RTree) -> String {
    // render a graphviz file?
    let mut gviz = String::new();
    gviz.push_str("digraph rtree {\n");

    // first list all of the node indexes
    for (index, node) in tree.nodes.iter() {
        // dont render the hidden root node
        if node.parent.is_none() {
            continue;
        }
        let child_count = node.child_count();

        gviz.push_str(&format!(
            "\t{}[label=\"{},{}\"];\n",
            index.into_raw_parts().0,
            child_count,
            if node.is_leaf() { "leaf" } else { "internal" }
        ));
    }

    // now recurse
    _gviz(&mut gviz, &tree, tree.root);

    gviz.push_str("}\n");

    gviz
}

fn _gviz(buffer: &mut String, tree: &RTree, index: Index) {
    for (child_index, _) in tree.nodes[index].child_iter(tree) {
        buffer.push_str(&format!(
            "\t{} -> {};\n",
            index.into_raw_parts().0,
            child_index.into_raw_parts().0
        ));

        _gviz(buffer, tree, child_index);
    }
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
        Rgb([40u8, 180u8, 120u8]),
        Rgb([59u8, 49u8, 118u8]),
        Rgb([170u8, 108u8, 57u8]),
        Rgb([86u8, 119u8, 20u8]),
        Rgb([70u8, 50u8, 160u8]),
    ];

    draw_line_segment_mut(
        canvas,
        (x0 as f32, y0 as f32),
        (x1 as f32, y1 as f32),
        colors[level % colors.len()],
    );
}

const BUFFER_WIDTH: f64 = 1.0;

fn draw_mbr<C: Canvas<Pixel = Rgb<u8>>>(canvas: &mut C, mbr: &Region, level: usize) {
    let (x0, x1) = mbr.coordinates[0];
    let (y0, y1) = mbr.coordinates[1];

    for thickness in 0..3 {
        draw_line(
            canvas,
            level,
            x0 + (BUFFER_WIDTH * level as f64) + thickness as f64,
            y0 + (BUFFER_WIDTH * level as f64) + thickness as f64,
            x0 + (BUFFER_WIDTH * level as f64) + thickness as f64,
            y1 - (BUFFER_WIDTH * level as f64) - thickness as f64,
        );
        draw_line(
            canvas,
            level,
            x0 + (BUFFER_WIDTH * level as f64) + thickness as f64,
            y1 - (BUFFER_WIDTH * level as f64) - thickness as f64,
            x1 - (BUFFER_WIDTH * level as f64) - thickness as f64,
            y1 - (BUFFER_WIDTH * level as f64) - thickness as f64,
        );
        draw_line(
            canvas,
            level,
            x1 - (BUFFER_WIDTH * level as f64) - thickness as f64,
            y1 - (BUFFER_WIDTH * level as f64) - thickness as f64,
            x1 - (BUFFER_WIDTH * level as f64) - thickness as f64,
            y0 + (BUFFER_WIDTH * level as f64) + thickness as f64,
        );
        draw_line(
            canvas,
            level,
            x1 - (BUFFER_WIDTH * level as f64) - thickness as f64,
            y0 + (BUFFER_WIDTH * level as f64) + thickness as f64,
            x0 + (BUFFER_WIDTH * level as f64) + thickness as f64,
            y0 + (BUFFER_WIDTH * level as f64) + thickness as f64,
        );
    }
}

fn render_node(
    canvas: &mut RgbImage,
    tree: &RTree,
    node: &Node,
    level: usize,
    threshold: usize,
    hard: bool,
) {
    if hard && level > threshold {
        return;
    }

    // now do all children
    for (_, child_node) in node.child_iter(tree) {
        render_node(canvas, tree, child_node, level + 1, threshold, hard);
    }

    if level == threshold || !hard {
        draw_mbr(canvas, &node.minimum_bounding_region, level);
    }
}
