use crate::geometry::{Shape, Region};
use generational_arena::{Arena, Index};

struct RTree {
    root_id: Index,
    header_id: Index,
    fill_factor: f64,
    index_capacity: usize,
    leaf_capacity: usize,

    /// The R*-Tree `p` constant, for calculating nearly minimum overlap cost.
    /// [Beckmann, Kriegel, Schneider, Seeger 'The R*-tree: An efficient and Robust Access Method
    /// for Points and Rectangles', Section 4.1]
    near_minimum_overlap_factor: usize,

    /// The R*-Tree `m` constant, for calculating splitting distributions.
    /// [Beckmann, Kriegel, Schneider, Seeger 'The R*-tree: An efficient and Robust Access Method
    /// for Points and Rectangles', Section 4.2]
    split_distribution_factor: f64,

    /// The R*-Tree `p` constant, for removing entries at reinserts.
    /// [Beckmann, Kriegel, Schneider, Seeger 'The R*-tree: An efficient and Robust Access Method
    ///  for Points and Rectangles', Section 4.3]
    reinsert_factor: f64,
    dimension: usize,
    infinite_region: Region,
    tight_minimum_bounding_regions: bool,

    /// The nodes are stored in a generational arena.
    nodes_arena: Arena<Node>,
}


#[derive(Clone)]
struct Node {
    // keep a ref to the tree?
    // or perhaps all operations should be done at the tree level...

    /// The level of the node in the tree
    level: usize,

    /// The unique ID of this node
    identifier: Index,

    /// The number of children pointed by this node
    children: usize,

    /// The node capacity
    capacity: usize,

    /// The minimum bounding region enclosing all data contained in the node
    minimum_bounding_region: Region,
}

impl Node {
    pub fn get_identifier(&self) -> Index {
        self.identifier
    }

    pub fn get_shape(&self) -> Shape {
        unimplemented!()
    }

    pub fn get_children_count(&self) -> usize {
        self.children
    }

    pub fn get_child_identifier(&self, index: usize) -> Index {
        unimplemented!()
    }

    pub fn get_child_shape(&self, index: usize) -> Shape {
        unimplemented!()
    }

    pub fn get_child_data(&self, index: usize) {
        // TODO: figure out data REPR
    }

    pub fn get_level(&self) -> usize {
        self.level
    }

    pub fn is_index(&self) -> bool {
        true
    }

    pub fn is_leaf(&self) -> bool {
        true
    }

    fn insert_entry(&mut self) {
        // TODO
    }

    fn delete_entry(&mut self, index: usize) {
        // TODO
    }

    fn insert_data(&mut self) {
        // TODO
    }

    fn reinsert_data(&mut self) {
        // TODO
    }

    fn rtree_split(&mut self) {
        // TODO
    }

    fn rstar_split(&mut self) {
        // TODO
    }
}
