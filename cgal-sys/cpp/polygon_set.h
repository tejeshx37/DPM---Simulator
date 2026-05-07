#pragma once

#include <CGAL/Boolean_set_operations_2.h>
#include <CGAL/General_polygon_set_2.h>

#include "polygon_with_holes.h"

typedef CGAL::General_polygon_set_2<GPSTraits> PolygonSet;

std::unique_ptr<PolygonSet> create_polygon_set();
std::unique_ptr<PolygonSet> create_polygon_set(const PolygonSet &polygon_set);
void split_curve(PolygonSet &polygon_set,
                 const XMonotoneCurve &ref_curve,
                 const Point &point);
std::unique_ptr<std::vector<PolygonWithHoles>> polygon_with_holes(const PolygonSet &polygon_set);