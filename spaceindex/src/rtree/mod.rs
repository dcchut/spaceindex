use std::collections::HashSet;

use generational_arena::Arena;
pub use generational_arena::Index;

pub use node::Node;

use crate::geometry::{
    IntoPoint, IntoRegion, LineSegment, Point, Region, Shape, Shapelike, ShapelikeError,
};

mod node;
pub mod rendering;
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct RTree<ND> {
    /// Nodes are stored in a generational arena.
    nodes: Arena<Node<ND>>,

    /// The index of the root node of this tree.
    root: Index,

    /// The minimum number of children a node can have
    min_children: usize,

    /// The maximum number of children a node can have
    max_children: usize,
}

impl<ND> RTree<ND> {
    /// Creates a new [`RTree`] with the given number of dimensions.
    ///
    /// # Example
    /// ```rust
    /// use spaceindex::rtree::RTree;
    ///
    /// let mut tree = RTree::new(2);
    /// tree.insert(((0.0, 0.0), (2.0, 4.0)), 1);
    ///
    /// # tree.validate_consistency();
    /// ```
    pub fn new(dimension: usize) -> Self {
        let node = Node::new_internal_node(Region::infinite(dimension), None);
        let mut nodes = Arena::new();
        let root_index = nodes.insert(node);

        // TODO: figure out a better way to pass through min/max children here (maybe some sort of builder?)
        Self {
            nodes,
            root: root_index,
            min_children: 2,
            max_children: 8,
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
    ///
    /// let mut tree = RTree::new(2);
    /// tree.insert(((-1.0, 0.0), (3.0, 3.0)), 0);
    ///
    /// # tree.validate_consistency();
    /// ```
    pub fn insert<'a, IR: IntoRegion<'a>>(
        &mut self,
        region: IR,
        data: ND,
    ) -> Result<(), ShapelikeError> {
        let region = region.into_region().into_owned();

        // If we only have the root node, then set the MBR of the root node to be our input region.
        if self.nodes.len() == 1 {
            // This call is fine because the root node currently has no children.
            self.get_node_mut(self.root)
                .set_minimum_bounding_region_unsafe(region.clone());
        } else {
            // Otherwise extend the MBR of the root node by the input region.
            // This call is fine because the root node has no parents, so we don't need to
            // worry about having inconsistent minimum bounding regions.
            self.get_node_mut(self.root).combine_region_unsafe(&region);
        }

        // The internal `root` node always contains everything.
        self.insert_at_node(region, data, self.root)
    }

    /// Inserts a node with data `data` into the tree at the given index.  This function is unsafe
    /// as using it incorrectly can use to inconsistent data.  A key assumption here is that
    /// `region` must be contained in the minimum bounding region of the node corresponding to `index`.
    fn _insert(&mut self, region: Region, data: ND, index: Index) {
        // Parent node should always contain the input region
        assert_eq!(
            self.nodes[index].get_region().contains_region(&region),
            Ok(true)
        );

        // add a new leaf as a child of this node
        let leaf_node = Node::new_leaf(region, data, Some(index));
        let leaf_index = self.nodes.insert(leaf_node);

        // This call is safe as `leaf_index` has their parent attribute set to `Some(index)`, i.e.
        // the index of the current node, and the child node is contained in this tree.
        self.get_node_mut(index).add_child_unsafe(leaf_index);

        // If this node node has too many children, split it.
        if self.get_node(index).child_count() >= self.max_children {
            self.split_node(index);
        }
    }

    /// Recursively searches for the internal node whose minimum bounding region contains `region`.
    fn insert_at_node(
        &mut self,
        region: Region,
        data: ND,
        index: Index,
    ) -> Result<(), ShapelikeError> {
        // current node under consideration
        let node = &self.nodes[index];

        // If we've reached a node with leaf children, insert here.
        if self.has_child_leaf(index) || !node.has_children() {
            // If we've reached a leaf node, insert this as a leaf of the parent
            // This call is safe as `region` is guaranteed to be contained in the minimum
            // bounding region of this node.
            self._insert(region, data, index);
            return Ok(());
        }

        // Does any child of this node have an MBR containing our input region?
        let mut child_containing_region = None;

        'mbr_search: for (child_index, child_node) in self.child_iter(index) {
            if child_node.get_region().contains_region(&region)? {
                child_containing_region = Some(child_index);
                break 'mbr_search;
            }
        }

        // If we found a child node containing our region, recurse into that node
        if let Some(child_index) = child_containing_region {
            return self.insert_at_node(region, data, child_index);
        }

        // Otherwise there is no child MBR containing our input `region`.  Thus find
        // the bounding box in this node such that enlarging it to contain
        // `minimum_bounding_region` will add the least amount of area.
        if let Some((_, combined_region, child_index)) = self
            .child_iter(index)
            .map(|(child_index, child_node)| {
                let initial_area = child_node.get_region().get_area();
                // TODO: figure out a better error handling path here (perhaps use `filter_map`)
                let combined_region = child_node
                    .get_region()
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
            self.get_node_mut(child_index)
                .set_minimum_bounding_region_unsafe(combined_region);

            // Since the enlarged bounding box now contains our object, recurse into that subtree
            return self.insert_at_node(region, data, child_index);
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
            let r1 = &self.nodes[*node1].get_region();
            let a1 = r1.get_area();

            for (l2_index, node2) in leaves.iter().enumerate().skip(l1_index + 1) {
                let r2 = &self.nodes[*node2].get_region();
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
        let mut group1 = Vec::with_capacity(self.max_children - self.min_children);
        group1.push(ix1);

        // Keep track of the minimum bounding regions for the first and second group
        let mut group1_mbr = self.nodes[children[ix1]].get_region().clone();
        let mut group2_mbr = self.nodes[children[ix2]].get_region().clone();

        // Partition the nodes into two groups.  The basic strategy is that at each stepp
        // we find the unpicked node
        // If one of the groups gets too large, stop.
        while !unpicked_children.is_empty()
            && group1.len() < self.max_children - self.min_children
            && (children.len() - group1.len() - unpicked_children.len())
                < self.max_children - self.min_children
        {
            let mut best_d = std::f64::MAX;
            let mut best_index = None;

            for &index in unpicked_children.iter() {
                let g1r = group1_mbr
                    .combine_region(self.nodes[children[index]].get_region())
                    .expect("failed to combine leaves");
                let g2r = group2_mbr
                    .combine_region(self.nodes[children[index]].get_region())
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
                group1_mbr.combine_region_in_place(self.nodes[children[best_index]].get_region());
            } else {
                group2_mbr.combine_region_in_place(self.nodes[children[best_index]].get_region());
            }
        }

        if !unpicked_children.is_empty() {
            if group1.len() < self.min_children {
                // rest of the unpicked children go in group 1
                for child_index in unpicked_children {
                    group1_mbr
                        .combine_region_in_place(self.nodes[children[child_index]].get_region());
                    group1.push(child_index);
                }
            } else {
                // rest of the unpicked children go in group 2
                for child_index in unpicked_children {
                    group2_mbr
                        .combine_region_in_place(self.nodes[children[child_index]].get_region());
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
    /// - Every child of `index` has its parent set to `Some(index)`.
    pub(crate) fn set_children_safe(
        &mut self,
        index: Index,
        children: impl IntoIterator<Item = Index>,
    ) {
        // Make sure we don't have any children
        assert!(!self.get_node(index).has_children());

        // Make sure `index` exists in our tree
        assert!(self.nodes.contains(index));

        for child_index in children {
            self.get_node_mut(child_index).set_parent(index);

            // This call is fine because `child_index` refers to a node in this tree whose parent
            // attribute is set to `Some(index)`, as required.
            self.get_node_mut(index).add_child_unsafe(child_index);
        }
    }

    /// Splits the overfull node corresponding to `index`.
    fn split_node(&mut self, index: Index) {
        // Get all of the children of the current node
        let children = self.get_node_mut(index).clear_children();

        // Partition the leave indexes using the QuadraticSplit strategy
        let (left, right, left_mbr, right_mbr) = self.quadratic_partition(children);

        // check that everything has the correct size
        debug_assert!(left.len() >= self.min_children);
        debug_assert!(right.len() >= self.min_children);

        // If we're splitting the root node, collect all children of the root node into two groups
        // which will be our new root children.
        //
        //       root     =>    root
        //      / | \          /    \
        //     /  |  \       left  right
        //
        if index == self.root {
            // insert a new left node
            let left_node = Node::new_internal_node(left_mbr, Some(index));
            let left_index = self.nodes.insert(left_node);
            self.set_children_safe(left_index, left);

            // insert a new right node
            let right_node = Node::new_internal_node(right_mbr, Some(index));
            let right_index = self.nodes.insert(right_node);
            self.set_children_safe(right_index, right);

            // Add the left and right nodes as children of the current node.
            // This call is safe because:
            // - The current node has no children,
            // - The nodes corresponding to `left_index` and `right_index` both have their `parent`
            //   attribute set to `Some(index)`, i.e. the index of the current node.
            self.get_node_mut(index)
                .set_children_unsafe(vec![left_index, right_index]);
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
            left_node.set_minimum_bounding_region_unsafe(left_mbr);
            left_node.set_children_unsafe(left);

            // make a new empty right node
            let right_index = self
                .nodes
                .insert(Node::new_internal_node(right_mbr, Some(parent)));

            // add the right as children (safely) of the right node
            self.set_children_safe(right_index, right.iter().cloned());

            // This call is fine here because `right_index` refers to a node in this tree
            // whose parent attribute is set to `Some(parent)`.
            self.get_node_mut(parent).add_child_unsafe(right_index);

            if self.nodes[parent].child_count() >= self.max_children {
                self.split_node(parent);
            }
        }
    }

    /// Validates the consistency of the tree.  In particular, this function checks that:
    ///
    /// - Every child is contained in the minimum bounding region of its parent, and
    /// - The total number of descendants of the root node is equal to the number
    ///   of nodes in the tree minus one.
    pub fn validate_consistency(&self) {
        let mut node_counter = 0;

        self._validate_consistency(self.root, &mut node_counter);

        // check we have the expected number of nodes.
        assert_eq!(node_counter, self.nodes.len());
    }

    /// Recursively validates that the children of each node are contained in the MBR
    /// of their parent.
    fn _validate_consistency(&self, index: Index, node_counter: &mut usize) {
        let node = &self.nodes[index];

        // increment the node counter
        *node_counter += 1;

        for (_, child_node) in self.child_iter(index) {
            // are all children of this node contained in the MBR of this node?
            assert_eq!(
                node.get_region().contains_region(child_node.get_region()),
                Ok(true)
            );

            // does every child have its parent attribute set correctly?
            assert_eq!(child_node.get_parent(), Some(index));
        }

        // validate all children of this node
        for child_index in node.child_index_iter() {
            self._validate_consistency(child_index, node_counter);
        }
    }

    /// Returns an iterator of pairs `(Index, &Node)` of children of the node corresponding to the
    /// given `index`.
    ///
    /// # Panics
    /// This function will panic if `index` does not refer to a node in this tree.
    #[inline(always)]
    fn child_iter(&self, index: Index) -> impl Iterator<Item = (Index, &Node<ND>)> + '_ {
        self.nodes[index]
            .child_index_iter()
            .map(move |index| (index, self.get_node(index)))
    }

    /// Returns a reference to the [`Node`] with index `index`.
    ///
    /// # Panics
    /// This function will panic if `index` does not refer to a node in this tree.
    #[inline(always)]
    pub fn get_node(&self, index: Index) -> &Node<ND> {
        &self.nodes[index]
    }

    /// Returns a mutable reference to the [`Node`] with index `index.
    ///
    /// # Panics
    /// This function will panic if `index` does not refer to a node in this tree.
    #[inline(always)]
    pub fn get_node_mut(&mut self, index: Index) -> &mut Node<ND> {
        &mut self.nodes[index]
    }

    /// Returns a reference to the root [`Node`] in this tree.
    #[inline(always)]
    pub fn root_node(&self) -> &Node<ND> {
        &self.nodes[self.root]
    }

    /// Returns the index of the root node in this tree.
    #[inline(always)]
    pub fn root_index(&self) -> Index {
        self.root
    }

    /// Returns a vector of pairs `(Index, Index)` corresponding to all edges in this tree.
    /// The edges are always of the form `(Parent, Child)`.
    #[cfg(feature = "graphviz")]
    #[inline(always)]
    fn collect_edges(&self) -> Vec<(Index, Index)> {
        // collect all edges in the tree
        let mut edges = Vec::new();
        self._collect_edges(&mut edges, self.root);

        edges
    }

    /// Recursively extends `buffer` with all children of the given node.
    #[inline(always)]
    fn _collect_edges(&self, buffer: &mut Vec<(Index, Index)>, index: Index) {
        // extend buffer with all edges from the current node
        let node = self.get_node(index);

        for child_index in node.child_index_iter() {
            buffer.push((index, child_index));
            self._collect_edges(buffer, child_index);
        }
    }

    /// Returns `true` if any direct child of this node is a leaf node, `false` otherwise.
    #[inline(always)]
    fn has_child_leaf(&self, index: Index) -> bool {
        for (_, child_node) in self.child_iter(index) {
            if child_node.is_leaf() {
                return true;
            }
        }
        false
    }

    /// Returns a `Vec<Index>` of those elements in the tree whose bounding box contains the
    /// minimum bounding box of the input `shape`.
    #[inline(always)]
    pub fn shape_lookup(&self, shape: &Shape) -> Vec<Index> {
        match shape {
            Shape::Point(point) => self._point_lookup(point),
            Shape::LineSegment(line) => self.line_lookup(line),
            Shape::Region(region) => self._region_lookup(region),
        }
    }

    /// Searches the tree for any leaves containing the input shape `shape`.
    /// `pred` should be a function `Fn(shape: &S, region: &Region) -> bool` indicating whether
    /// whether we should recurse into `region`.  Some examples of `pred` could be:
    /// - Check whether `shape` is contained in region,
    /// - Check whether `shape` and `region` overlap
    fn _lookup<S, F: Fn(&S, &Region) -> bool>(
        &self,
        shape: &S,
        pred: F,
        index: Index,
    ) -> Vec<Index> {
        let mut hits = Vec::new();
        let mut work_queue = vec![index];

        'work_loop: while let Some(index) = work_queue.pop() {
            let node = self.get_node(index);

            // If we're at a leaf node, then add it to our hits vector.
            if node.is_leaf() {
                hits.push(index);
                continue 'work_loop;
            }

            // Otherwise iterate over the children of this node, extending `work_queue`
            // by any children where `pref` whose bounding box contains region`.
            for (child_index, child_node) in self.child_iter(index) {
                if pred(shape, child_node.get_region()) {
                    work_queue.push(child_index);
                }
            }
        }

        hits
    }

    /// Returns a `Vec<Index>` of those regions in the tree containing the given point `point`.
    ///
    /// # Example
    /// ```rust
    /// use spaceindex::rtree::RTree;
    ///
    /// let mut tree = RTree::new(2);
    ///
    /// // insert a couple of regions
    /// tree.insert(((0.0, 0.0), (2.0, 2.0)), ());
    /// tree.insert(((1.0, 0.0), (3.0, 3.0)), ());
    ///
    /// // Both rectangles contain the point (1.0, 1.0)
    /// assert_eq!(tree.point_lookup((1.0, 1.0)).len(), 2);
    ///
    /// // No rectangle should contain the point (-1.0, 0.0)
    /// assert!(tree.point_lookup((-1.0, 0.0)).is_empty());
    ///
    /// // Only one hit for (0.5, 0.5)
    /// assert_eq!(tree.point_lookup((0.5, 0.5)).len(), 1);
    ///
    /// // Two hits at (2.0, 2.0)
    /// assert_eq!(tree.point_lookup((2.0, 2.0)).len(), 2);
    ///
    /// // Only one hit at (2.5, 2.5)
    /// assert_eq!(tree.point_lookup((2.5, 2.5)).len(), 1);
    /// ```
    #[inline(always)]
    pub fn point_lookup<IP: IntoPoint>(&self, point: IP) -> Vec<Index> {
        self._point_lookup(&point.into_pt())
    }

    #[inline(always)]
    fn _point_lookup(&self, point: &Point) -> Vec<Index> {
        self._lookup(
            point,
            |point, child_region| child_region.contains_point(point).unwrap(),
            self.root,
        )
    }

    /// Returns a `Vec<Index>` of those elements in the tree whose minimum bounding box
    /// intersects the given region.
    ///
    /// # Example
    /// ```rust
    /// use spaceindex::rtree::RTree;
    ///
    /// let mut tree = RTree::new(2);
    ///
    /// // insert a couple of regions
    /// tree.insert(((0.0, 0.0), (5.0, 5.0)), ());
    /// tree.insert(((-1.0, 1.0), (1.0, 3.0)), ());
    ///
    /// // Nothing should intersect with the box ((-3.0, 0.0), (-2.0, 2.0))
    /// assert!(tree.region_intersection_lookup(((-3.0, 0.0), (-2.0, 2.0))).is_empty());
    ///
    /// // The region ((-3.0, 0.0), (-0.5, 4.0)) should intersect the second region.
    /// assert_eq!(tree.region_intersection_lookup(((-3.0, 0.0), (-0.5, 4.0))).len(), 1);
    ///
    /// // The skinny box ((-2.0, 1.5), (8.0, 1.5)) should intersect both regions.
    /// assert_eq!(tree.region_intersection_lookup(((-2.0, 1.5), (8.0, 1.5))).len(), 2);
    ///
    /// // The region ((3.0, 2.0), (4.0, 4.0)) should only intersect the first region.
    /// assert_eq!(tree.region_intersection_lookup(((3.0, 2.0), (4.0, 4.0))).len(), 1);
    /// # tree.validate_consistency();
    /// ```
    #[inline(always)]
    pub fn region_intersection_lookup<'a, IC: IntoRegion<'a>>(&self, region: IC) -> Vec<Index> {
        self._region_intersection_lookup(&region.into_region())
    }

    #[inline(always)]
    fn _region_intersection_lookup(&self, region: &Region) -> Vec<Index> {
        self._lookup(
            region,
            |region, child_region| child_region.intersects_region(region).unwrap(),
            self.root,
        )
    }

    /// Returns a `Vec<Index>` of those elements in the tree whose minimum bounding box
    /// contains the given region.
    ///
    /// # Example
    /// ```rust
    /// use spaceindex::rtree::RTree;
    ///
    /// let mut tree = RTree::new(2);
    ///
    /// // insert a couple of regions
    /// tree.insert(((0.0, 0.0), (2.0, 2.0)), ());
    /// tree.insert(((1.0, 0.0), (3.0, 3.0)), ());
    ///
    /// // Both regions contain the box ((1.25, 1.0), (1.75, 1.75))
    /// assert_eq!(tree.region_lookup(((1.25, 1.0), (1.75, 1.75))).len(), 2);
    ///
    /// // While the box ((-0.5, -0.5), (0.5, 0.5)) does intersect our first region,
    /// // it is not contained in any region, so we should get no results.
    /// assert!(tree.region_lookup(((-0.5, -0.5), (0.5, 0.5))).is_empty());
    ///
    /// /// The box ((0.0, 0.5), (0.75, 1.99)) is only contained in the first region.
    /// assert_eq!(tree.region_lookup(((0.0, 0.5), (0.75, 1.99))).len(), 1);
    /// # tree.validate_consistency();
    /// ```
    #[inline(always)]
    pub fn region_lookup<'a, IC: IntoRegion<'a>>(&self, region: IC) -> Vec<Index> {
        self._region_lookup(&region.into_region())
    }

    #[inline(always)]
    fn _region_lookup(&self, region: &Region) -> Vec<Index> {
        self._lookup(
            region,
            |region, child_region| child_region.contains_region(region).unwrap(),
            self.root,
        )
    }

    /// Returns a `Vec<Index>` of those elements in the tree whose minimum bounding box
    /// contains the given line.
    #[inline(always)]
    pub fn line_lookup(&self, line: &LineSegment) -> Vec<Index> {
        let minimum_bounding_region = line.get_min_bounding_region();
        self.region_lookup(minimum_bounding_region)
    }
}
