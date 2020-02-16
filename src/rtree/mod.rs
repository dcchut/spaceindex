use crate::geometry::{Region, Shapelike, ShapelikeError};
use generational_arena::{Arena, Index};
use std::collections::HashSet;

const MAX_LEAVES: usize = 4;
const MAX_CHILDREN: usize = 4;

#[derive(Debug)]
pub struct RTree {
    /// Nodes are stored in a generational arena
    pub nodes: Arena<Node>,

    pub root: Index,
}

impl RTree {
    pub fn validate_consistency(&self, index: Index) {
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
            self.validate_consistency(*child_index);
        }
    }

    /// Creates a new [`RTree`]
    pub fn new(dimension: usize) -> (Self, Index) {
        let node = Node::new(Region::infinite(dimension));
        let mut nodes = Arena::new();
        let root_index = nodes.insert(node);

        (
            Self {
                nodes,
                root: root_index,
            },
            root_index,
        )
    }

    /// Inserts a node into our tree at the given position.
    fn _insert(&mut self, region: Region, leaf_index: Index, parent: Option<Index>) {
        // Parent node should always contain the input region
        debug_assert_eq!(
            self.nodes[leaf_index]
                .minimum_bounding_region
                .contains_region(&region),
            Ok(true)
        );

        // don't attach leaves directly to the root node
        if leaf_index == self.root {
            // create a new child node containing only this leaf
            let node = Node::new(region.clone());
            let node_index = self.nodes.insert(node);

            // add our new node as a child of the root node
            self.nodes[self.root].children.push(node_index);

            return self._insert(region, node_index, Some(self.root));
        }

        // otherwise attach our leaf to this node
        let leafed_node = &mut self.nodes[leaf_index];
        leafed_node.leaves.push(Leaf::new(region));

        if leafed_node.leaf_count() >= MAX_LEAVES {
            self.split_leaf(leaf_index, parent.unwrap());
        }
    }

    /// Attempts to insert a given object into the tree.
    pub fn insert(&mut self, region: Region, object: usize) -> Result<(), ShapelikeError> {
        // The internal `root` node always contains everything.
        self.insert_at_node(region, object, self.root, None)
    }

    fn insert_at_node(
        &mut self,
        region: Region,
        object: usize,
        index: Index,
        parent: Option<Index>,
    ) -> Result<(), ShapelikeError> {
        // current node under consideration
        let node = &self.nodes[index];

        if !node.has_children() || node.has_leaves() {
            // If we've reached a leaf note, insert this as a leaf of the parent?
            self._insert(region, index, parent);
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
            return self.insert_at_node(region, object, child_index, Some(index));
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
            return self.insert_at_node(region, object, child_index, Some(index));
        }

        panic!("something weird happened");
    }

    fn quadratic_partition<'a, S>(
        &self,
        nodes: &'a [S],
        get_region: impl Fn(&'a S) -> &Region,
    ) -> (HashSet<usize>, Region, Region) {
        let mut worst_pair = None;
        let mut worst_area = -1.0;

        // find the two leaves of this node that would be the most terrible together
        for (l1_index, node1) in nodes.iter().enumerate() {
            let r1 = get_region(node1);
            let a1 = r1.get_area();

            for (l2_index, node2) in nodes.iter().enumerate().skip(l1_index + 1) {
                let r2 = get_region(node2);
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

        let (l1, l2) = worst_pair.unwrap();
        let mut unpicked_nodes: HashSet<usize> = (0..nodes.len()).collect();
        unpicked_nodes.remove(&l1);
        unpicked_nodes.remove(&l2);

        let mut group1 = HashSet::new();
        group1.insert(l1);

        let mut left_mbr = get_region(&nodes[l1]).clone();
        let mut right_mbr = get_region(&nodes[l2]).clone();

        while !unpicked_nodes.is_empty() {
            let mut best_d = -1.0;
            let mut best_index = None;

            for &index in unpicked_nodes.iter() {
                let g1r = left_mbr
                    .combine_region(get_region(&nodes[index]))
                    .expect("failed to combine leaves");
                let g2r = right_mbr
                    .combine_region(get_region(&nodes[index]))
                    .expect("failed to combine leaves");

                let d1 = g1r.get_area() - left_mbr.get_area();
                let d2 = g2r.get_area() - right_mbr.get_area();

                if (d1 - d2).abs() > best_d {
                    best_d = (d1 - d2).abs();
                    best_index = Some((index, d1, d2));
                }
            }

            let (best_index, d1, d2) = best_index.unwrap();
            unpicked_nodes.remove(&best_index);

            if d1 < d2 {
                // add to group 1
                group1.insert(best_index);
                left_mbr = left_mbr
                    .combine_region(get_region(&nodes[best_index]))
                    .expect("failed to combine leaves");
            } else {
                right_mbr = right_mbr
                    .combine_region(get_region(&nodes[best_index]))
                    .expect("failed to combine leaves");
            }
        }

        (group1, left_mbr, right_mbr)
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

    // Consider a situation like this:
    //
    //          root
    //           |
    //          node
    //        /  |  \
    //   leaf1 leaf2 leaf3
    //
    // Assuming a maximum leaf count of two, this found should rebalance our tree to look like:
    //
    //            root
    //           /    \
    //        node   new node
    //      /     \      \
    //    leaf1, leaf2   leaf3
    //
    fn split_leaf(&mut self, index: Index, parent: Index) {
        let node = &mut self.nodes[index];

        let mut leaves = Vec::new();
        std::mem::swap(&mut leaves, &mut node.leaves);

        let (left, left_mbr, right_mbr) = self.quadratic_partition(&leaves, |leaf| &leaf.region);

        let (left, right) = Self::assemble(leaves, left);

        let node = &mut self.nodes[index];
        node.minimum_bounding_region = left_mbr;
        node.leaves = left;

        let mut right_node = Node::new(right_mbr);
        right_node.leaves = right;
        let right_index = self.nodes.insert(right_node);

        let parent_node = &mut self.nodes[parent];
        parent_node.children.push(right_index);

        // our parent node now might require rebalancing
        if parent_node.child_count() >= MAX_CHILDREN {
            self.split_internal_node(parent);
        }
    }

    // Similar to a previous function, but based on the number of children.
    //
    //                 root
    //         /     |      \      \
    //      child1  child2  child3 child4
    //
    // should become
    //
    //               root
    //            /        \
    //        child        child
    //         /  \         /  \
    //     child1 child2 child3 child4
    fn split_internal_node(&mut self, index: Index) {
        let node = &mut self.nodes[index];

        let mut children = Vec::with_capacity(2);
        std::mem::swap(&mut children, &mut node.children);

        let (left, left_mbr, right_mbr) = self.quadratic_partition(&children, |index| {
            &self.nodes[*index].minimum_bounding_region
        });

        let (left, right) = Self::assemble(children, left);

        // add a new node for the left half
        let mut left_node = Node::new(left_mbr);
        left_node.children = left;

        // add a new node for the right half
        let mut right_node = Node::new(right_mbr);
        right_node.children = right;

        // our original node now has only two nodes, left_node and right_node
        let left_node_index = self.nodes.insert(left_node);
        let right_node_index = self.nodes.insert(right_node);

        let node = &mut self.nodes[index];
        node.children.push(left_node_index);
        node.children.push(right_node_index);
    }
}

#[derive(Debug)]
pub struct Node {
    /// The minimum bounding region enclosing all data contained in this node
    pub minimum_bounding_region: Region,

    /// Children of this node
    pub children: Vec<Index>,

    /// Leaves attached to this node
    pub leaves: Vec<Leaf>,
}

#[derive(Debug)]
pub struct Leaf {
    /// A region
    pub region: Region,
}

impl Leaf {
    pub fn new(region: Region) -> Self {
        Self { region }
    }
}

impl Node {
    pub fn new(minimum_bounding_region: Region) -> Self {
        Self {
            minimum_bounding_region,
            children: Vec::new(),
            leaves: Vec::new(),
        }
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn has_leaves(&self) -> bool {
        !self.leaves.is_empty()
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn leaf_count(&self) -> usize {
        self.leaves.len()
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
