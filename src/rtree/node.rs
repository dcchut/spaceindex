use crate::geometry::Region;
use generational_arena::Index;

#[derive(Debug)]
pub struct Node<S> {
    /// The minimum bounding region enclosing all data contained in this node.
    minimum_bounding_region: Region,

    /// A vector containing all of the children of this node.
    children: Vec<Index>,

    /// A boolean indicating whether this node is a leaf node.
    is_leaf: bool,

    /// Some data owned by this node
    data: Option<S>,

    /// The index of the parent node in our tree.
    parent: Option<Index>,
}

impl<S> Node<S> {
    /// Create a new node.
    fn new(
        minimum_bounding_region: Region,
        children: Vec<Index>,
        is_leaf: bool,
        data: Option<S>,
        parent: Option<Index>,
    ) -> Self {
        Self {
            minimum_bounding_region,
            children,
            is_leaf,
            data,
            parent,
        }
    }

    /// Creates a new internal [`Node`] with the given minimum bounding region and parent.
    pub(crate) fn new_internal_node(
        minimum_bounding_region: Region,
        parent: Option<Index>,
    ) -> Self {
        Self::new(minimum_bounding_region, Vec::new(), false, None, parent)
    }

    /// Creates a new leaf [`Node`] with the given minimum bounding region and parent.
    pub(crate) fn new_leaf(
        minimum_bounding_region: Region,
        data: S,
        parent: Option<Index>,
    ) -> Self {
        Self::new(
            minimum_bounding_region,
            Vec::new(),
            true,
            Some(data),
            parent,
        )
    }

    /// Returns `true` if this node is a leaf node, `false` otherwise.
    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    /// Returns `true` if this node has any children, `false` otherwise.
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns the number of direct children this node has.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Returns a reference to the minimum bounding region of this node.
    pub fn region(&self) -> &Region {
        &self.minimum_bounding_region
    }

    /// Returns an iterator over the `Index`es of children of this node.
    pub fn child_index_iter(&self) -> impl Iterator<Item = Index> + '_ {
        self.children.iter().cloned()
    }

    /// Sets the children vector of `self` to be equal to `children`.  This method is unsafe,
    /// as using it incorrectly will lead to corrupt data.
    ///
    /// To use this function safely, it is required that:
    /// - The node currently has no children (to prevent dangling nodes in our tree), and
    /// - All of the nodes referred to by `children` must already have their `parent` attribute
    ///   set to the index of the current node.
    pub(crate) unsafe fn set_children_unsafe(&mut self, children: Vec<Index>) {
        // Again, we should only ever do this on a node that has no children.
        debug_assert!(self.children.is_empty());

        self.children = children;
    }

    /// Adds a new child to  the current node.  This method is unsafe, as using it incorrectly
    /// will lead to corrupt data.
    ///
    /// To use this function safely, it is required that the node with index `child_index`
    /// in our tree has their `parent` attribute set to the index of the current node, and
    /// that the child is contained in the minimum bounding region of this node.
    pub(crate) unsafe fn add_child_unsafe(&mut self, child_index: Index) {
        self.children.push(child_index);
    }

    /// Returns the `parent` of the current node
    pub(crate) fn get_parent(&self) -> Option<Index> {
        self.parent
    }

    /// Updates the `parent` of the current node
    pub(crate) fn set_parent(&mut self, index: Index) {
        self.parent = Some(index);
    }

    /// Overwrites the current minimum bounding region of this node.  This method is unsafe,
    /// as using it incorrectly can lead to corrupt data.
    ///
    /// To use this function safely,
    pub(crate) unsafe fn set_minimum_bounding_region_unsafe(
        &mut self,
        minimum_bounding_region: Region,
    ) {
        self.minimum_bounding_region = minimum_bounding_region;
    }

    /// Clears all children of the current node, returning a vector of all of the direct
    /// children of the current node.
    pub(crate) fn clear_children(&mut self) -> Vec<Index> {
        let mut buffer = Vec::new();
        std::mem::swap(&mut buffer, &mut self.children);

        buffer
    }
}
