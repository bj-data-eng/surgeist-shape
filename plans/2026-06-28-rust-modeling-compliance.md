# Rust Modeling Compliance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring `surgeist-shape` into compliance with `guidance/surgeist-rust-modeling-guide.md` by moving geometry, stroke, dash, and output invariants to typed construction boundaries.

**Architecture:** Keep this crate as the owner of resolved geometry and shape boundary primitives. Add small crate-owned scalar/value front doors, make public models constructor-owned instead of literal-owned, keep conversion to Kurbo at explicit chokepoints, and refresh the committed public API artifact after each public API task.

**Tech Stack:** Rust 2024, `kurbo = 0.13.1`, crate-local API generator in `api/generator`, baseline checks from `AGENTS.md`.

**Compatibility:** Backwards compatibility shims are not required at this phase of development. Prefer the clean modeled API over deprecated aliases, duplicate constructors, or compatibility wrappers.

---

## Review Findings

The crate is healthy by tests and formatting, but several public models rely on late validation instead of construction-time invariants.

- `src/geometry.rs:4`, `src/geometry.rs:56`, `src/geometry.rs:99`, `src/geometry.rs:302`, `src/geometry.rs:359`, and `src/geometry.rs:440` expose raw numeric fields or tuple storage that can represent non-finite values, negative sizes/radii, and malformed transforms through ordinary callers.
- `src/stroke.rs:4` exposes raw stroke fields, so negative width, negative miter limits, and invalid dash values are representable until a later `validate()` call.
- `src/dash.rs:5`, `src/dash.rs:146`, `src/dash.rs:208`, and `src/dash.rs:248` mix authored dash options, validated dash configuration, and generated dash geometry. The generated segment fields are public, so callers can construct impossible output geometry.
- `src/error.rs:30` has semantic error codes, but validation helpers map all negative numeric failures to `NegativeSize`; `NegativeRadius` is unused, and caller-facing errors do not consistently name the violated invariant.
- `src/primitive.rs:5` exposes `Shape` variants that can be constructed with invalid raw values. Because `Shape::bounds`, `Shape::contains`, `Shape::key`, and conversion methods operate before validation in some paths, invalid shapes can leak behavior.
- `src/path.rs:20` allows path commands to be built in invalid order and only reports the error during `validate()`. This is acceptable for an authored path builder if explicitly modeled as such, but the current `Path` type is also the normalized shape path.
- `src/geometry.rs:49`, `src/geometry.rs:295`, and transform helpers such as `src/geometry.rs:486` are public conversion/helper backdoors that can still admit non-finite geometry unless they become fallible or are constrained to validated inputs.
- `src/dash.rs:146` exposes raw dash anchor offsets. Either `DashAnchor` must own its own non-negative invariant or the API must clearly model anchors as authored data validated only when attached to `Dash`.
- `api/public-api.txt` must be regenerated whenever public constructors, fields, or exports change.

## Intended End State

- Public structs that own invariants have private fields and typed constructors.
- Existing ergonomic constructors either validate and return `Result<Self>` or are renamed to `unchecked` only for crate-private use.
- Literal construction remains available only for true plain data without invariants, or for constants where the invariant is visibly fixed.
- `Path` becomes a command-order-validated path type with a separate `PathBuilder`. Empty paths remain valid because `Shape::Path` currently treats empty paths as empty geometry, but this is made explicit with tests and docs.
- `Shape` construction goes through fallible front doors for dimensions and path validity; enum variants are no longer public construction backdoors.
- `Stroke` and `DashSegment` fields are private, with accessors and validating constructors.
- Error codes distinguish negative size, negative radius, invalid stroke, invalid dash, invalid path order, and non-finite input.
- Public external conversions and transform helpers do not bypass validation.
- Dash anchors do not expose raw invalid contour offsets.
- The API artifact documents the intentional breaking changes.

## Files

- Create: `src/value.rs` for shared finite and non-negative scalar wrappers.
- Modify: `src/error.rs` for semantic validation errors.
- Modify: `src/geometry.rs` for private geometry fields, accessors, and fallible constructors.
- Modify: `src/path.rs` for validated path construction or explicit builder separation.
- Modify: `src/primitive.rs` for private shape representation and fallible constructors.
- Modify: `src/stroke.rs` for private stroke fields and validating mutation methods.
- Modify: `src/dash.rs` for private dash output fields and typed dash config.
- Modify: `src/lib.rs` for new exports and internal imports.
- Modify: `src/tests.rs` for compliance tests and adjusted call sites.
- Regenerate: `api/public-api.txt` using `cargo run --manifest-path api/generator/Cargo.toml`.

