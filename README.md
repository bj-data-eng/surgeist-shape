# surgeist-shape

Shape, geometry, and primitive path-data contracts for Surgeist rendering and layout-adjacent surfaces.

## API Artifact

The committed API coordination artifact lives at `api/public-api.txt`, but the
generator is owned by the root `surgeist` repo.

Refresh this crate's artifact from the root repo with:

```sh
cargo run --manifest-path api/generator/Cargo.toml -- --crate surgeist-shape
```

API refresh tooling is command-only and must not run as part of normal `cargo test`.

## Modeling Contract

`surgeist-shape` owns resolved geometry and shape-boundary invariants. Public
constructors validate finite coordinates, non-negative dimensions, valid path
ordering, stroke geometry, and dash geometry before values enter normal use.
The committed API artifact in `api/public-api.txt` records these front doors.
