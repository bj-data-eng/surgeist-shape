use super::*;
use kurbo::Shape as KurboShape;

#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Rect(Rect),
    RoundedRect { rect: Rect, radii: Radii },
    Circle { center: Point, radius: f64 },
    Ellipse { center: Point, radii: Size },
    Path { path: Path, fill_rule: FillRule },
}

impl Shape {
    #[must_use]
    pub const fn rect(rect: Rect) -> Self {
        Self::Rect(rect)
    }

    #[must_use]
    pub const fn rounded_rect(rect: Rect, radii: Radii) -> Self {
        Self::RoundedRect { rect, radii }
    }

    #[must_use]
    pub const fn circle(center: Point, radius: f64) -> Self {
        Self::Circle { center, radius }
    }

    #[must_use]
    pub const fn ellipse(center: Point, radii: Size) -> Self {
        Self::Ellipse { center, radii }
    }

    #[must_use]
    pub const fn path(path: Path, fill_rule: FillRule) -> Self {
        Self::Path { path, fill_rule }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Shape::Rect(rect) => rect.validate("rect"),
            Shape::RoundedRect { rect, radii } => {
                rect.validate("rounded rect")?;
                radii.normalized_for(*rect).map(|_| ())
            }
            Shape::Circle { center, radius } => {
                center.validate("circle center")?;
                validate_non_negative(*radius, "circle radius")
            }
            Shape::Ellipse { center, radii } => {
                center.validate("ellipse center")?;
                radii.validate("ellipse radii")
            }
            Shape::Path { path, .. } => path.validate(),
        }
    }

    pub fn bounds(&self) -> Rect {
        match self {
            Shape::Rect(rect) | Shape::RoundedRect { rect, .. } => *rect,
            Shape::Circle { center, radius } => Rect::new(
                center.x - radius,
                center.y - radius,
                radius * 2.0,
                radius * 2.0,
            ),
            Shape::Ellipse { center, radii } => Rect::new(
                center.x - radii.width,
                center.y - radii.height,
                radii.width * 2.0,
                radii.height * 2.0,
            ),
            Shape::Path { path, .. } => path.bounds(),
        }
    }

    pub fn visual_bounds(&self, stroke: Option<Stroke>) -> Result<Rect> {
        let Some(stroke) = stroke else {
            return Ok(self.bounds());
        };
        stroke.validate()?;
        let outset = match stroke.align {
            StrokeAlign::Center => stroke.width * 0.5,
            StrokeAlign::Inside => 0.0,
            StrokeAlign::Outside => stroke.width,
        };
        if matches!(self, Shape::Path { .. }) && stroke.align != StrokeAlign::Center {
            return Err(Error::new(
                ErrorCode::UnsupportedStrokeBounds,
                "inside and outside stroke bounds for arbitrary paths are not supported",
            ));
        }
        Ok(self.bounds().outset(Insets::all(outset)))
    }

    pub fn support_bounds(&self, outset: Insets) -> Result<Rect> {
        outset.validate("support outset")?;
        Ok(self.bounds().outset(outset))
    }

    #[must_use]
    pub fn transformed_bounds(&self, transform: Transform) -> Rect {
        transform.apply_rect(self.bounds())
    }

    pub fn contains(&self, point: Point) -> bool {
        match self {
            Shape::Rect(rect) => rect.contains(point),
            Shape::RoundedRect { rect, radii } => rounded_rect_contains(*rect, *radii, point),
            Shape::Circle { center, radius } => center.distance_to(point) <= *radius,
            Shape::Ellipse { center, radii } => {
                if radii.width <= 0.0 || radii.height <= 0.0 {
                    return false;
                }
                let x = (point.x - center.x) / radii.width;
                let y = (point.y - center.y) / radii.height;
                x * x + y * y <= 1.0
            }
            Shape::Path { path, fill_rule } => path.contains(point, *fill_rule),
        }
    }

    pub fn inflate(&self, amount: f64) -> Result<Self> {
        validate_finite(amount, "inflate amount")?;
        Ok(match self {
            Shape::Rect(rect) => Shape::Rect(rect.outset(Insets::all(amount))),
            Shape::RoundedRect { rect, radii } => Shape::RoundedRect {
                rect: rect.outset(Insets::all(amount)),
                radii: radii.outset(amount),
            },
            Shape::Circle { center, radius } => Shape::Circle {
                center: *center,
                radius: (radius + amount).max(0.0),
            },
            Shape::Ellipse { center, radii } => Shape::Ellipse {
                center: *center,
                radii: Size::new(
                    (radii.width + amount).max(0.0),
                    (radii.height + amount).max(0.0),
                ),
            },
            Shape::Path { .. } => {
                return Err(Error::new(
                    ErrorCode::UnsupportedStrokeBounds,
                    "inflating arbitrary paths is not supported",
                ));
            }
        })
    }

    pub fn deflate(&self, amount: f64) -> Result<Self> {
        self.inflate(-amount)
    }

    pub fn to_path(&self) -> Result<Path> {
        Ok(Path::from_kurbo(self.to_kurbo_path()?))
    }

    pub fn to_kurbo_path(&self) -> Result<kurbo::BezPath> {
        self.validate()?;
        Ok(match self {
            Shape::Rect(rect) => rect.to_kurbo().to_path(PATH_TOLERANCE),
            Shape::RoundedRect { rect, radii } => {
                let radii = radii.normalized_for(*rect)?;
                kurbo::RoundedRect::from_rect(rect.to_kurbo(), radii.to_kurbo())
                    .to_path(PATH_TOLERANCE)
            }
            Shape::Circle { center, radius } => {
                kurbo::Circle::new(center.to_kurbo(), *radius).to_path(PATH_TOLERANCE)
            }
            Shape::Ellipse { center, radii } => {
                kurbo::Ellipse::new(center.to_kurbo(), (radii.width, radii.height), 0.0)
                    .to_path(PATH_TOLERANCE)
            }
            Shape::Path { path, .. } => path.to_kurbo(),
        })
    }

    #[must_use]
    pub fn to_kurbo_rect(&self) -> Option<kurbo::Rect> {
        match self {
            Shape::Rect(rect) => Some(rect.to_kurbo()),
            _ => None,
        }
    }

    pub fn to_kurbo_rounded_rect(&self) -> Option<kurbo::RoundedRect> {
        match self {
            Shape::RoundedRect { rect, radii } => Some(kurbo::RoundedRect::from_rect(
                rect.to_kurbo(),
                radii.normalized_for(*rect).ok()?.to_kurbo(),
            )),
            _ => None,
        }
    }

    #[must_use]
    pub fn key(&self) -> Key {
        let mut state = StableHasher::new(0xcbf2_9ce4_8422_2325);
        match self {
            Shape::Rect(rect) => {
                state.write_u8(1);
                hash_rect(&mut state, *rect);
            }
            Shape::RoundedRect { rect, radii } => {
                state.write_u8(2);
                hash_rect(&mut state, *rect);
                hash_radii(&mut state, radii.normalized_for(*rect).unwrap_or(*radii));
            }
            Shape::Circle { center, radius } => {
                state.write_u8(3);
                hash_point(&mut state, *center);
                state.write_f64(*radius);
            }
            Shape::Ellipse { center, radii } => {
                state.write_u8(4);
                hash_point(&mut state, *center);
                state.write_f64(radii.width);
                state.write_f64(radii.height);
            }
            Shape::Path { path, fill_rule } => {
                state.write_u8(5);
                path.hash_with_rule(&mut state, *fill_rule);
            }
        }
        Key::from_parts(
            state.finish(),
            state.finish_with_seed(0x9e37_79b9_7f4a_7c15),
        )
    }

    pub fn dashed_stroke(&self, stroke: Stroke) -> Result<DashGeometry> {
        stroke.validate()?;
        let dash = stroke
            .dash
            .ok_or_else(|| Error::new(ErrorCode::InvalidDash, "stroke has no dash geometry"))?;
        dash.validate()?;
        match self {
            Shape::Rect(rect) => dash_rect(*rect, Radii::zero(), stroke),
            Shape::RoundedRect { rect, radii } => {
                dash_rect(*rect, radii.normalized_for(*rect)?, stroke)
            }
            Shape::Circle { center, radius } => {
                dash_ellipse(*center, Size::new(*radius, *radius), stroke)
            }
            Shape::Ellipse { center, radii } => dash_ellipse(*center, *radii, stroke),
            Shape::Path { .. } => Err(Error::new(
                ErrorCode::InvalidDash,
                "dash geometry for arbitrary paths is not part of the first implementation",
            )),
        }
    }
}

