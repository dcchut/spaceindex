use anyhow::Result;
use rand::Rng;

use spaceindex::rtree::rendering::image::TreeRenderOptions;
use spaceindex::{Rect, RTree};

const RENDER_WIDTH: u32 = 1000;
const RENDER_HEIGHT: u32 = 1000;

fn main() -> Result<()> {
    let mut tree = RTree::new();
    let mut rng = rand::thread_rng();

    for _ in 0..35 {
        // pick a random x-coordinate
        let x = rng.gen_range(0.0..=650.0);
        let y = rng.gen_range(0.0..=50.0);

        // pick a length
        let length = rng.gen_range(15.0..=150.0);

        // insert this region into our tree
        let rect = Rect::new((x, y), (x+length, y+length));
        tree.insert(rect, 0)?;
    }

    TreeRenderOptions::new(RENDER_WIDTH, RENDER_HEIGHT)
        .without_threshold()
        .draw_tree("Tree.png", &tree, tree.root_index());

    Ok(())
}
