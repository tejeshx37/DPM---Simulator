#include "point.h"

std::unique_ptr<Point> create_point(const Algebraic &x, const Algebraic &y)
{
    return std::make_unique<Point>(x, y);
}

std::unique_ptr<Point> create_point(const Point &point)
{
    return std::make_unique<Point>(point);
}

bool points_eq(const Point &first, const Point &second)
{
    const ConicTraits traits;
    return traits.compare_xy_2_object()(first, second) == CGAL::EQUAL;
}