## Coordinator Workflow

For each task below:

- Assign one implementation worker to exactly that task.
- Tell the worker they are not alone in the codebase and must not revert others' work.
- Wait for the worker result, including tests and `git status --short --branch`.
- Assign a separate reviewer to inspect only that task's changes against this plan and `guidance/surgeist-rust-modeling-guide.md`.
- Reconcile reviewer findings before the next task.
- Run the task's focused checks locally.
- Commit the task as a logical point before assigning the next task.

After all tasks are complete:

- Assign a holistic reviewer to inspect the complete result against this plan, crate boundaries, public API artifact, tests, and modeling guide.
- Run final checks: `cargo test -p surgeist-shape`, `cargo clippy -p surgeist-shape --all-targets -- -D warnings`, and `cargo fmt --check`.
- Mark the goal complete only if the holistic review cycle comes back clean.

### Task 1: Add Scalar Invariant Types And Semantic Errors

**Files:**
- Create: `src/value.rs`
- Modify: `src/error.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing scalar and error tests**

Add these tests to `src/tests.rs`:

```rust
#[test]
fn non_negative_scalar_rejects_negative_values_with_semantic_code() {
    let error = NonNegative::try_new(-1.0, NumericKind::Size).unwrap_err();

    assert_eq!(error.code, ErrorCode::NegativeSize);
}

#[test]
fn non_negative_scalar_rejects_negative_radius_with_radius_code() {
    let error = NonNegative::try_new(-1.0, NumericKind::Radius).unwrap_err();

    assert_eq!(error.code, ErrorCode::NegativeRadius);
}

#[test]
fn finite_scalar_rejects_nan() {
    let error = Finite::try_new(f64::NAN, "point x").unwrap_err();

    assert_eq!(error.code, ErrorCode::NonFinite);
}
```

- [ ] **Step 2: Run the focused tests and verify they fail**

Run:

```sh
cargo test -p surgeist-shape scalar -- --nocapture
```

Expected: fail with unresolved names `NonNegative`, `NumericKind`, and `Finite`.

- [ ] **Step 3: Add `src/value.rs`**

Create `src/value.rs` with:

```rust
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericKind {
    Coordinate,
    Size,
    Radius,
    Stroke,
    Dash,
    Transform,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Finite(f64);

impl Finite {
    pub const ZERO: Self = Self(0.0);

    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    pub fn try_new(value: f64, name: &str) -> Result<Self> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(Error::new(
                ErrorCode::NonFinite,
                format!("{name} must be finite"),
            ))
        }
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct NonNegative(Finite);

impl NonNegative {
    pub const ZERO: Self = Self(Finite::ZERO);

    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    pub fn try_new(value: f64, kind: NumericKind) -> Result<Self> {
        let finite = Finite::try_new(value, numeric_name(kind))?;
        if value >= 0.0 {
            Ok(Self(finite))
        } else {
            Err(Error::new(
                negative_code(kind),
                format!("{} must be non-negative", numeric_name(kind)),
            ))
        }
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0.get()
    }
}

pub(crate) fn numeric_name(kind: NumericKind) -> &'static str {
    match kind {
        NumericKind::Coordinate => "coordinate",
        NumericKind::Size => "size",
        NumericKind::Radius => "radius",
        NumericKind::Stroke => "stroke",
        NumericKind::Dash => "dash",
        NumericKind::Transform => "transform",
    }
}

pub(crate) fn negative_code(kind: NumericKind) -> ErrorCode {
    match kind {
        NumericKind::Radius => ErrorCode::NegativeRadius,
        NumericKind::Stroke => ErrorCode::InvalidStroke,
        NumericKind::Dash => ErrorCode::InvalidDash,
        _ => ErrorCode::NegativeSize,
    }
}
```

- [ ] **Step 4: Update `src/error.rs`**

Add `InvalidStroke` to `ErrorCode`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    NonFinite,
    NegativeSize,
    NegativeRadius,
    InvalidPath,
    InvalidStroke,
    InvalidDash,
    EmptyPath,
    UnsupportedStrokeBounds,
}
```

