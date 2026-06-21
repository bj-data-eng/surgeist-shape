use super::dash::{
    DashMetrics, RectContour, centered_closed_points, centered_points_from_anchors,
    corner_anchor_angle, corner_center, corner_radius, ellipse_point, ellipse_polyline,
    ellipse_steps, polyline_length,
};
use super::*;

#[test]
fn radii_normalization_reduces_asymmetric_corners() {
    let rect = Rect::new(0.0, 0.0, 10.0, 8.0);
    let radii = Radii::new(8.0, 8.0, 4.0, 4.0).normalized_for(rect).unwrap();

    assert_eq!(radii.top_left, 5.0);
    assert_eq!(radii.top_right, 5.0);
    assert_eq!(radii.bottom_right, 2.5);
    assert_eq!(radii.bottom_left, 2.5);
}

#[test]
fn path_rejects_line_before_move() {
    let mut path = Path::new();
    path.line_to(Point::new(1.0, 1.0));

    assert_eq!(path.validate().unwrap_err().code, ErrorCode::InvalidPath);
}

#[test]
fn path_fill_rule_affects_key() {
    let mut path = Path::new();
    path.move_to(Point::new(0.0, 0.0))
        .line_to(Point::new(10.0, 0.0))
        .line_to(Point::new(10.0, 10.0))
        .close();

    assert_ne!(path.key(FillRule::NonZero), path.key(FillRule::EvenOdd));
    assert_ne!(
        Shape::path(path.clone(), FillRule::NonZero).key(),
        Shape::path(path, FillRule::EvenOdd).key()
    );
}

#[test]
fn shape_keys_are_stable_and_distinguish_normalized_geometry() {
    let rect = Shape::rounded_rect(Rect::new(0.0, 0.0, 10.0, 8.0), Radii::all(12.0));
    let same = Shape::rounded_rect(Rect::new(0.0, 0.0, 10.0, 8.0), Radii::all(20.0));
    let different = Shape::rounded_rect(Rect::new(0.0, 0.0, 12.0, 8.0), Radii::all(20.0));

    assert_eq!(rect.key(), rect.key());
    assert_eq!(rect.key(), same.key());
    assert_ne!(rect.key(), different.key());
}

#[test]
fn visual_bounds_respect_stroke_alignment() {
    let shape = Shape::rect(Rect::new(10.0, 20.0, 30.0, 40.0));

    assert_eq!(
        shape.visual_bounds(Some(Stroke::inside(8.0))).unwrap(),
        Rect::new(10.0, 20.0, 30.0, 40.0)
    );
    assert_eq!(
        shape.visual_bounds(Some(Stroke::centered(8.0))).unwrap(),
        Rect::new(6.0, 16.0, 38.0, 48.0)
    );
    assert_eq!(
        shape.visual_bounds(Some(Stroke::outside(8.0))).unwrap(),
        Rect::new(2.0, 12.0, 46.0, 56.0)
    );
}

#[test]
fn dash_rejects_non_finite_phase() {
    let stroke = Stroke {
        dash: Some(Dash::dashed().with_phase(f64::NAN)),
        ..Stroke::default()
    };

    assert_eq!(
        Shape::rect(Rect::new(0.0, 0.0, 80.0, 40.0))
            .dashed_stroke(stroke)
            .unwrap_err()
            .code,
        ErrorCode::NonFinite
    );
}

#[test]
fn support_bounds_are_explicit_outsets() {
    let shape = Shape::rect(Rect::new(10.0, 20.0, 30.0, 40.0));
    let bounds = shape
        .support_bounds(Insets::new(1.0, 2.0, 3.0, 4.0))
        .unwrap();

    assert_eq!(bounds, Rect::new(6.0, 19.0, 36.0, 44.0));
}

#[test]
fn converts_rect_to_path() {
    let path = Shape::rect(Rect::new(0.0, 0.0, 10.0, 20.0))
        .to_path()
        .unwrap();

    assert!(!path.is_empty());
}

