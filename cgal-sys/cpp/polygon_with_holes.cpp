#include "polygon_with_holes.h"

std::unique_ptr<PolygonWithHoles> create_polygon_with_holes()
{
    return std::make_unique<PolygonWithHoles>();
}

std::unique_ptr<PolygonWithHoles> create_polygon_with_holes(const PolygonWithHoles &polygon)
{
    return std::make_unique<PolygonWithHoles>(polygon);
}

std::unique_ptr<HoleIterator> hole_iterator(const PolygonWithHoles &polygon)
{
    return std::make_unique<HoleIterator>(polygon.holes_begin(), polygon.holes_end());
}
