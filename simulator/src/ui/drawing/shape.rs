use enum_map::Enum;
use strum::{Display, EnumIter};

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter, Display, Enum)]
pub enum Shape {
    Rectangle,
    Polygon,
    Circle,
    Ellipse,
    Cube,
    Cuboid,
    Cone,
    Sphere,
}