Replace `validate_non_negative` with:

```rust
pub(crate) fn validate_non_negative(value: f64, name: &str) -> Result<()> {
    validate_finite(value, name)?;
    if value >= 0.0 {
        Ok(())
    } else {
        Err(Error::new(
            ErrorCode::NegativeSize,
            format!("{name} must be non-negative"),
        ))
    }
}

pub(crate) fn validate_non_negative_kind(
    value: f64,
    name: &str,
    kind: NumericKind,
) -> Result<()> {
    validate_finite(value, name)?;
    if value >= 0.0 {
        Ok(())
    } else {
        Err(Error::new(
            negative_code(kind),
            format!("{name} must be non-negative"),
        ))
    }
}
```

Also add the required imports at the top of `src/error.rs`:

```rust
use crate::{negative_code, NumericKind};
```

- [ ] **Step 5: Update `src/lib.rs` exports**

Add:

```rust
mod value;
pub use value::{Finite, NonNegative, NumericKind};
pub(crate) use error::{validate_finite, validate_non_negative, validate_non_negative_kind};
pub(crate) use value::{negative_code, numeric_name};
```

Keep existing exports unchanged until later tasks migrate call sites.

- [ ] **Step 6: Run task checks**

Run:

```sh
cargo test -p surgeist-shape scalar
cargo clippy -p surgeist-shape --all-targets -- -D warnings
cargo fmt --check
git status --short --branch
```

Expected: tests pass, clippy passes, fmt passes, and only Task 1 files are modified.

- [ ] **Step 7: Commit Task 1**

```sh
git add src/value.rs src/error.rs src/lib.rs src/tests.rs
git commit -m "model scalar invariants"
```

### Task 2: Make Geometry Values Constructor-Owned

**Files:**
- Modify: `src/geometry.rs`
- Modify: `src/key.rs`
- Modify: `src/primitive.rs`
- Modify: `src/dash.rs`
- Modify: `src/tests.rs`
- Regenerate: `api/public-api.txt`

- [ ] **Step 1: Write failing geometry construction tests**

Add these tests:

```rust
#[test]
fn size_constructor_rejects_negative_dimensions() {
    let error = Size::try_new(-1.0, 2.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::NegativeSize);
}

#[test]
fn radii_constructor_rejects_negative_radius() {
    let error = Radii::try_all(-1.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::NegativeRadius);
}

#[test]
fn transform_constructor_rejects_non_finite_matrix() {
    let error = Transform::try_new([1.0, 0.0, 0.0, 1.0, f64::NAN, 0.0]).unwrap_err();

    assert_eq!(error.code, ErrorCode::NonFinite);
}

#[test]
fn kurbo_point_conversion_rejects_non_finite_coordinates() {
    let error = Point::try_from(kurbo::Point::new(f64::NAN, 0.0)).unwrap_err();

    assert_eq!(error.code, ErrorCode::NonFinite);
}

#[test]
fn transform_helpers_reject_non_finite_inputs() {
    let error = Transform::try_translate(f64::NAN, 0.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::NonFinite);
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```sh
cargo test -p surgeist-shape constructor_rejects -- --nocapture
```

Expected: fail because `try_new`, `try_all`, `try_from`, and `try_translate` do not exist.

- [ ] **Step 3: Refactor `Point`, `Size`, `Rect`, `Insets`, `Radii`, and `Transform`**

Make fields private and add fallible public constructors while retaining crate-private unchecked constructors for fixed internal conversions.

Required public signatures:

```rust
impl Point {
    pub fn try_new(x: f64, y: f64) -> Result<Self>;
    pub fn try_translate(self, x: f64, y: f64) -> Result<Self>;
    pub(crate) const fn new_unchecked(x: f64, y: f64) -> Self;
    pub const fn x(self) -> f64;
    pub const fn y(self) -> f64;
}

impl Size {
    pub fn try_new(width: f64, height: f64) -> Result<Self>;
    pub(crate) const fn new_unchecked(width: f64, height: f64) -> Self;
    pub const fn width(self) -> f64;
    pub const fn height(self) -> f64;
}

