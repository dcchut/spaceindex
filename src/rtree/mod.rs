use std::collections::HashSet;

use generational_arena::{Arena, Index};

use crate::geometry::{Region, Shapelike, ShapelikeError};

mod node;
pub mod rendering;

pub use node::Node;

// completely scientific values
const MIN_CHILDREN: usize = 2;
const MAX_CHILDREN: usize = 8;

#[derive(Debug)]
pub struct RTree {
    /// Nodes are stored in a generational arena.
    nodes: Arena<Node>,

    /// The index of the root node of this tree.
    root: Index,
}

impl RTree {
    /// Creates a new [`RTree`] with the given number of dimensions.
    ///
    /// # Example
    /// ```rust
    /// use spaceindex::rtree::RTree;
    /// use spaceindex::geometry::IntoRegion;
    ///
    /// let mut tree = RTree::new(2);
    /// tree.insert(((0.0, 0.0), (2.0, 4.0)).into_region(), 1);
    ///
    /// # tree.validate_consistency();
    /// ```
    pub fn new(dimension: usize) -> Self {
        let node = Node::new_internal_node(Region::infinite(dimension), None);
        let mut nodes = Arena::new();
        let root_index = nodes.insert(node);

        let root_child_node =
            Node::new_internal_node(Region::infinite(dimension), Some(root_index));
        let root_child_index = nodes.insert(root_child_node);

        Self {
            nodes,
            root: root_child_index,
        }
    }

    /// Attempts to insert a given object into the tree.
    ///
    /// # Errors
    /// This function will return an error if `region` does not have the same dimension as this tree.
    ///
    /// # Example
    /// ```rust
    /// use spaceindex::rtree::RTree;
    /// use spaceindex::geometry::IntoRegion;
    ///
    /// let mut tree = RTree::new(2);
    /// tree.insert(((-1.0, 0.0), (3.0, 3.0)).into_region(), 0);
    ///
    /// # tree.validate_consistency();
    /// ```
    pub fn insert(&mut self, region: Region, object: usize) -> Result<(), ShapelikeError> {
        // The internal `root` node always contains everything.
        self.insert_at_node(region, object, self.root)
    }

    /// Inserts a node into our tree at the given position.
    fn _insert(&mut self, region: Region, index: Index) {
        // Parent node should always contain the input region
        assert_eq!(
            self.nodes[index].region().contains_region(&region),
            Ok(true)
        );

        // add a new leaf as a child of this node
        let leaf_node = Node::new_leaf(region, Some(index));
        let leaf_index = self.nodes.insert(leaf_node);

        // This call is safe as `leaf_index` has their parent attribute set to `Some(index)`, i.e.
        // the index of the current node, and the child node is contained in this tree.
        unsafe {
            self.get_node_mut(index).add_child_unsafe(leaf_index);
        }

        // If this node node has too many children, split it.
        if self.get_node(index).child_count() >= MAX_CHILDREN {
            self.split_node(index);
        }
    }

    /// Recursively searches for the internal node whose minimum bounding region contains `region`.
    fn insert_at_node(
        &mut self,
        region: Region,
        object: usize,
        index: Index,
    ) -> Result<(), ShapelikeError> {
        // current node under consideration
        let node = &self.nodes[index];

        // If we've reached a node with leaf children, insert here.
        if node.has_leaf_child(self) || !node.has_children() {
            // If we've reached a leaf node, insert this as a leaf of the parent?
            self._insert(region, index);
            return Ok(());
        }

        // Does any child of this node have an MBR containing our input region?
        let mut child_containing_region = None;

        'mbr_search: for (child_index, child_node) in self.child_iter(index) {
            if child_node.region().contains_region(&region)? {
                child_containing_region = Some(child_index);
                break 'mbr_search;
            }
        }

        // If we found a child node containing our region, recurse into that node
        if let Some(child_index) = child_containing_region {
            return self.insert_at_node(region, object, child_index);
        }

