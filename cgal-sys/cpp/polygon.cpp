#include <CGAL/centroid.h>

#include "polygon.h"

std::unique_ptr<Polygon> create_polygon()
{
    return std::make_unique<Polygon>();
}

std::unique_ptr<Polygon> create_polygon(const Polygon &polygon)
{
    return std::make_unique<Polygon>(polygon);
}

std::unique_ptr<CurveIterator> curve_iterator(const Polygon &polygon)
{
    return std::make_unique<CurveIterator>(polygon.curves_begin(), polygon.curves_end());
}

std::unique_ptr<Point> centroid(const Polygon &polygon)
{
    std::vector<EpicKernel::Point_2> points;
    for (Polygon::Curve_const_iterator it = polygon.curves_begin(); it != polygon.curves_end(); it++)
    {
        if (it->is_special_segment())
        {
            const Point& p = it->target();
            points.emplace_back(p.x().doubleValue(), p.y().doubleValue());
        }
        else
        {
            std::vector<std::pair<double, double>> polyline;
            polyline.reserve(100);
            it->polyline_approximation(100, std::back_inserter(polyline));
            for (auto it = polyline.begin() + 1; it != polyline.end(); it++)
            {
                points.emplace_back(it->first, it->second);
            }
        }
    }
    const EpicKernel::Point_2 centroid = CGAL::centroid(points.begin(), points.end());
    return std::make_unique<Point>(centroid.x(), centroid.y());
}