impl Rect {
    pub fn try_new(x: f64, y: f64, width: f64, height: f64) -> Result<Self>;
    pub fn try_from_min_max(min: Point, max: Point) -> Result<Self>;
    pub fn try_from_origin_size(origin: Point, size: Size) -> Result<Self>;
    pub fn try_inset(self, insets: Insets) -> Result<Self>;
    pub fn try_outset(self, insets: Insets) -> Result<Self>;
    pub fn try_translate(self, x: f64, y: f64) -> Result<Self>;
    pub(crate) const fn new_unchecked(x: f64, y: f64, width: f64, height: f64) -> Self;
    pub const fn origin(self) -> Point;
    pub const fn size(self) -> Size;
}

impl Insets {
    pub fn try_new(top: f64, right: f64, bottom: f64, left: f64) -> Result<Self>;
    pub fn try_all(value: f64) -> Result<Self>;
    pub(crate) const fn new_unchecked(top: f64, right: f64, bottom: f64, left: f64) -> Self;
    pub const fn top(self) -> f64;
    pub const fn right(self) -> f64;
    pub const fn bottom(self) -> f64;
    pub const fn left(self) -> f64;
}

impl Radii {
    pub fn try_new(top_left: f64, top_right: f64, bottom_right: f64, bottom_left: f64) -> Result<Self>;
    pub fn try_all(radius: f64) -> Result<Self>;
    pub fn try_inset(self, amount: f64) -> Result<Self>;
    pub fn try_outset(self, amount: f64) -> Result<Self>;
    pub(crate) const fn new_unchecked(top_left: f64, top_right: f64, bottom_right: f64, bottom_left: f64) -> Self;
    pub const fn top_left(self) -> f64;
    pub const fn top_right(self) -> f64;
    pub const fn bottom_right(self) -> f64;
    pub const fn bottom_left(self) -> f64;
}

impl Transform {
    pub fn try_new(matrix: [f64; 6]) -> Result<Self>;
    pub fn try_translate(x: f64, y: f64) -> Result<Self>;
    pub fn try_scale(x: f64, y: f64) -> Result<Self>;
    pub fn try_rotate(radians: f64) -> Result<Self>;
    pub fn try_then(self, next: Self) -> Result<Self>;
    pub fn try_apply_point(self, point: Point) -> Result<Point>;
    pub fn try_apply_rect(self, rect: Rect) -> Result<Rect>;
    pub(crate) const fn new_unchecked(matrix: [f64; 6]) -> Self;
    pub const fn matrix(self) -> [f64; 6];
}
```

Use `validate_non_negative_kind(..., NumericKind::Radius)` for radii and `NumericKind::Size` for sizes. Keep `ZERO` and `IDENTITY` constants by building them from unchecked fixed values. Since backwards compatibility shims are not required, remove or make crate-private the old infallible geometry-producing helpers that accept raw `f64` or can synthesize non-finite values, including `Point::translate`, `Rect::inset`, `Rect::outset`, `Rect::translate`, `Radii::inset`, `Radii::outset`, `Transform::then`, `Transform::apply_point`, and `Transform::apply_rect`.

- [ ] **Step 4: Close conversion and helper backdoors**

Remove infallible external conversions that can admit invalid geometry:

```rust
impl TryFrom<kurbo::Point> for Point {
    type Error = Error;

    fn try_from(point: kurbo::Point) -> Result<Self> {
        Self::try_new(point.x, point.y)
    }
}

impl TryFrom<kurbo::Rect> for Rect {
    type Error = Error;

