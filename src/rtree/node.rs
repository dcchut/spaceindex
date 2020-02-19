use crate::geometry::Region;
use generational_arena::Index;

#[derive(Debug)]
pub struct Node<S> {
    /// The minimum bounding region enclosing all data contained in this node.
    minimum_bounding_region: Region,

    /// A vector containing all of the children of this node.
    children: Vec<Index>,

    /// Some data owned by this node
    data: Option<S>,

    /// The index of the parent node in our tree.
    parent: Option<Index>,
}

impl<S> Node<S> {
    /// Create a new node.
    #[inline(always)]
    fn new(
        minimum_bounding_region: Region,
        children: Vec<Index>,
        data: Option<S>,
        parent: Option<Index>,
    ) -> Self {
        Self {
            minimum_bounding_region,
            children,
            data,
            parent,
        }
    }

    /// Returns `true` if this node is a leaf node, `false` otherwise.
    #[inline(always)]
    pub fn is_leaf(&self) -> bool {
        self.data.is_some()
    }

    /// Returns `true` if this node has any children, `false` otherwise.
    #[inline(always)]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns the number of direct children this node has.
    #[inline(always)]
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Returns a reference to the minimum bounding region of this node.
    #[inline(always)]
    pub fn get_region(&self) -> &Region {
        &self.minimum_bounding_region
    }

    /// Returns an iterator over the `Index`es of children of this node.
    #[inline(always)]
    pub fn child_index_iter(&self) -> impl Iterator<Item = Index> + '_ {
        self.children.iter().cloned()
    }

    /// Creates a new internal [`Node`] with the given minimum bounding region and parent.
    #[inline(always)]
    pub(crate) fn new_internal_node(
        minimum_bounding_region: Region,
        parent: Option<Index>,
    ) -> Self {
        Self::new(minimum_bounding_region, Vec::new(), None, parent)
    }

    /// Creates a new leaf [`Node`] with the given minimum bounding region and parent.
    #[inline(always)]
    pub(crate) fn new_leaf(
        minimum_bounding_region: Region,
        data: S,
        parent: Option<Index>,
    ) -> Self {
        Self::new(minimum_bounding_region, Vec::new(), Some(data), parent)
    }

    /// Combines the current minimum bounding of this region with `region`.  This method is unsafe,
    /// as using it incorrectly will lead to corrupt data.
    ///
    /// To use this function safely, it is required that the minimum bounding region of the parent
    /// of this node contains `region` (and is thus guaranteed to contain the combination of
    /// this nodes current [`Region`] and `region`).
    #[inline(always)]
    pub(crate) unsafe fn combine_region(&mut self, region: &Region) {
        self.minimum_bounding_region.combine_region_in_place(region);
    }

    /// Sets the children vector of `self` to be equal to `children`.  This method is unsafe,
    /// as using it incorrectly will lead to corrupt data.
    ///
    /// To use this function safely, it is required that:
    /// - The node currently has no children (to prevent dangling nodes in our tree), and
    /// - All of the nodes referred to by `children` must already have their `parent` attribute
    ///   set to the index of the current node.
    #[inline(always)]
    pub(crate) unsafe fn set_children_unsafe(&mut self, children: Vec<Index>) {
        // Again, we should only ever do this on a node that has no children.
        debug_assert!(self.children.is_empty());

        self.children = children;
    }

    /// Adds a new child to the current node.  This method is unsafe, as using it incorrectly
    /// will lead to corrupt data.
    ///
    /// To use this function safely, it is required that the node with index `child_index`
    /// in our tree has their `parent` attribute set to the index of the current node, and
    /// that the child is contained in the minimum bounding region of this node.
    #[inline(always)]
    pub(crate) unsafe fn add_child_unsafe(&mut self, child_index: Index) {
        self.children.push(child_index);
    }

    /// Returns the `parent` of the current node
    #[inline(always)]
    pub(crate) fn get_parent(&self) -> Option<Index> {
        self.parent
    }

    /// Updates the `parent` of the current node
    #[inline(always)]
    pub(crate) fn set_parent(&mut self, index: Index) {
        self.parent = Some(index);
    }

    /// Overwrites the current minimum bounding region of this node.  This method is unsafe,
    /// as using it incorrectly can lead to corrupt data.
    ///
    /// To use this function safely,
    #[inline(always)]
    pub(crate) unsafe fn set_minimum_bounding_region_unsafe(
        &mut self,
        minimum_bounding_region: Region,
    ) {
        self.minimum_bounding_region = minimum_bounding_region;
    }

    /// Clears all children of the current node, returning a vector of all of the direct
    /// children of the current node.
    #[inline(always)]
    pub(crate) fn clear_children(&mut self) -> Vec<Index> {
        let mut buffer = Vec::new();
        std::mem::swap(&mut buffer, &mut self.children);

        buffer
    }
}
