//! Resolved geometry and shape boundary for Surgeist.
//!
//! This module owns logical geometry, shape normalization, bounds, containment,
//! path conversion, stable geometry keys, and first-pass UI dash geometry. It
//! does not own style resolution, layout, rendering, GPU resources, widgets, or
//! application behavior.

mod bounds;
mod dash;
mod error;
mod geometry;
mod key;
mod path;
mod primitive;
mod stroke;

#[cfg(test)]
mod tests;

pub use bounds::{Bounds, BoundsKind};
pub use dash::{
    Corner, Dash, DashAnchor, DashConstraint, DashGeometry, DashSegment, Side, SideSet,
};
pub use error::{Error, ErrorCode, Result};
pub use geometry::{Insets, Point, Radii, Rect, Size, Transform};
pub use key::Key;
pub use path::{Command, FillRule, Path};
pub use primitive::Shape;
pub use stroke::{LineCap, LineJoin, Stroke, StrokeAlign};

pub(crate) use dash::{dash_ellipse, dash_rect};
pub(crate) use error::{validate_finite, validate_non_negative};
pub(crate) use key::{StableHasher, hash_point, hash_radii, hash_rect};
pub(crate) use path::{path_from_points, path_polyline_length};

pub(crate) const PATH_TOLERANCE: f64 = 0.1;
pub(crate) const CORNER_ARC_STEPS: usize = 8;
