import pyspaceindex as psi


def test_basic_rtree():
    # Make an RTree instance
    tree = psi.RTree()

    # A region is described by a tuple (min_x, min_y, max_x, max_y).
    tree.insert((0, 0, 3, 3), 12)

    # A tree can contain data, as well.
    tree.insert((-1, -1, 2, 2), 99)

    # Query the tree for whether it contains a point
    assert sorted(tree.query(0.5, 1.0)) == [12, 99]