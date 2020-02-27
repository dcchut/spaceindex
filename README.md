# spaceindex

spaceindex is a tool for building r-trees.

![Tree](https://user-images.githubusercontent.com/266585/74727907-fa3b0c80-5295-11ea-9e96-7bd1545bbfcb.png)

![Another Tree](https://user-images.githubusercontent.com/266585/74735021-f2826480-52a3-11ea-8c6c-5de316ff2297.png)

[![Build Status](https://travis-ci.org/dcchut/spaceindex.svg?branch=master)](https://travis-ci.org/dcchut/spaceindex)
[![codecov](https://codecov.io/gh/dcchut/spaceindex/branch/master/graph/badge.svg)](https://codecov.io/gh/dcchut/spaceindex)


* [API Documentation](https://docs.rs/spaceindex/)
* Cargo package: [spaceindex](https://crates.io/crates/spaceindex)

---
## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
spaceindex = "0.3"
```

## Basic Usage

To create a new `RTree`, use:

```rust
use spaceindex::rtree::RTree;

// Creates a 2-dimensional RTree
let mut rtree : RTree<()> = RTree::new(2);

// This region is the rectangle whose lower-left corner is at (0,0) and whose upper-right corner is at (2, 2)
rtree.insert(((0.0, 0.0), (2.0, 2.0)), ()).expect("failed to insert");

// This region goes from (1, 0) to (3, 3).
rtree.insert((1.0, 0.0), (3.0, 3.0), ()).expect("failed to insert");

// Both rectangles contain the point (1, 1)
assert_eq!(rtree.point_lookup((1.0, 1.0)).len(), 2);

// No rectangle should contain the point (-1, 0)
assert!(rtree.point_lookup((-1.0, 0.0)).is_empty());
```

## Python module

Also included is `pyspaceindex`, a Python module exposing a simple interface
for working with two dimensional RTree's.  

### Build instructions

To build `pyspaceindex`:
- Install the excellent [maturin](https://pypi.org/project/maturin/) package from pypi.
- Navigate to the `spaceindex-py` directory in this repository, then run `maturin build` to build a copy
  of the wheel.  To install the module in your current virtualenv instead, run `maturin develop` instead.

### Example usage
```python
import pyspaceindex as psi

# Make an RTree instance
tree = psi.RTree()

# A region is described by a tuple (min_x, min_y, max_x, max_y).
tree.insert((0, 0, 3, 3), 12)

# A tree can contain data, as well.
tree.insert((-1, -1, 2, 2), 99)

# Query the tree for whether it contains a point
assert sorted(tree.query(-0.5, 1.0)) == [12, 99] 
```
Also included is a Python module, `spaceindex-py`.  

### License
Licensed under either of
 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
