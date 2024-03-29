use geo::kernels::HasKernel;
use geo_types::CoordFloat;
use std::borrow::Cow;
use std::path::Path;

use rustc_ap_graphviz as dot;

use crate::rtree::{Index, RTree};

type Nd = Index;
type Ed = (Index, Index);

impl<'a, ND, T> dot::Labeller<'a> for RTree<ND, T>
where
    T: CoordFloat + HasKernel,
{
    type Node = Nd;
    type Edge = Ed;

    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("rtree").unwrap()
    }

    fn node_id(&'a self, n: &Self::Node) -> dot::Id<'a> {
        let node = self.get_node(*n);

        dot::Id::new(format!("N{}_{}", n.into_raw_parts().0, node.child_count())).unwrap()
    }
}

impl<'a, ND, T> dot::GraphWalk<'a> for RTree<ND, T>
where
    T: CoordFloat + HasKernel,
{
    type Node = Nd;
    type Edge = Ed;

    fn nodes(&'a self) -> Cow<'a, [Self::Node]> {
        self.nodes.iter().map(|x| x.0).collect()
    }

    fn edges(&'a self) -> Cow<'a, [Self::Edge]> {
        Cow::from(self.collect_edges())
    }

    fn source(&'a self, edge: &Self::Edge) -> Self::Node {
        edge.0
    }

    fn target(&'a self, edge: &Self::Edge) -> Self::Node {
        edge.1
    }
}

pub fn render_gviz<P: AsRef<Path>, ND, T: CoordFloat + HasKernel>(tree: &RTree<ND, T>, path: P) {
    let path = path.as_ref();

    let mut f = std::fs::File::create(path).unwrap();
    dot::render(tree, &mut f).unwrap();
}