        // Otherwise there is no child MBR containing our input `region`.  Thus find
        // the bounding box in this node such that enlarging it to contain
        // `minimum_bounding_region` will add the least amount of area.
        if let Some((_, combined_region, child_index)) = self
            .child_iter(index)
            .map(|(child_index, child_node)| {
                let initial_area = child_node.region().get_area();
                // TODO: figure out a better error handling path here (perhaps use `filter_map`)
                let combined_region = child_node
                    .region()
                    .combine_region(&region)
                    .expect("Failed to combine regions");
                (
                    combined_region.get_area() - initial_area,
                    combined_region,
                    child_index,
                )
            })
            .min_by(|(left_change, _, _), (right_change, _, _)| {
                // TODO: this should be fine, but worth investigating.
                f64::partial_cmp(left_change, right_change).unwrap()
            })
        {
            // Enlarge `child_index`'s bounding box.  This call is safe as `combined_region`
            // is enlarged from the MBR of the child node.
            unsafe {
                self.get_node_mut(child_index)
                    .set_minimum_bounding_region_unsafe(combined_region);
            }

            // Since the enlarged bounding box now contains our object, recurse into that subtree
            return self.insert_at_node(region, object, child_index);
        }

        panic!("something weird happened");
    }

    /// Given a set of nodes, finds the pair of nodes whose combined bounding box is
    /// the worst.  To be concrete, we find the pair whose combined bounding box
    /// has the maximum difference to the sum of the areas of the bounding boxes
    /// for the original two nodes.
    fn find_worst_pair(&self, leaves: &[Index]) -> (usize, usize) {
        // This would be silly.
        debug_assert!(leaves.len() >= 2);

        let mut worst_pair = None;
        let mut worst_area = std::f64::MIN;

        // find the two leaves of this node that would be the most terrible together
        for (l1_index, node1) in leaves.iter().enumerate() {
            let r1 = &self.nodes[*node1].region();
            let a1 = r1.get_area();

            for (l2_index, node2) in leaves.iter().enumerate().skip(l1_index + 1) {
                let r2 = &self.nodes[*node2].region();
                let a2 = r2.get_area();

                // combine these two regions together
                let combined_region = r1.combine_region(r2).expect("failed to combine regions");
                let combined_area = combined_region.get_area() - a1 - a2;

                if combined_area > worst_area {
                    worst_pair = Some((l1_index, l2_index));
                    worst_area = combined_area;
                }
            }
        }

        worst_pair.unwrap()
    }

    /// Splits a vector of nodes into two groups using the QuadraticSplit algorithm.
    fn quadratic_partition(
        &self,
        children: Vec<Index>,
    ) -> (Vec<Index>, Vec<Index>, Region, Region) {
        let (ix1, ix2) = self.find_worst_pair(&children);

        let mut unpicked_children: HashSet<usize> = (0..children.len()).collect();
        unpicked_children.remove(&ix1);
        unpicked_children.remove(&ix2);

        // Keep track of nodes in the first group
        let mut group1 = Vec::with_capacity(MAX_CHILDREN - MIN_CHILDREN);
        group1.push(ix1);

        // Keep track of the minimum bounding regions for the first and second group
        let mut group1_mbr = self.nodes[children[ix1]].region().clone();
        let mut group2_mbr = self.nodes[children[ix2]].region().clone();

        // Partition the nodes into two groups.  The basic strategy is that at each stepp
        // we find the unpicked node
        // If one of the groups gets too large, stop.
        while !unpicked_children.is_empty()
            && group1.len() < MAX_CHILDREN - MIN_CHILDREN
            && (children.len() - group1.len() - unpicked_children.len())
                < MAX_CHILDREN - MIN_CHILDREN
        {
            let mut best_d = std::f64::MAX;
            let mut best_index = None;

            for &index in unpicked_children.iter() {
                let g1r = group1_mbr
                    .combine_region(self.nodes[children[index]].region())
                    .expect("failed to combine leaves");
                let g2r = group2_mbr
                    .combine_region(self.nodes[children[index]].region())
                    .expect("failed to combine leaves");

                let d1 = g1r.get_area() - group1_mbr.get_area();
                let d2 = g2r.get_area() - group2_mbr.get_area();

                if d1 < d2 && d1 < best_d {
                    best_index = Some((index, 1));
                    best_d = d1;
                } else if d2 < d1 && d2 < best_d {
                    best_index = Some((index, 2));
                    best_d = d2;
                } else if (d1 - d2).abs() < std::f64::EPSILON && d1 < best_d {
                    // in case of ties, assign to MBR with smallest area
                    if group1_mbr.get_area() < group2_mbr.get_area() {
                        best_index = Some((index, 1));
                    } else {
                        best_index = Some((index, 2));
                    }
                    best_d = d1;
                }
            }

            let (best_index, side) = best_index.unwrap();
            unpicked_children.remove(&best_index);

            if side == 1 {
                // add to group 1
                group1.push(best_index);
                group1_mbr.combine_region_in_place(self.nodes[children[best_index]].region());
            } else {
                group2_mbr.combine_region_in_place(self.nodes[children[best_index]].region());
            }
        }

        if !unpicked_children.is_empty() {
            if group1.len() < MIN_CHILDREN {
                // rest of the unpicked children go in group 1
                for child_index in unpicked_children {
                    group1_mbr.combine_region_in_place(self.nodes[children[child_index]].region());
                    group1.push(child_index);
                }
            } else {
                // rest of the unpicked children go in group 2
                for child_index in unpicked_children {
                    group2_mbr.combine_region_in_place(self.nodes[children[child_index]].region());
                }
            }
        }

        let (group1, group2) = Self::assemble(children, group1.into_iter().collect());

        (group1, group2, group1_mbr, group2_mbr)
    }

    /// Splits a vector `v` into two vectors, with the first vector containing all elements
    /// of `v` whose indexes are in `left_indexes`, and the second vector containing the rest.
    fn assemble<S>(v: Vec<S>, left_indexes: HashSet<usize>) -> (Vec<S>, Vec<S>) {
        let mut left = Vec::with_capacity(left_indexes.len());
        let mut right = Vec::with_capacity(v.len() - left_indexes.len());

        for (index, vs) in v.into_iter().enumerate() {
            if left_indexes.contains(&index) {
                left.push(vs);
            } else {
                right.push(vs);
            }
        }

        (left, right)
    }

    /// Collects an iterator of children into the `children` vec of the node corresponding to `index`,
    /// ensuring that the `parent` attribute of the corresponding node in the tree is set appropriately.
    ///
    /// # Panics
    /// This function will panic if:
    /// - The node correspoding to `index` already has children, or
    /// - `index` does not refer to a node in `self`, or
    /// - Any index in `children` does not refer to a node in `self`.
    /// - `index` appears in `children`
    pub(crate) fn set_children_safe(
        &mut self,
        index: Index,
        children: impl IntoIterator<Item = Index>,
    ) {
        // get a mutable reference to the current node
        let node = unsafe { (&mut self.nodes[index] as *mut Node).as_mut().unwrap() };

        // Make sure we don't have any children
        assert!(!node.has_children());

        // Make sure `index` exists in our tree
        assert!(self.nodes.contains(index));

        for child_index in children {
            assert_ne!(index, child_index);

            // set the parent of the child node to be `Some(index)`.
            self.nodes[child_index].set_parent(index);

            // This is fine because `child_index` refers to a node in this tree whose parent
            // attribute is set to `Some(index)`, as required.
            unsafe {
                node.add_child_unsafe(child_index);
            }
        }
    }

    /// Splits the overfull node corresponding to `index`.
    fn split_node(&mut self, index: Index) {
        // Get all of the children of the current node
        let children = self.get_node_mut(index).clear_children();

        // Partition the leave indexes using the QuadraticSplit strategy
        let (left, right, left_mbr, right_mbr) = self.quadratic_partition(children);

        // check that everything has the correct size
        debug_assert!(left.len() >= MIN_CHILDREN);
        debug_assert!(right.len() >= MIN_CHILDREN);

        // If we're splitting the root node, collect all children of the root node into two groups
        // which will be our new root children.
        //      ( hidden )          (hidden)
        //          |                  |
        //         root     =>        root
        //       /  |  \            /      \
        //      /   |   \         left    right
        if index == self.root {
            // insert a new left node
            let left_node = Node::new_internal_node(left_mbr, Some(index));
            let left_index = self.nodes.insert(left_node);
            self.set_children_safe(left_index, left);

            // insert a new right node
            let right_node = Node::new_internal_node(right_mbr, Some(index));
            let right_index = self.nodes.insert(right_node);
            self.set_children_safe(right_index, right);

            // This call is safe because:
            // - The current node has no children,
            // - The nodes corresponding to `left_index` and `right_index` both have their `parent`
            //   attribute set to `Some(index)`, i.e. the index of the current node.
            unsafe {
                // Add the left and right nodes as children of the current node.
                self.get_node_mut(index)
                    .set_children_unsafe(vec![left_index, right_index]);
            }
        } else {
            // Otherwise we apply the transformation:
            //
            //     parent            parent
            //       |               /   \
            //      node    =>    node    right
            //     / | \          /  \    /  \
            //    /  |  \        /    \  /    \
            //
            let parent = self.get_node(index).get_parent().unwrap();

            // the current node will become our new left node
            let left_node = self.get_node_mut(index);

            // This is safe as `left_node` has no children, and all of the children
            // in `left` already have their parent attribute set to `Some(index)`.
            // Finally, all of the children are contained in `left_mbr` by its construction.
            unsafe {
                left_node.set_minimum_bounding_region_unsafe(left_mbr);
                left_node.set_children_unsafe(left);
            };

            // make a new empty right node
            let right_index = self
                .nodes
                .insert(Node::new_internal_node(right_mbr, Some(parent)));

            // add the right as children (safely) of the right node
            self.set_children_safe(right_index, right.iter().cloned());

            // This `unsafe` call is fine here because `right_index` refers to a node in this tree
            // whose parent attribute is set to `Some(parent)`.
            unsafe { self.get_node_mut(parent).add_child_unsafe(right_index) };

            if self.nodes[parent].child_count() >= MAX_CHILDREN {
                self.split_node(parent);
            }
        }
    }

    /// Validates the consistency of the tree.  In particular, this function checks that:
    ///
    /// - Every child is contained in the minimum bounding region of its parent, and
    /// - The total number of descendants of the root node is equal to the number
    ///   of nodes in the tree minus two.
    pub fn validate_consistency(&self) {
        let mut node_counter = 0;

        self._validate_consistency(self.root, &mut node_counter);

        // check we have the expected number of nodes.  The +1 is for the hidden super-root
        // node which we won't talk about.
        assert_eq!(node_counter + 1, self.nodes.len());
    }

    /// Recursively validates that the children of each node are contained in the MBR
    /// of their parent.
    fn _validate_consistency(&self, index: Index, node_counter: &mut usize) {
        let node = &self.nodes[index];

        // increment the node counter
        *node_counter += 1;

        // are all children of this node contained in the MBR of this node?
        for (_, child_node) in self.child_iter(index) {
            assert_eq!(node.region().contains_region(child_node.region()), Ok(true));
        }

        // validate all children of this node
        for child_index in node.child_index_iter() {
            self._validate_consistency(child_index, node_counter);
        }
    }

    /// Returns an iterator over pairs `(Index, &Node)` corresponding to the nodes in this tree.
    pub fn node_iter(&self) -> impl Iterator<Item = (Index, &Node)> {
        self.nodes.iter()
    }

    /// Returns an iterator of pairs `(Index, &Node)` of children of the node corresponding to the
    /// given `index`.
    ///
    /// # Panics
    /// This function will panic if `index` does not refer to a node in this tree.
    pub fn child_iter(&self, index: Index) -> impl Iterator<Item = (Index, &Node)> + '_ {
        self.nodes[index]
            .child_index_iter()
            .map(move |index| (index, self.get_node(index)))
    }

    /// Returns a reference to the [`Node`] with index `index`.
    ///
    /// # Panics
    /// This function will panic if `index` does not refer to a node in this tree.
    pub fn get_node(&self, index: Index) -> &Node {
        &self.nodes[index]
    }

    /// Returns a mutable reference to the [`Node`] with index `index.
    ///
    /// # Panics
    /// This function will panic if `index` does not refer to a node in this tree.
    pub fn get_node_mut(&mut self, index: Index) -> &mut Node {
        &mut self.nodes[index]
    }

    /// Returns a reference to the root [`Node`] in this tree.
    pub fn root_node(&self) -> &Node {
        &self.nodes[self.root]
    }

    /// Returns the index of the root node in this tree.
    pub fn root_index(&self) -> Index {
        self.root
    }

    /// Returns a vector of pairs `(Index, Index)` corresponding to all edges in this tree.
    /// The edges are always of the form `(Parent, Child)`.
    pub fn collect_edges(&self) -> Vec<(Index, Index)> {
        // collect all edges in the tree
        let mut edges = Vec::new();
        self._collect_edges(&mut edges, self.root);

        edges
    }

    /// Recursively extends `buffer` with all children of the given node.
    fn _collect_edges(&self, buffer: &mut Vec<(Index, Index)>, index: Index) {
        // extend buffer with all edges from the current node
        let node = self.get_node(index);

        for child_index in node.child_index_iter() {
            buffer.push((index, child_index));
            self._collect_edges(buffer, child_index);
        }
    }
}