fn rounded_rect_contains(rect: Rect, radii: Radii, point: Point) -> bool {
    if !rect.contains(point) {
        return false;
    }
    let radii = radii.normalized_for(rect).unwrap_or(radii);
    let max = rect.max();
    for (corner, radius, cx, cy) in [
        (
            Corner::TopLeft,
            radii.top_left,
            rect.origin.x + radii.top_left,
            rect.origin.y + radii.top_left,
        ),
        (
            Corner::TopRight,
            radii.top_right,
            max.x - radii.top_right,
            rect.origin.y + radii.top_right,
        ),
        (
            Corner::BottomRight,
            radii.bottom_right,
            max.x - radii.bottom_right,
            max.y - radii.bottom_right,
        ),
        (
            Corner::BottomLeft,
            radii.bottom_left,
            rect.origin.x + radii.bottom_left,
            max.y - radii.bottom_left,
        ),
    ] {
        if radius <= 0.0 {
            continue;
        }
        let in_corner = match corner {
            Corner::TopLeft => point.x < cx && point.y < cy,
            Corner::TopRight => point.x > cx && point.y < cy,
            Corner::BottomRight => point.x > cx && point.y > cy,
            Corner::BottomLeft => point.x < cx && point.y > cy,
        };
        if in_corner && Point::new(cx, cy).distance_to(point) > radius {
            return false;
        }
    }
    true
}
