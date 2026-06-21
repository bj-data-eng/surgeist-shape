use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    pub width: f64,
    pub join: LineJoin,
    pub start_cap: LineCap,
    pub end_cap: LineCap,
    pub miter_limit: f64,
    pub dash: Option<Dash>,
    pub align: StrokeAlign,
}

impl Stroke {
    #[must_use]
    pub fn new(width: f64) -> Self {
        Self {
            width,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn hairline() -> Self {
        Self::new(1.0)
    }

    #[must_use]
    pub fn centered(width: f64) -> Self {
        Self {
            align: StrokeAlign::Center,
            ..Self::new(width)
        }
    }

    #[must_use]
    pub fn inside(width: f64) -> Self {
        Self {
            align: StrokeAlign::Inside,
            ..Self::new(width)
        }
    }

    #[must_use]
    pub fn outside(width: f64) -> Self {
        Self {
            align: StrokeAlign::Outside,
            ..Self::new(width)
        }
    }

    pub fn validate(self) -> Result<()> {
        validate_non_negative(self.width, "stroke width")?;
        validate_non_negative(self.miter_limit, "stroke miter limit")?;
        if let Some(dash) = self.dash {
            dash.validate()?;
        }
        Ok(())
    }

    pub fn bounds_for(self, shape: &Shape) -> Result<Rect> {
        shape.visual_bounds(Some(self))
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            join: LineJoin::Miter,
            start_cap: LineCap::Butt,
            end_cap: LineCap::Butt,
            miter_limit: 4.0,
            dash: None,
            align: StrokeAlign::Center,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StrokeAlign {
    #[default]
    Center,
    Inside,
    Outside,
}