#[test]
fn side_scoped_dashes_emit_only_included_sides() {
    let dash = Dash::dashed()
        .with_sides(SideSet::top())
        .with_corner_anchors();
    let stroke = Stroke {
        width: 2.0,
        dash: Some(dash),
        ..Stroke::default()
    };
    let geometry = Shape::rect(Rect::new(0.0, 0.0, 80.0, 40.0))
        .dashed_stroke(stroke)
        .unwrap();

    assert!(!geometry.is_empty());
    assert!(geometry.segments().iter().all(|segment| {
        segment
            .path()
            .commands()
            .iter()
            .all(|command| match *command {
                Command::MoveTo(point) | Command::LineTo(point) => point.y <= 2.0,
                _ => true,
            })
    }));
}

#[test]
fn dash_phase_offsets_resolved_geometry() {
    let base = Stroke {
        width: 4.0,
        dash: Some(Dash::dashed()),
        ..Stroke::default()
    };
    let phased = Stroke {
        width: 4.0,
        dash: Some(Dash::dashed().with_phase(10.0)),
        ..Stroke::default()
    };
    let shape = Shape::rect(Rect::new(0.0, 0.0, 80.0, 40.0));

    assert_ne!(
        shape.dashed_stroke(base).unwrap(),
        shape.dashed_stroke(phased).unwrap()
    );
}

#[test]
fn corner_dash_directions_stay_inside_rect_bounds() {
    let dash = Dash::dashed().with_corner_anchors();
    let stroke = Stroke {
        width: 4.0,
        dash: Some(dash),
        ..Stroke::default()
    };
    let rect = Rect::new(0.0, 0.0, 80.0, 40.0);
    let geometry = Shape::rect(rect).dashed_stroke(stroke).unwrap();

    assert!(geometry.segments().iter().all(|segment| {
        segment
            .path()
            .commands()
            .iter()
            .all(|command| match *command {
                Command::MoveTo(point) | Command::LineTo(point) => rect.contains(point),
                Command::QuadTo { control, end } => rect.contains(control) && rect.contains(end),
                Command::CubicTo {
                    control_a,
                    control_b,
                    end,
                } => rect.contains(control_a) && rect.contains(control_b) && rect.contains(end),
                Command::Close => true,
            })
    }));
}

#[test]
fn circular_one_sided_corner_dots_are_omitted() {
    let dash = Dash::dotted()
        .with_sides(SideSet::top())
        .with_corner_anchors();
    let stroke = Stroke {
        width: 4.0,
        dash: Some(dash),
        ..Stroke::default()
    };
    let geometry = Shape::rect(Rect::new(0.0, 0.0, 80.0, 40.0))
        .dashed_stroke(stroke)
        .unwrap();

    assert!(
        geometry
            .segments()
            .iter()
            .all(|segment| segment.contour_length() > 0.0)
    );
}

#[test]
fn ellipse_dashes_have_stable_output() {
    let dash = Dash::dashed().with_density(1.5);
    let stroke = Stroke {
        width: 2.0,
        dash: Some(dash),
        ..Stroke::default()
    };
    let geometry = Shape::ellipse(Point::new(20.0, 20.0), Size::new(15.0, 10.0))
        .dashed_stroke(stroke)
        .unwrap();

    assert!(!geometry.is_empty());
    assert_eq!(
        geometry,
        Shape::ellipse(Point::new(20.0, 20.0), Size::new(15.0, 10.0))
            .dashed_stroke(stroke)
            .unwrap()
    );
}

#[test]
fn ellipse_dashes_use_consistent_concave_arc_lengths() {
    let dash = Dash::dashed().with_density(1.2).rounded();
    let stroke = Stroke {
        width: 4.0,
        dash: Some(dash),
        ..Stroke::default()
    };
    let metrics = DashMetrics::resolve(stroke, dash);
    let radii = Size::new(40.0, 18.0);
    let steps = ellipse_steps(radii);
    let measure = ellipse_polyline(
        Point::new(20.0, 20.0),
        Size::new(
            (radii.width - metrics.measure_inset).max(0.001),
            (radii.height - metrics.measure_inset).max(0.001),
        ),
        steps,
    );
    let measure_total = polyline_length(&closed_measure_points(&measure));
    let centers = centered_closed_points(measure_total, metrics);
    let geometry = Shape::ellipse(Point::new(20.0, 20.0), Size::new(40.0, 18.0))
        .dashed_stroke(stroke)
        .unwrap();

    assert_eq!(geometry.segments().len(), centers.len());
    assert!(centers.windows(2).all(|pair| pair[1] > pair[0]));
}

