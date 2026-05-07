#pragma once

#include "traits.h"

typedef ConicTraits::Point_2 Point;
typedef CGAL::Comparison_result ComparisonResult;

std::unique_ptr<Point> create_point(const Algebraic &x, const Algebraic &y);
std::unique_ptr<Point> create_point(const Point &point);
bool points_eq(const Point &first, const Point &second);
