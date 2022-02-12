use anyhow::Result;
use rand::Rng;

use spaceindex::rtree::rendering::image::TreeRenderOptions;
use spaceindex::{Rect, RTree};

const RENDER_WIDTH: u32 = 4000;
const RENDER_HEIGHT: u32 = 2000;

fn main() -> Result<()> {
    let mut tree = RTree::new();
    let mut rng = rand::thread_rng();

    for _ in 0..5_000 {
        let x = rng.gen_range(0.0..=4000.0);
        let y = rng.gen_range(0.0..=2000);

        // pick a length
        let length = rng.gen_range(15.0..=45.0);

        // insert this region into our tree
        let rect = Rect::new((x, y), (x+length, y+length));
        tree.insert(rect, 0)?;
    }

    TreeRenderOptions::new(RENDER_WIDTH, RENDER_HEIGHT).draw_tree(
        "Tree.png",
        &tree,
        tree.root_index(),
    );

    Ok(())
}