    fn try_from(rect: kurbo::Rect) -> Result<Self> {
        let min = Point::try_new(rect.x0, rect.y0)?;
        let max = Point::try_new(rect.x1, rect.y1)?;
        Self::try_from_min_max(min, max)
    }
}
```

Keep `From<Point> for kurbo::Point` and `From<Rect> for kurbo::Rect` because those convert validated crate-owned values outward. Replace public `Transform::translate`, `Transform::scale`, and `Transform::rotate` with the fallible `try_*` helpers listed above. Use crate-private unchecked helpers only for constants and already validated internal values.

- [ ] **Step 5: Update internal call sites**

Replace direct field access in `src/key.rs`, `src/primitive.rs`, and `src/dash.rs` with accessors. Replace internal construction with `new_unchecked` only where the inputs are already validated or derived from validated values. Use `try_new` in public-facing constructors and tests.

- [ ] **Step 6: Regenerate the public API artifact**

Run:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Expected: `api/public-api.txt` no longer lists public fields for geometry values, no longer lists `impl From<kurbo::point::Point> for surgeist_shape::Point`, no longer lists `impl From<kurbo::rect::Rect> for surgeist_shape::Rect`, and includes the new fallible constructors, fallible conversions, and accessors.

- [ ] **Step 7: Run task checks**

```sh
cargo test -p surgeist-shape constructor_rejects
cargo test -p surgeist-shape
cargo clippy -p surgeist-shape --all-targets -- -D warnings
cargo fmt --check
git diff --stat
git status --short --branch
```

Expected: all checks pass; modified files are limited to the Task 2 files and `api/public-api.txt`.

- [ ] **Step 8: Commit Task 2**

```sh
git add src/geometry.rs src/key.rs src/primitive.rs src/dash.rs src/tests.rs api/public-api.txt
git commit -m "model geometry invariants"
```

### Task 3: Separate Authored Path Building From Command-Order-Validated Path Use

**Files:**
- Modify: `src/path.rs`
- Modify: `src/primitive.rs`
- Modify: `src/dash.rs`
- Modify: `src/tests.rs`
- Modify: `src/lib.rs`
- Regenerate: `api/public-api.txt`

- [ ] **Step 1: Write failing path boundary tests**

Add:

```rust
#[test]
fn path_builder_rejects_line_before_move_on_build() {
    let mut builder = PathBuilder::new();
    builder.line_to(Point::try_new(1.0, 1.0).unwrap());

    assert_eq!(builder.build().unwrap_err().code, ErrorCode::InvalidPath);
}

#[test]
fn validated_path_is_not_empty_after_build() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point::try_new(0.0, 0.0).unwrap())
        .line_to(Point::try_new(1.0, 0.0).unwrap());

    let path = builder.build().unwrap();

    assert!(!path.is_empty());
}

