use crate::geometry::{Region, Shapelike, ShapelikeError};
use generational_arena::{Arena, Index};
use std::collections::HashSet;

const MIN_CHILDREN: usize = 2;
const MAX_CHILDREN: usize = 8;

#[derive(Debug)]
pub struct RTree {
    /// Nodes are stored in a generational arena
    pub nodes: Arena<Node>,

    pub root: Index,
}

impl RTree {
    fn _validate_consistency(&self, index: Index) {
        let node = &self.nodes[index];

        // are all children of this node contained in the MBR of this node?
        for (_, child_node) in node.child_iter(self) {
            assert_eq!(
                node.minimum_bounding_region
                    .contains_region(&child_node.minimum_bounding_region),
                Ok(true)
            );
        }

        // validate all children of this node
        for child_index in node.children.iter() {
            self._validate_consistency(*child_index);
        }
    }

    pub fn validate_consistency(&self) {
        self._validate_consistency(self.root)
    }

    /// Creates a new [`RTree`]
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

    /// Inserts a node into our tree at the given position.
    fn _insert(&mut self, region: Region, index: Index) {
        // Parent node should always contain the input region
        assert_eq!(
            self.nodes[index]
                .minimum_bounding_region
                .contains_region(&region),
            Ok(true)
        );

        // enlarge the existing MBR
        //self.nodes[index].minimum_bounding_region.combine_region_in_place(&region);

        // add a new leaf as a child of this node
        let leaf_node = Node::new_leaf(region, Some(index));
        let leaf_index = self.nodes.insert(leaf_node);

        self.nodes[index].children.push(leaf_index);

        if self.nodes[index].child_count() >= MAX_CHILDREN {
            self.split_node(index);
        }
    }

    /// Attempts to insert a given object into the tree.
    pub fn insert(&mut self, region: Region, object: usize) -> Result<(), ShapelikeError> {
        // The internal `root` node always contains everything.
        self.insert_at_node(region, object, self.root)
    }

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

        'mbr_search: for (child_index, child_node) in node.child_iter(self) {
            if child_node
                .minimum_bounding_region
                .contains_region(&region)?
            {
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
        if let Some((_, combined_region, child_index)) = node
            .child_iter(self)
            .map(|(child_index, child_node)| {
                let initial_area = child_node.minimum_bounding_region.get_area();
                // TODO: figure out a better error handling path here (perhaps use `filter_map`)
                let combined_region = child_node
                    .minimum_bounding_region
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
            // Enlarge `child_index`'s bounding box
            self.nodes[child_index].minimum_bounding_region = combined_region;

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

    fn split_node(&mut self, index: Index) {
        // take ownership of the children of the current node
        let mut children = Vec::new();
        std::mem::swap(&mut children, &mut self.nodes[index].children);

        // Partition the leave indexes using the QuadraticSplit strategy
        // TODO: parametrize this strategy
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

            // mark the left node as the parent of all of its children
            for left_child_index in left.iter() {
                self.nodes[*left_child_index].parent = Some(left_index);

                // left node MBR should always contain all of its children
                debug_assert_eq!(
                    self.nodes[left_index]
                        .minimum_bounding_region
                        .contains_region(&self.nodes[*left_child_index].minimum_bounding_region),
                    Ok(true)
                );
            }

            self.nodes[left_index].children = left;

            // insert a new right node
            let right_node = Node::new_internal_node(right_mbr, Some(index));
            let right_index = self.nodes.insert(right_node);

            // mark the right node as the parent of all its children
            for right_child_index in right.iter() {
                self.nodes[*right_child_index].parent = Some(right_index);

                //                assert_eq!(self.nodes[right_index]
                //                               .minimum_bounding_region
                //                               .contains_region(&self.nodes[*right_child_index].minimum_bounding_region),
                //                           Ok(true));
            }

            self.nodes[right_index].children = right;

            // add the left and right nodes as children of the current node
            self.nodes[index].children = vec![left_index, right_index];
        } else {
            // Otherwise we apply the transformation:
            //
            //     parent            parent
            //       |               /   \
            //      node    =>    node    right
            //     / | \          /  \    /  \
            //    /  |  \        /    \  /    \
            //
            let parent = self.nodes[index].parent.unwrap();

            // the current node will become our new left node
            let left_node = &mut self.nodes[index];
            left_node.minimum_bounding_region = left_mbr;
            left_node.children = left;

            // make a new empty right node
            let right_index = self
                .nodes
                .insert(Node::new_internal_node(right_mbr, Some(parent)));

            // mark the right node as the parent of all its children
            for right_child_index in right.iter() {
                self.nodes[*right_child_index].parent = Some(right_index);
            }

            self.nodes[right_index].children = right;

            // add the right node as a child of the parent node
            self.nodes[parent].children.push(right_index);

            if self.nodes[parent].child_count() >= MAX_CHILDREN {
                self.split_node(parent);
            }
        }
    }
}

#[derive(Debug)]
pub struct Node {
    /// The minimum bounding region enclosing all data contained in this node
    pub minimum_bounding_region: Region,

    /// Children of this node
    pub children: Vec<Index>,

    /// Is this a leaf node?
    pub is_leaf: bool,

    /// The index of the parent node
    pub parent: Option<Index>,
}

impl Node {
    fn new(
        minimum_bounding_region: Region,
        children: Vec<Index>,
        is_leaf: bool,
        parent: Option<Index>,
    ) -> Self {
        Self {
            minimum_bounding_region,
            children,
            is_leaf,
            parent,
        }
    }

    pub fn new_internal_node(minimum_bounding_region: Region, parent: Option<Index>) -> Self {
        Self::new(minimum_bounding_region, Vec::new(), false, parent)
    }

    pub fn new_leaf(minimum_bounding_region: Region, parent: Option<Index>) -> Self {
        Self::new(minimum_bounding_region, Vec::new(), true, parent)
    }

    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    /// Are any (direct) children of this node a leaf?
    pub fn has_leaf_child(&self, tree: &RTree) -> bool {
        for child_index in self.children.iter() {
            if tree.nodes[*child_index].is_leaf() {
                return true;
            }
        }

        false
    }

    /// Does this node have any children?
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// How many direct children does this node have?
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Get the minimum bounding region of this node
    pub fn region(&self) -> &Region {
        &self.minimum_bounding_region
    }

    pub fn child_iter<'s, 't, 'g>(
        &'s self,
        tr: &'t RTree,
    ) -> impl Iterator<Item = (Index, &'t Node)> + 'g
    where
        's: 'g,
        't: 'g,
    {
        self.children.iter().map(move |ix| (*ix, &tr.nodes[*ix]))
    }
}
