use anyhow::Result;
use rand::Rng;

use spaceindex::rtree::rendering::graphviz::render_gviz;
use spaceindex::rtree::rendering::image::TreeRenderOptions;
use spaceindex::rtree::RTree;

const RENDER_WIDTH: u32 = 1920;
const RENDER_HEIGHT: u32 = 1080;
const MAX_REGION_SIDE_LENGTH: f64 = 300.0;

fn main() -> Result<()> {
    // Generate some random points to fill in our tree
    let mut tree = RTree::new(2);
    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let xmin = rng.gen_range(0.0, RENDER_WIDTH as f64);
        let ymin = rng.gen_range(0.0, RENDER_HEIGHT as f64);
        let height = rng.gen_range(0.1, MAX_REGION_SIDE_LENGTH);
        let width = rng.gen_range(0.1, MAX_REGION_SIDE_LENGTH);

        let r = ((xmin, ymin), (xmin + width, ymin + height));
        tree.insert(r, 11)?;
    }

    tree.validate_consistency();

    // render a graphviz representation of the RTree
    render_gviz(&tree, "tree.dot");

    let mut render_options = TreeRenderOptions::new(1920, 1080);

    // draw out each child of the root node layer-by-layer
    for (ix, child_index) in tree.root_node().child_index_iter().enumerate() {
        // draw it all out
        for threshold in 0..5 {
            render_options.with_threshold(threshold).draw_tree(
                format!("Tree_C{}_T{}.png", ix, threshold),
                &tree,
                child_index,
            );
        }
    }

    // do a global rendering
    render_options
        .without_threshold()
        .draw_tree("Tree.png", &tree, tree.root_index());

    println!(
        "{} children of the root node",
        tree.root_node().child_count()
    );

    Ok(())
}