#[test]
fn empty_path_is_explicitly_valid_empty_geometry() {
    let path = PathBuilder::new().build().unwrap();

    assert!(path.is_empty());
    assert_eq!(path.bounds(), Rect::empty());
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```sh
cargo test -p surgeist-shape path_builder -- --nocapture
```

Expected: fail because `PathBuilder` does not exist.

- [ ] **Step 3: Refactor `src/path.rs`**

Introduce `PathBuilder` as the mutable authored command collector and make `Path` an immutable command list whose command ordering and point values are validated. Empty paths are valid and represent empty geometry.

```rust
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PathBuilder {
    commands: Vec<Command>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    commands: Vec<Command>,
}

impl PathBuilder {
    pub fn new() -> Self;
    pub fn move_to(&mut self, point: Point) -> &mut Self;
    pub fn line_to(&mut self, point: Point) -> &mut Self;
    pub fn quad_to(&mut self, control: Point, end: Point) -> &mut Self;
    pub fn cubic_to(&mut self, control_a: Point, control_b: Point, end: Point) -> &mut Self;
    pub fn close(&mut self) -> &mut Self;
    pub fn build(self) -> Result<Path>;
}

impl Path {
    pub fn empty() -> Self;
    pub fn from_commands(commands: Vec<Command>) -> Result<Self>;
    pub(crate) fn from_commands_unchecked(commands: Vec<Command>) -> Self;
    pub fn commands(&self) -> &[Command];
}
```

Move the existing order validation into `Path::from_commands`. Keep `Path::validate` as an idempotent public method that re-checks the invariant for callers and tests. Do not derive or implement public `Default` for `Path`; callers should use `Path::empty()` so the empty-geometry choice is explicit.

- [ ] **Step 4: Update shape and dash internals**

Update helpers that build internal generated paths to use `Path::from_commands_unchecked` only after generating a move command first, or `Path::empty()` for intentional empty geometry. Update user-facing path construction in tests to use `PathBuilder::build()`.

- [ ] **Step 5: Export `PathBuilder`**

Update `src/lib.rs`:

```rust
pub use path::{Command, FillRule, Path, PathBuilder};
```

- [ ] **Step 6: Regenerate the public API artifact**

Run:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

- [ ] **Step 7: Run task checks**

```sh
cargo test -p surgeist-shape path_builder
cargo test -p surgeist-shape
cargo clippy -p surgeist-shape --all-targets -- -D warnings
cargo fmt --check
git status --short --branch
```

- [ ] **Step 8: Commit Task 3**

```sh
git add src/path.rs src/primitive.rs src/dash.rs src/tests.rs src/lib.rs api/public-api.txt
git commit -m "model validated paths"
```

### Task 4: Make Shape Construction Fallible And Close Variant Backdoors

**Files:**
- Modify: `src/primitive.rs`
- Modify: `src/lib.rs`
- Modify: `src/tests.rs`
- Regenerate: `api/public-api.txt`

- [ ] **Step 1: Write failing shape invariant tests**

Add:

```rust
#[test]
fn shape_circle_rejects_negative_radius_at_construction() {
    let error = Shape::try_circle(Point::zero(), -1.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::NegativeRadius);
}

#[test]
fn shape_ellipse_accepts_valid_radii_through_front_door() {
    let shape = Shape::try_ellipse(Point::zero(), Size::try_new(2.0, 3.0).unwrap()).unwrap();

    assert_eq!(shape.bounds(), Rect::try_new(-2.0, -3.0, 4.0, 6.0).unwrap());
}
```

Add this path test in the same step so Task 4 proves `Shape` accepts only validated paths:

```rust
#[test]
fn shape_path_accepts_only_validated_path() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point::zero())
        .line_to(Point::try_new(1.0, 0.0).unwrap());
    let path = builder.build().unwrap();
    let shape = Shape::try_path(path, FillRule::NonZero).unwrap();

    assert!(!shape.bounds().is_empty());
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

```sh
cargo test -p surgeist-shape shape_ -- --nocapture
```

Expected: fail because `try_circle`, `try_ellipse`, or `try_path` do not exist.

- [ ] **Step 3: Replace public enum with private representation**

Change `Shape` from a public enum to a public struct with a private enum:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    kind: ShapeKind,
}

#[derive(Clone, Debug, PartialEq)]
enum ShapeKind {
    Rect(Rect),
    RoundedRect { rect: Rect, radii: Radii },
    Circle { center: Point, radius: NonNegative },
    Ellipse { center: Point, radii: Size },
    Path { path: Path, fill_rule: FillRule },
}
```

Required constructors:

```rust
impl Shape {
    pub fn try_rect(rect: Rect) -> Result<Self>;
    pub fn try_rounded_rect(rect: Rect, radii: Radii) -> Result<Self>;
    pub fn try_circle(center: Point, radius: f64) -> Result<Self>;
    pub fn try_ellipse(center: Point, radii: Size) -> Result<Self>;
    pub fn try_path(path: Path, fill_rule: FillRule) -> Result<Self>;
}
```

Keep any infallible convenience only when it cannot fail, such as `Shape::rect(rect)` if `Rect` is already validated by construction.

- [ ] **Step 4: Update methods to match on `self.kind`**

Update `validate`, `bounds`, `visual_bounds`, `contains`, `inflate`, `deflate`, Kurbo conversion, keying, and dash dispatch to use `ShapeKind`. Remove paths where invalid geometry is silently hashed or bounded before validation.

- [ ] **Step 5: Regenerate the public API artifact**

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Expected: `api/public-api.txt` no longer lists `Shape::Circle::radius`, `Shape::Ellipse::radii`, or other public enum fields.

- [ ] **Step 6: Run task checks**

```sh
cargo test -p surgeist-shape shape_
cargo test -p surgeist-shape
cargo clippy -p surgeist-shape --all-targets -- -D warnings
cargo fmt --check
git status --short --branch
```

- [ ] **Step 7: Commit Task 4**

```sh
git add src/primitive.rs src/lib.rs src/tests.rs api/public-api.txt
git commit -m "model shape invariants"
```

### Task 5: Close Stroke, Dash, Anchor, And Dash Output Invariants

**Files:**
- Modify: `src/stroke.rs`
- Modify: `src/dash.rs`
- Modify: `src/primitive.rs`
- Modify: `src/tests.rs`
- Regenerate: `api/public-api.txt`

- [ ] **Step 1: Write failing stroke and dash invariant tests**

Add:

```rust
#[test]
fn stroke_constructor_rejects_negative_width() {
    let error = Stroke::try_new(-1.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidStroke);
}

#[test]
fn dash_constructor_rejects_zero_density() {
    let error = Dash::try_new(0.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidDash);
}

#[test]
fn dash_segment_constructor_rejects_negative_width() {
    let path = {
        let mut builder = PathBuilder::new();
        builder
            .move_to(Point::zero())
            .line_to(Point::try_new(1.0, 0.0).unwrap());
        builder.build().unwrap()
    };
    let error = DashSegment::try_new(path, -1.0, false).unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidStroke);
}

#[test]
fn dash_anchor_rejects_negative_contour_offset() {
    let error = DashAnchor::try_contour_offset(-1.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidDash);
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

```sh
cargo test -p surgeist-shape constructor_rejects -- --nocapture
```

Expected: fail because fallible stroke, dash, segment, and anchor constructors are missing.

- [ ] **Step 3: Refactor `Stroke`**

Make fields private and expose accessors plus validating builders:

```rust
impl Stroke {
    pub fn try_new(width: f64) -> Result<Self>;
    pub fn hairline() -> Self;
    pub fn try_centered(width: f64) -> Result<Self>;
    pub fn try_inside(width: f64) -> Result<Self>;
    pub fn try_outside(width: f64) -> Result<Self>;
    pub fn with_join(self, join: LineJoin) -> Self;
    pub fn with_caps(self, start_cap: LineCap, end_cap: LineCap) -> Self;
    pub fn try_with_miter_limit(self, miter_limit: f64) -> Result<Self>;
    pub fn with_dash(self, dash: Dash) -> Self;
    pub const fn width(self) -> f64;
    pub const fn align(self) -> StrokeAlign;
    pub const fn dash(self) -> Option<Dash>;
}
```

Use `validate_non_negative_kind(..., NumericKind::Stroke)` for width and miter limit.

- [ ] **Step 4: Refactor `Dash`, `DashAnchor`, and `DashSegment`**

Make invalid dash configurations and contour anchors unconstructable:

```rust
pub struct DashAnchor {
    kind: DashAnchorKind,
}

enum DashAnchorKind {
    Corner(Corner),
    ContourOffset(NonNegative),
}

impl DashAnchor {
    pub fn corner(corner: Corner) -> Self;
    pub fn try_contour_offset(offset: f64) -> Result<Self>;
}

impl Dash {
    pub fn try_new(density: f64) -> Result<Self>;
    pub fn dashed() -> Self;
    pub fn dotted() -> Self;
    pub fn try_with_density(self, density: f64) -> Result<Self>;
    pub fn try_with_phase(self, phase: f64) -> Result<Self>;
    pub fn with_sides(self, sides: SideSet) -> Result<Self>;
    pub fn with_anchor(self, anchor: DashAnchor) -> Result<Self>;
    pub fn with_corner_anchors(self) -> Self;
}

impl DashSegment {
    pub fn try_new(path: Path, width: f64, rounded: bool) -> Result<Self>;
    pub(crate) fn new_unchecked(path: Path, width: f64, rounded: bool) -> Self;
    pub fn width(&self) -> f64;
}
```

Keep `Dash::dashed()`, `Dash::dotted()`, and `DashAnchor::corner(...)` infallible because their constants are valid. Make `Dash::with_anchor` return `Result<Self>` and reject more than four anchors with `ErrorCode::InvalidDash` instead of silently dropping the extra anchor. `SideSet` remains an authored side-selection value where `SideSet::none()` is valid standalone; `Dash::with_sides` rejects an empty side set at the dash boundary.

- [ ] **Step 5: Update shape dash dispatch**

Replace `stroke.dash.expect("dash checked by caller")` with an accessor and explicit error handling so dash geometry never depends on panic for an internal precondition.

- [ ] **Step 6: Regenerate the public API artifact**

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

- [ ] **Step 7: Run task checks**

```sh
cargo test -p surgeist-shape constructor_rejects
cargo test -p surgeist-shape
cargo clippy -p surgeist-shape --all-targets -- -D warnings
cargo fmt --check
git status --short --branch
```

- [ ] **Step 8: Commit Task 5**

```sh
git add src/stroke.rs src/dash.rs src/primitive.rs src/tests.rs api/public-api.txt
git commit -m "model stroke and dash invariants"
```

### Task 6: Final Compliance Pass And Documentation

**Files:**
- Modify: `README.md`
- Modify: `src/tests.rs`
- Regenerate: `api/public-api.txt`

- [ ] **Step 1: Add compliance regression tests**

Add tests that exercise the new front doors together:

```rust
#[test]
fn public_front_doors_construct_valid_shape_and_dash_geometry() {
    let rect = Rect::try_new(0.0, 0.0, 80.0, 40.0).unwrap();
    let shape = Shape::try_rounded_rect(rect, Radii::try_all(8.0).unwrap()).unwrap();
    let dash = Dash::dashed().with_corner_anchors();
    let stroke = Stroke::try_centered(2.0).unwrap().with_dash(dash);
    let geometry = shape.dashed_stroke(stroke).unwrap();

    assert!(!geometry.is_empty());
}

#[test]
fn invalid_public_models_do_not_have_literal_backdoors() {
    let api = std::fs::read_to_string("api/public-api.txt").unwrap();

    assert!(!api.contains("pub surgeist_shape::Stroke::width"));
    assert!(!api.contains("pub surgeist_shape::DashSegment::width"));
    assert!(!api.contains("pub surgeist_shape::Rect::size"));
    assert!(!api.contains("pub surgeist_shape::Shape::Circle::radius"));
    assert!(!api.contains("pub surgeist_shape::Point::x"));
    assert!(!api.contains("pub surgeist_shape::Size::width"));
    assert!(!api.contains("pub struct surgeist_shape::Transform(pub [f64; 6])"));
    assert!(!api.contains("pub surgeist_shape::DashAnchor::ContourOffset(f64)"));
    assert!(!api.contains("impl core::convert::From<kurbo::point::Point> for surgeist_shape::Point"));
    assert!(!api.contains("impl core::convert::From<kurbo::rect::Rect> for surgeist_shape::Rect"));
    assert!(!api.contains("pub fn surgeist_shape::Point::translate("));
    assert!(!api.contains("pub fn surgeist_shape::Rect::inset("));
    assert!(!api.contains("pub fn surgeist_shape::Rect::outset("));
    assert!(!api.contains("pub fn surgeist_shape::Rect::translate("));
    assert!(!api.contains("pub fn surgeist_shape::Radii::inset("));
    assert!(!api.contains("pub fn surgeist_shape::Radii::outset("));
    assert!(!api.contains("pub fn surgeist_shape::Transform::then("));
    assert!(!api.contains("pub fn surgeist_shape::Transform::apply_point("));
    assert!(!api.contains("pub fn surgeist_shape::Transform::apply_rect("));
}
```

Also inspect `api/public-api.txt` manually for remaining public fields or infallible constructors that can create non-finite geometry, negative sizes/radii, invalid strokes, invalid dashes, invalid dash anchors, or unordered paths. Record any remaining intentional authored-data exceptions in this plan before the final holistic review.

- [ ] **Step 2: Update README API note**

Add this section to `README.md`:

```markdown
## Modeling Contract

`surgeist-shape` owns resolved geometry and shape-boundary invariants. Public
constructors validate finite coordinates, non-negative dimensions, valid path
ordering, stroke geometry, and dash geometry before values enter normal use.
The committed API artifact in `api/public-api.txt` records these front doors.
```

- [ ] **Step 3: Regenerate public API artifact**

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

- [ ] **Step 4: Run final checks**

```sh
cargo test -p surgeist-shape
cargo clippy -p surgeist-shape --all-targets -- -D warnings
cargo fmt --check
git diff --stat
git status --short --branch
```

- [ ] **Step 5: Commit Task 6**

```sh
git add README.md src/tests.rs api/public-api.txt
git commit -m "document shape modeling contract"
```

## Final Holistic Review Gate

- [ ] Assign a final reviewer to inspect the complete branch against:
  - `guidance/surgeist-rust-modeling-guide.md`
  - this implementation plan
  - `AGENTS.md` crate boundaries
  - `api/public-api.txt`
  - final check outputs
- [ ] Fix any Critical or Important reviewer findings.
- [ ] Re-run:

```sh
cargo test -p surgeist-shape
cargo clippy -p surgeist-shape --all-targets -- -D warnings
cargo fmt --check
git status --short --branch
```

- [ ] Mark the goal complete only after the reviewer reports no blocking findings.

## Self-Review

Spec coverage: The plan covers the modeling guide findings for invalid public construction, phase-specific paths, shape invariants, dash/stroke invariants, semantic errors, API artifacts, crate boundaries, focused checks, task-scoped commits, and a final holistic review.

Placeholder scan: No red-flag placeholder phrases or unspecified error-handling steps remain.

Type consistency: The plan consistently uses `Finite`, `NonNegative`, `NumericKind`, `PathBuilder`, private `ShapeKind`, `try_*` constructors, accessors, and `api/public-api.txt` regeneration across dependent tasks.
