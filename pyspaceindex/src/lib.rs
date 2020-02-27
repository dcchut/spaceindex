use pyo3::exceptions::{RuntimeError, ValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySet, PyTuple};

use spaceindex::geometry::Region;
use spaceindex::rtree::{Index, RTree as Tree};

#[pyclass]
struct RTree {
    tree: Tree<PyObject>,
}

impl RTree {
    pub fn _query<S, IT: IntoIterator<Item = Index>>(
        &self,
        py: Python,
        shape: S,
        lookup: impl Fn(S) -> IT,
        hit_test: Option<PyObject>,
    ) -> PyResult<Vec<PyObject>> {
        let mut hits = Vec::new();

        // Iterate over all points in our tree containing the point `(x, y)`.
        for hit in lookup(shape) {
            // for hit in self.tree.point_lookup((x, y)) {
            // Retrieve a ref to the item in the tree
            let item = self.tree.get_node(hit).get_data().ok_or_else(|| {
                PyErr::new::<RuntimeError, _>(format!(
                    "failed to retrieve item with index {:?}",
                    hit
                ))
            })?;

            // whether this item should be included in the result
            let include_in_result: bool = match &hit_test {
                Some(hit_test) => hit_test.call1(py, (item,))?.extract(py)?,
                None => true,
            };

            if include_in_result {
                // Clone our internally held reference (increases the reference count)
                hits.push(item.clone_ref(py));
            }
        }

        Ok(hits)
    }

    fn _to_region(&self, bounds: &PyTuple) -> PyResult<Region> {
        // Extract the bounding box
        let minx: f64 = bounds.get_item(0).extract()?;
        let miny: f64 = bounds.get_item(1).extract()?;
        let maxx: f64 = bounds.get_item(2).extract()?;
        let maxy: f64 = bounds.get_item(3).extract()?;

        // Build up the region
        let region = Region::new(vec![(minx, maxx), (miny, maxy)]);

        Ok(region)
    }
}

#[pymethods]
impl RTree {
    #[new]
    fn new() -> Self {
        Self { tree: Tree::new(2) }
    }

    pub fn insert(&mut self, bounds: &PyTuple, item: PyObject) -> PyResult<()> {
        if bounds.len() != 4 {
            return Err(PyErr::new::<ValueError, _>(format!(
                "expected `bounds` to be a 4-tuple, instead it was a {}-tuple",
                bounds.len()
            )));
        }

        // Insert it into our tree
        self.tree
            .insert(self._to_region(bounds)?, item)
            .map_err(|_| PyErr::new::<RuntimeError, _>("failed to insert into tree"))?;

        Ok(())
    }

    /// Finds all items in the tree that intersect with the given point.
    pub fn query(
        &self,
        py: Python,
        x: f64,
        y: f64,
        hit_test: Option<PyObject>,
        key: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let hits = self._query(py, (x, y), |point| self.tree.point_lookup(point), hit_test)?;

        if let Some(key) = key {
            // If a key is provided, then sort our hits vector and return it
            let locals = PyDict::new(py);
            locals.set_item("key", key)?;

            let hits = PyList::new(py, hits).to_object(py);
            hits.call_method(py, "sort", (), Some(locals))?;

            Ok(hits)
        } else {
            // Otherwise return a set of hits
            let hits = PySet::new(py, &hits)?;

            Ok(hits.to_object(py))
        }
    }

    /// Finds all items in the tree intersecting the supplied region.
    pub fn query_intersecting(
        &self,
        py: Python,
        bounds: &PyTuple,
        hit_test: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let region = self._to_region(bounds)?;

        let hits = self._query(
            py,
            region,
            |region| self.tree.region_intersection_lookup(region),
            hit_test,
        )?;

        // Make a set
        Ok(PySet::new(py, &hits)?.to_object(py))
    }
}

#[pymodule]
fn pyspaceindex(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RTree>()?;

    Ok(())
}
