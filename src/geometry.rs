use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    #[must_use]
    pub fn translate(self, x: f64, y: f64) -> Self {
        Self::new(self.x + x, self.y + y)
    }

    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    #[must_use]
    pub fn to_kurbo(self) -> kurbo::Point {
        kurbo::Point::new(self.x, self.y)
    }

    pub(crate) fn validate(self, name: &str) -> Result<()> {
        validate_finite(self.x, &format!("{name} x"))?;
        validate_finite(self.y, &format!("{name} y"))
    }
}

impl From<Point> for kurbo::Point {
    fn from(point: Point) -> Self {
        point.to_kurbo()
    }
}

impl From<kurbo::Point> for Point {
    fn from(point: kurbo::Point) -> Self {
        Self::new(point.x, point.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    #[must_use]
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    #[must_use]
    pub fn min_side(self) -> f64 {
        self.width.min(self.height)
    }

    #[must_use]
    pub fn max_side(self) -> f64 {
        self.width.max(self.height)
    }

    pub(crate) fn validate(self, name: &str) -> Result<()> {
        validate_non_negative(self.width, &format!("{name} width"))?;
        validate_non_negative(self.height, &format!("{name} height"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const ZERO: Self = Self {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    #[must_use]
    pub fn from_min_max(min: Point, max: Point) -> Self {
        Self::new(
            min.x,
            min.y,
            (max.x - min.x).max(0.0),
            (max.y - min.y).max(0.0),
        )
    }

    #[must_use]
    pub const fn from_origin_size(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    #[must_use]
    pub fn from_center_size(center: Point, size: Size) -> Self {
        Self::new(
            center.x - size.width * 0.5,
            center.y - size.height * 0.5,
            size.width,
            size.height,
        )
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self::ZERO
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.size.is_empty()
    }

    #[must_use]
    pub fn min(self) -> Point {
        self.origin
    }

    #[must_use]
    pub fn max(self) -> Point {
        Point::new(
            self.origin.x + self.size.width,
            self.origin.y + self.size.height,
        )
    }

    #[must_use]
    pub fn center(self) -> Point {
        Point::new(
            self.origin.x + self.size.width * 0.5,
            self.origin.y + self.size.height * 0.5,
        )
    }

    #[must_use]
    pub fn width(self) -> f64 {
        self.size.width
    }

    #[must_use]
    pub fn height(self) -> f64 {
        self.size.height
    }

    #[must_use]
    pub fn size(self) -> Size {
        self.size
    }

    #[must_use]
    pub fn contains(self, point: Point) -> bool {
        let max = self.max();
        point.x >= self.origin.x && point.x <= max.x && point.y >= self.origin.y && point.y <= max.y
    }

    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        let a = self.max();
        let b = other.max();
        self.origin.x < b.x && a.x > other.origin.x && self.origin.y < b.y && a.y > other.origin.y
    }

    #[must_use]
    pub fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        let a = self.max();
        let b = other.max();
        Self::from_min_max(
            Point::new(
                self.origin.x.min(other.origin.x),
                self.origin.y.min(other.origin.y),
            ),
            Point::new(a.x.max(b.x), a.y.max(b.y)),
        )
    }

    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        let a = self.max();
        let b = other.max();
        Self::from_min_max(
            Point::new(
                self.origin.x.max(other.origin.x),
                self.origin.y.max(other.origin.y),
            ),
            Point::new(a.x.min(b.x), a.y.min(b.y)),
        )
    }

    #[must_use]
    pub fn inset(self, insets: Insets) -> Self {
        let width = (self.size.width - insets.left - insets.right).max(0.0);
        let height = (self.size.height - insets.top - insets.bottom).max(0.0);
        Self::new(
            self.origin.x + insets.left,
            self.origin.y + insets.top,
            width,
            height,
        )
    }

    #[must_use]
    pub fn outset(self, insets: Insets) -> Self {
        Self::new(
            self.origin.x - insets.left,
            self.origin.y - insets.top,
            self.size.width + insets.left + insets.right,
            self.size.height + insets.top + insets.bottom,
        )
    }

    #[must_use]
    pub fn translate(self, x: f64, y: f64) -> Self {
        Self::from_origin_size(self.origin.translate(x, y), self.size)
    }

    #[must_use]
    pub fn transformed_bounds(self, transform: Transform) -> Self {
        transform.apply_rect(self)
    }

    #[must_use]
    pub fn to_kurbo(self) -> kurbo::Rect {
        let max = self.max();
        kurbo::Rect::new(self.origin.x, self.origin.y, max.x, max.y)
    }

    pub(crate) fn validate(self, name: &str) -> Result<()> {
        self.origin.validate(name)?;
        self.size.validate(name)
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<Rect> for kurbo::Rect {
    fn from(rect: Rect) -> Self {
        rect.to_kurbo()
    }
}

impl From<kurbo::Rect> for Rect {
    fn from(rect: kurbo::Rect) -> Self {
        Self::from_min_max(Point::new(rect.x0, rect.y0), Point::new(rect.x1, rect.y1))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Insets {
    #[must_use]
    pub const fn all(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    #[must_use]
    pub const fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::all(0.0)
    }

    #[must_use]
    pub fn horizontal(self) -> f64 {
        self.left + self.right
    }

    #[must_use]
    pub fn vertical(self) -> f64 {
        self.top + self.bottom
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.top == 0.0 && self.right == 0.0 && self.bottom == 0.0 && self.left == 0.0
    }

    pub(crate) fn validate(self, name: &str) -> Result<()> {
        validate_finite(self.top, &format!("{name} top"))?;
        validate_finite(self.right, &format!("{name} right"))?;
        validate_finite(self.bottom, &format!("{name} bottom"))?;
        validate_finite(self.left, &format!("{name} left"))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Radii {
    pub top_left: f64,
    pub top_right: f64,
    pub bottom_right: f64,
    pub bottom_left: f64,
}

impl Radii {
    #[must_use]
    pub const fn all(radius: f64) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    #[must_use]
    pub const fn new(top_left: f64, top_right: f64, bottom_right: f64, bottom_left: f64) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::all(0.0)
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
    }

    #[must_use]
    pub fn is_uniform(self) -> bool {
        self.top_left == self.top_right
            && self.top_left == self.bottom_right
            && self.top_left == self.bottom_left
    }

    pub fn normalized_for(self, rect: Rect) -> Result<Self> {
        self.validate()?;
        rect.validate("radius rect")?;
        let mut radii = self;
        let top = radii.top_left + radii.top_right;
        let bottom = radii.bottom_left + radii.bottom_right;
        let left = radii.top_left + radii.bottom_left;
        let right = radii.top_right + radii.bottom_right;
        let mut factor: f64 = 1.0;
        if top > 0.0 {
            factor = factor.min(rect.width() / top);
        }
        if bottom > 0.0 {
            factor = factor.min(rect.width() / bottom);
        }
        if left > 0.0 {
            factor = factor.min(rect.height() / left);
        }
        if right > 0.0 {
            factor = factor.min(rect.height() / right);
        }
        factor = factor.clamp(0.0, 1.0);
        radii.top_left *= factor;
        radii.top_right *= factor;
        radii.bottom_right *= factor;
        radii.bottom_left *= factor;
        Ok(radii)
    }

    #[must_use]
    pub fn inset(self, amount: f64) -> Self {
        Self::new(
            (self.top_left - amount).max(0.0),
            (self.top_right - amount).max(0.0),
            (self.bottom_right - amount).max(0.0),
            (self.bottom_left - amount).max(0.0),
        )
    }

    #[must_use]
    pub fn outset(self, amount: f64) -> Self {
        Self::new(
            (self.top_left + amount).max(0.0),
            (self.top_right + amount).max(0.0),
            (self.bottom_right + amount).max(0.0),
            (self.bottom_left + amount).max(0.0),
        )
    }

    #[must_use]
    pub fn to_kurbo(self) -> kurbo::RoundedRectRadii {
        kurbo::RoundedRectRadii::new(
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        )
    }

    fn validate(self) -> Result<()> {
        validate_non_negative(self.top_left, "top-left radius")?;
        validate_non_negative(self.top_right, "top-right radius")?;
        validate_non_negative(self.bottom_right, "bottom-right radius")?;
        validate_non_negative(self.bottom_left, "bottom-left radius")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform(pub [f64; 6]);

impl Transform {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    #[must_use]
    pub const fn identity() -> Self {
        Self::IDENTITY
    }

    #[must_use]
    pub const fn translate(x: f64, y: f64) -> Self {
        Self([1.0, 0.0, 0.0, 1.0, x, y])
    }

    #[must_use]
    pub const fn scale(x: f64, y: f64) -> Self {
        Self([x, 0.0, 0.0, y, 0.0, 0.0])
    }

    #[must_use]
    pub fn rotate(radians: f64) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self([cos, sin, -sin, cos, 0.0, 0.0])
    }

    #[must_use]
    pub fn then(self, next: Self) -> Self {
        let [a, b, c, d, e, f] = self.0;
        let [g, h, i, j, k, l] = next.0;
        Self([
            a * g + c * h,
            b * g + d * h,
            a * i + c * j,
            b * i + d * j,
            a * k + c * l + e,
            b * k + d * l + f,
        ])
    }

    #[must_use]
    pub fn inverse(self) -> Option<Self> {
        let [a, b, c, d, e, f] = self.0;
        let det = a * d - b * c;
        if det.abs() <= f64::EPSILON {
            return None;
        }
        let inv = 1.0 / det;
        Some(Self([
            d * inv,
            -b * inv,
            -c * inv,
            a * inv,
            (c * f - d * e) * inv,
            (b * e - a * f) * inv,
        ]))
    }

    #[must_use]
    pub fn apply_point(self, point: Point) -> Point {
        let [a, b, c, d, e, f] = self.0;
        Point::new(a * point.x + c * point.y + e, b * point.x + d * point.y + f)
    }

    #[must_use]
    pub fn apply_rect(self, rect: Rect) -> Rect {
        let min = rect.min();
        let max = rect.max();
        let points = [
            self.apply_point(min),
            self.apply_point(Point::new(max.x, min.y)),
            self.apply_point(max),
            self.apply_point(Point::new(min.x, max.y)),
        ];
        bounds_of_points(&points).unwrap_or_default()
    }

    #[must_use]
    pub fn to_kurbo(self) -> kurbo::Affine {
        kurbo::Affine::new(self.0)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

fn bounds_of_points(points: &[Point]) -> Option<Rect> {
    let first = points.first()?;
    let mut min_x = first.x;
    let mut min_y = first.y;
    let mut max_x = first.x;
    let mut max_y = first.y;
    for point in &points[1..] {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Some(Rect::from_min_max(
        Point::new(min_x, min_y),
        Point::new(max_x, max_y),
    ))
}
