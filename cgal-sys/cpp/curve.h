#pragma once

#include "point.h"

typedef ConicTraits::Curve_2 ConicCurve;
typedef ConicTraits::X_monotone_curve_2 XMonotoneCurve;
typedef CGAL::Orientation Orientation;

std::unique_ptr<XMonotoneCurve> construct_linear_curve(const Point &source,
                                                       const Point &target);
std::unique_ptr<ConicCurve> construct_conic_curve(const Rational &h,
                                                  const Rational &k,
                                                  const Rational &width,
                                                  const Rational &height);
std::unique_ptr<std::vector<XMonotoneCurve>> split_conic_curve(const ConicCurve &curve);
std::unique_ptr<XMonotoneCurve> clone_x_monotone_curve(const XMonotoneCurve &curve);
void set_orientation(XMonotoneCurve &curve, const Orientation orientation);

using DoublePair = std::pair<double, double>;
std::unique_ptr<std::vector<DoublePair>> polyline_approximation(const XMonotoneCurve &curve,
                                                                const size_t num_points);

bool is_horizontal(const XMonotoneCurve &curve);

bool equals(const XMonotoneCurve &left, const XMonotoneCurve &right);

struct EllipseData
{
public:
    EllipseData(const Point center,
                const Algebraic a,
                const Algebraic b,
                const Algebraic angle_start,
                const Algebraic angle_end) noexcept;
    EllipseData(const EllipseData &data) noexcept;

    const Point &center() const noexcept;
    const Algebraic &a() const noexcept;
    const Algebraic &b() const noexcept;
    const Algebraic &angle_start() const noexcept;
    const Algebraic &angle_end() const noexcept;

private:
    const Point _center;
    const Algebraic _a, _b;
    const Algebraic _angle_start, _angle_end;
};

std::unique_ptr<EllipseData> get_ellipse_data(const XMonotoneCurve &curve);

#define POINT_AT_C(C)                                                \
    std::unique_ptr<Point> point_at_##C(const XMonotoneCurve &curve, \
                                        const Algebraic &C);

POINT_AT_C(x)
POINT_AT_C(y)

#undef POINT_AT_C

std::unique_ptr<std::string> to_string(const XMonotoneCurve &curve);