use crate::model::PolygonData;
use cgal::{curve::Curve, BoundaryId, PolygonSet, PolygonWithHoles};
use egui::{Context, Vec2b};
use egui_plot::{Line, Plot, PlotUi};
use std::hash::Hash;

pub fn plot(id_source: impl Hash) -> Plot<'static> {
    Plot::new(id_source)
        .data_aspect(1.0)
        .show_axes(Vec2b::FALSE)
        .allow_boxed_zoom(false)
        .boxed_zoom_pointer_button(egui::PointerButton::Extra1)
        .allow_drag(false)
        .allow_double_click_reset(false)
}

pub fn plot_without_clutter(id_source: impl Hash) -> Plot<'static> {
    plot(id_source)
        .show_grid(false)
        .show_x(false)
        .show_y(false)
        .show_background(false)
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

    pub fn project(&self, p: [f32; 3]) -> [f64; 3] {
        let x = p[0] * self.cy + p[2] * self.sy;
        let z_rot_y = -p[0] * self.sy + p[2] * self.cy;
        let y = p[1] * self.cx - z_rot_y * self.sx;
        let z = p[1] * self.sx + z_rot_y * self.cx;
        [x as f64, y as f64, z as f64]
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

pub fn plot_solid_geometry(
    ui: &mut PlotUi,
    polygon_data: &PolygonData,
    rotation_x: f32,
    rotation_y: f32,
) {
    let projector = Projector::new(rotation_x, rotation_y);
    let geom_3d = polygon_data.plot_geometry_3d();
    
    let projected_vertices: Vec<[f64; 3]> = geom_3d.vertices
        .iter()
        .map(|v| projector.project([v[0] as f32, v[1] as f32, v[2] as f32]))
        .collect();

    // Light direction in world space
    let light_dir = [0.5, 0.5, 0.8]; 
    let base_color = egui::Color32::from_rgb(180, 190, 255);
    
    // Create a list of triangles to sort
    struct TriangleRender {
        p1: [f64; 2],
        p2: [f64; 2],
        p3: [f64; 2],
        z_avg: f64,
        color: egui::Color32,
    }
    
    let mut render_triangles = Vec::with_capacity(geom_3d.triangles.len());

    for tri in &geom_3d.triangles {
        // Safe access to vertices with bounds checking to prevent panics
        let v1 = geom_3d.vertices.get(tri[0]);
        let v2 = geom_3d.vertices.get(tri[1]);
        let v3 = geom_3d.vertices.get(tri[2]);
        
        let (Some(v1), Some(v2), Some(v3)) = (v1, v2, v3) else { continue; };
        
        let e1 = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
        let e2 = [v3[0] - v1[0], v3[1] - v1[1], v3[2] - v1[2]];
        let mut normal = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let len = (normal[0]*normal[0] + normal[1]*normal[1] + normal[2]*normal[2]).sqrt();
        if len > 0.0 {
            normal[0] /= len; normal[1] /= len; normal[2] /= len;
        }

        let p1_3d = projected_vertices.get(tri[0]);
        let p2_3d = projected_vertices.get(tri[1]);
        let p3_3d = projected_vertices.get(tri[2]);
        
        let (Some(p1_3d), Some(p2_3d), Some(p3_3d)) = (p1_3d, p2_3d, p3_3d) else { continue; };
        
        let p1 = [p1_3d[0], p1_3d[1]];
        let p2 = [p2_3d[0], p2_3d[1]];
        let p3 = [p3_3d[0], p3_3d[1]];
        
        let pe1 = [p2[0] - p1[0], p2[1] - p1[1]];
        let pe2 = [p3[0] - p1[0], p3[1] - p1[1]];
        let normal_z = pe1[0] * pe2[1] - pe1[1] * pe2[0];
        if normal_z < 0.0 { continue; } 

        let dot = (normal[0] * light_dir[0] + normal[1] * light_dir[1] + normal[2] * light_dir[2]) as f32;
        let intensity = (0.3f32 + 0.7f32 * dot.max(0.0f32)).clamp(0.0f32, 1.0f32);
        
        let [r, g, b, _] = base_color.to_array();
        let shaded_color = egui::Color32::from_rgb(
            (r as f32 * intensity) as u8,
            (g as f32 * intensity) as u8,
            (b as f32 * intensity) as u8,
        );
        
        let z_avg = (p1_3d[2] + p2_3d[2] + p3_3d[2]) / 3.0;
        
        render_triangles.push(TriangleRender {
            p1, p2, p3, z_avg, color: shaded_color,
        });
    }
    
    // Sort back-to-front (smaller Z is further away in our projection)
    render_triangles.sort_by(|a, b| a.z_avg.partial_cmp(&b.z_avg).unwrap_or(std::cmp::Ordering::Equal));
    
    // Batch render using egui::Mesh for maximum performance
    let transform = ui.transform();
    let mut mesh = egui::Mesh::default();
    
    for tri in render_triangles {
        let p1 = transform.position_from_point(&egui_plot::PlotPoint::new(tri.p1[0], tri.p1[1]));
        let p2 = transform.position_from_point(&egui_plot::PlotPoint::new(tri.p2[0], tri.p2[1]));
        let p3 = transform.position_from_point(&egui_plot::PlotPoint::new(tri.p3[0], tri.p3[1]));
        
        let n = mesh.vertices.len() as u32;
        mesh.vertices.push(egui::epaint::Vertex { pos: p1, uv: egui::Pos2::ZERO, color: tri.color });
        mesh.vertices.push(egui::epaint::Vertex { pos: p2, uv: egui::Pos2::ZERO, color: tri.color });
        mesh.vertices.push(egui::epaint::Vertex { pos: p3, uv: egui::Pos2::ZERO, color: tri.color });
        mesh.indices.extend_from_slice(&[n, n + 1, n + 2]);
    }
    
    if !mesh.is_empty() {
        let painter = ui.ctx().layer_painter(ui.response().layer_id).with_clip_rect(ui.response().rect);
        painter.add(egui::Shape::mesh(mesh));
    }
    
    // Always render 2D shapes as outlines on the plane if they exist
    let color = super::on_primary_color(ui.ctx());
    let geom_2d: &[(BoundaryId, Vec<[f64; 2]>)] = polygon_data.plot_geometry();
    for (_id, points) in geom_2d {
        let projected_points: Vec<[f64; 2]> = points.iter().map(|p: &[f64; 2]| {
            let p3d = projector.project([p[0] as f32, p[1] as f32, 0.0]);
            [p3d[0], p3d[1]]
        }).collect();
        ui.line(egui_plot::Line::new(projected_points).color(color.linear_multiply(0.4)));
    }
}
