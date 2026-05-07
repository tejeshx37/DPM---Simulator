use cgal::{curve::Curve, BoundaryId, PolygonSet, PolygonWithHoles};
use egui::{Context, Vec2b};
use egui_plot::{Line, Plot, PlotUi};
use std::hash::Hash;

pub fn plot(id_source: impl Hash) -> Plot {
    Plot::new(id_source)
        .data_aspect(1.0)
        .show_axes(Vec2b::FALSE)
        .allow_boxed_zoom(false)
}

pub fn plot_without_clutter(id_source: impl Hash) -> Plot {
    plot(id_source).show_grid(false).show_x(false).show_y(false)
}

pub fn default_transform(_: BoundaryId, ctx: &Context, line: Line) -> Line {
    line.color(super::on_primary_color(ctx))
}

pub struct Projector {
    sx: f32,
    cx: f32,
    sy: f32,
    cy: f32,
}

impl Projector {
    pub fn new(rx: f32, ry: f32) -> Self {
        let (sx, cx) = rx.sin_cos();
        let (sy, cy) = ry.sin_cos();
        Self { sx, cx, sy, cy }
    }

    pub fn project(&self, p: [f32; 3]) -> [f64; 2] {
        let x = p[0] * self.cy + p[2] * self.sy;
        let z = -p[0] * self.sy + p[2] * self.cy;
        let y = p[1] * self.cx - z * self.sx;
        [x as f64, y as f64]
    }
}


pub fn plot_polygon_set<T>(ui: &mut PlotUi, polygon_set: &PolygonSet, transform: T)
where
    T: Fn(BoundaryId, &Context, Line) -> Line + Copy,
{
    polygon_set
        .polygon_with_holes()
        .iter()
        .flat_map(PolygonWithHoles::boundaries_iter)
        .map(|(id, curve)| {
            let line = match curve {
                Curve::Line(line) => Line::new(vec![
                    (line.end_points().start()).into(),
                    (line.end_points().end()).into(),
                ]),
                Curve::Ellipse(arc) => Line::new(arc.polyline().to_vec()),
            };
            (id, line)
        })
        .for_each(|(id, line)| ui.line(transform(id, ui.ctx(), line)))
}

pub fn plot_cached_geometry<T>(ui: &mut PlotUi, geometry: &[(BoundaryId, Vec<[f64; 2]>)], transform: T)
where
    T: Fn(BoundaryId, &Context, Line) -> Line + Copy,
{
    geometry.iter().for_each(|(id, points)| {
        let line = Line::new(points.clone());
        ui.line(transform(*id, ui.ctx(), line));
    });
}
