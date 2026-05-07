#pragma once

#include "point.h"
#include "curve.h"
#include "iterator.h"

typedef GPSTraits::General_polygon_2 Polygon;

std::unique_ptr<Polygon> create_polygon();
std::unique_ptr<Polygon> create_polygon(const Polygon &polygon);

using CurveIterator = Iterator<Polygon::Curve_const_iterator>;
std::unique_ptr<CurveIterator> curve_iterator(const Polygon &polygon);

std::unique_ptr<Point> centroid(const Polygon &polygon);