#pragma once

#include <CGAL/Boolean_set_operations_2/oriented_side.h>

#include "polygon.h"

typedef GPSTraits::General_polygon_with_holes_2 PolygonWithHoles;

std::unique_ptr<PolygonWithHoles> create_polygon_with_holes();
std::unique_ptr<PolygonWithHoles> create_polygon_with_holes(const PolygonWithHoles& polygon);

using HoleIterator = Iterator<PolygonWithHoles::Hole_const_iterator>;
std::unique_ptr<HoleIterator> hole_iterator(const PolygonWithHoles &polygon);
