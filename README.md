# spaceindex

spaceindex is a tool for building r-trees.

![Tree](https://user-images.githubusercontent.com/266585/74727907-fa3b0c80-5295-11ea-9e96-7bd1545bbfcb.png)


[![Build Status](https://travis-ci.org/dcchut/spaceindex.svg?branch=master)](https://travis-ci.org/dcchut/spaceindex)
[![codecov](https://codecov.io/gh/dcchut/spaceindex/branch/master/graph/badge.svg)](https://codecov.io/gh/dcchut/spaceindex)


* [API Documentation](https://docs.rs/spaceindex/)
* Cargo package: [spaceindex](https://crates.io/crates/spaceindex)

---
## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
spaceindex = "0.2"
```

## Basic Usage

To create a new `RTree`, use:

```rust
use spaceindex::rtree::RTree;

// Creates a 2-dimensional RTree
let mut rtree = RTree::new(2);

// Insert a region into the tree
rtree.insert(((0.0, 0.0), (3.0, 3.0)), 0).expect("failed to insert");
```

### License
Licensed under either of
 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
