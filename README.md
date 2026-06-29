# surgeist-shape

Shape, geometry, and primitive path-data contracts for Surgeist rendering and layout-adjacent surfaces.

## Modeling Contract

`surgeist-shape` owns resolved geometry and shape-boundary invariants. Public
constructors validate finite coordinates, non-negative dimensions, valid path
ordering, stroke geometry, and dash geometry before values enter normal use.