#[test]
fn rounded_rect_dashes_include_corner_anchors_without_overlap() {
    let dash = Dash::dashed().with_density(1.1);
    let stroke = Stroke {
        width: 6.0,
        dash: Some(dash),
        ..Stroke::default()
    };
    let rect = Rect::new(0.0, 0.0, 120.0, 80.0);
    let geometry = Shape::rounded_rect(rect, Radii::all(20.0))
        .dashed_stroke(stroke)
        .unwrap();

    assert!(geometry.segments().len() > 8);
    assert!(
        geometry
            .segments()
            .iter()
            .all(|segment| match segment.path().commands()[0] {
                Command::MoveTo(point) => rect.contains(point),
                _ => false,
            })
    );
}

#[test]
fn rounded_rect_corner_points_are_distribution_anchors() {
    let dash = Dash::dashed().with_density(1.0);
    let stroke = Stroke {
        width: 6.0,
        dash: Some(dash),
        ..Stroke::default()
    };
    let metrics = DashMetrics::resolve(stroke, dash);
    let contour = RectContour::new(Rect::new(0.0, 0.0, 120.0, 80.0), Radii::all(24.0), metrics);
    let anchors = contour.corner_offsets();
    let centers = centered_points_from_anchors(contour.total, &anchors, true, metrics);

    for anchor in anchors {
        assert!(
            centers
                .iter()
                .any(|center| (center - anchor).abs() <= 0.001)
        );
    }
}

#[test]
fn rounded_rect_dashes_contain_owned_corner_points() {
    let dash = Dash::dashed().with_density(1.0);
    let stroke = Stroke {
        width: 6.0,
        dash: Some(dash),
        ..Stroke::default()
    };
    let rect = Rect::new(0.0, 0.0, 120.0, 80.0);
    let radii = Radii::all(24.0);
    let geometry = Shape::rounded_rect(rect, radii)
        .dashed_stroke(stroke)
        .unwrap();

    for corner in [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomRight,
        Corner::BottomLeft,
    ] {
        let expected = ellipse_point(
            corner_center(rect, radii, corner),
            Size::new(corner_radius(radii, corner), corner_radius(radii, corner)),
            corner_anchor_angle(corner),
        );
        assert!(
            geometry.segments().iter().any(|segment| path_visits_point(
                segment.path(),
                expected,
                0.75
            )),
            "missing dash at {corner:?}"
        );
    }
}

fn closed_measure_points(points: &[Point]) -> Vec<Point> {
    let mut closed = points.to_vec();
    if let Some(first) = points.first() {
        closed.push(*first);
    }
    closed
}

fn path_visits_point(path: &Path, expected: Point, tolerance: f64) -> bool {
    let mut current = None;
    for command in path.commands() {
        match *command {
            Command::MoveTo(point) => current = Some(point),
            Command::LineTo(point) => {
                if let Some(start) = current
                    && point_to_segment_distance(expected, start, point) <= tolerance
                {
                    return true;
                }
                current = Some(point);
            }
            _ => {}
        }
    }
    false
}

fn point_to_segment_distance(point: Point, start: Point, end: Point) -> f64 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length_sq = dx * dx + dy * dy;
    if length_sq <= f64::EPSILON {
        return point.distance_to(start);
    }
    let t = (((point.x - start.x) * dx + (point.y - start.y) * dy) / length_sq).clamp(0.0, 1.0);
    point.distance_to(Point::new(start.x + dx * t, start.y + dy * t))
}
