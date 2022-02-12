use anyhow::Result;
use rand::Rng;

use spaceindex::rtree::rendering::image::TreeRenderOptions;
use spaceindex::{Rect, RTree};


const RENDER_WIDTH: u32 = 1920;
const RENDER_HEIGHT: u32 = 1080;
const MAX_REGION_SIDE_LENGTH: f64 = 300.0;

fn main() -> Result<()> {
    // Generate some random points to fill in our tree
    let mut tree = RTree::new();
    let mut rng = rand::thread_rng();

    // create a <really really big> tree.
    for _ in 0..500_000 {
        let xmin = rng.gen_range(0.0..=RENDER_WIDTH as f64);
        let ymin = rng.gen_range(0.0..=RENDER_HEIGHT as f64);
        let height = rng.gen_range(0.1..=MAX_REGION_SIDE_LENGTH);
        let width = rng.gen_range(0.1..=MAX_REGION_SIDE_LENGTH);

        let rect = Rect::new( (xmin, ymin), (xmin + width, ymin + height));
        tree.insert(rect, 11)?;
    }

    tree.validate_consistency();

    // do a global rendering
    TreeRenderOptions::new(RENDER_WIDTH, RENDER_HEIGHT)
        .without_threshold()
        .draw_tree("Tree.png", &tree, tree.root_index());

    Ok(())
}
