#include "curve.h"

std::unique_ptr<XMonotoneCurve> construct_linear_curve(const Point &source,
                                                       const Point &target)
{
    const ConicTraits traits;
    const ConicCurve curve = traits.construct_curve_2_object()(source, target);
    std::vector<CGAL::Object> objects;
    traits.make_x_monotone_2_object()(curve, std::back_inserter(objects));
    assert(objects.size() == 1);
    XMonotoneCurve xcv;
    if (CGAL::assign(xcv, objects[0]))
    {
        return std::make_unique<XMonotoneCurve>(xcv);
    }
    std::stringstream stream;
    stream << "Could not generate linear curve from " << source << " to " << target;
    throw std::runtime_error(stream.str());
}

std::unique_ptr<ConicCurve> construct_conic_curve(const Rational &h,
                                                  const Rational &k,
                                                  const Rational &width,
                                                  const Rational &height)
{
    const Rational a = width / 2;
    const Rational b = height / 2;

    const Rational r = b * b;
    const Rational s = a * a;
    const Rational u = -2 * h * r;
    const Rational v = -2 * k * s;
    const Rational w = (r * h * h) + (s * k * k) - (r * s);

    const ConicTraits traits;
    const ConicCurve curve = traits.construct_curve_2_object()(r, s, 0, u, v, w);

    return std::make_unique<ConicCurve>(std::move(curve));
}

std::unique_ptr<std::vector<XMonotoneCurve>> split_conic_curve(const ConicCurve &curve)
{
    std::vector<CGAL::Object> objects;
    const ConicTraits traits;
    traits.make_x_monotone_2_object()(curve, std::back_inserter(objects));
    auto curves = std::make_unique<std::vector<XMonotoneCurve>>();
    curves->reserve(objects.size());
    for (const auto &object : objects)
    {
        XMonotoneCurve xcv;
        if (CGAL::assign(xcv, object))
        {
            curves->push_back(std::move(xcv));
        }
    }
    return curves;
}

std::unique_ptr<XMonotoneCurve> clone_x_monotone_curve(const XMonotoneCurve &curve)
{
    return std::make_unique<XMonotoneCurve>(curve);
}

void set_orientation(XMonotoneCurve &curve, const Orientation orientation)
{
    curve.set_orientation(orientation);
}

std::unique_ptr<std::vector<DoublePair>> polyline_approximation(const XMonotoneCurve &curve,
                                                                const std::size_t num_points)
{
    auto out = std::make_unique<std::vector<DoublePair>>();
    out->reserve(num_points);
    curve.polyline_approximation(num_points, std::back_inserter(*out));
    return out;
}

bool is_horizontal(const XMonotoneCurve &curve)
{
    return curve.is_special_segment() &&
           (curve.target().y() - curve.source().y()).isZero();
}

bool equals(const XMonotoneCurve &left, const XMonotoneCurve &right)
{
    const ConicTraits traits;
    return traits.equal_2_object()(left, right);
}

EllipseData::EllipseData(const Point center,
                         const Algebraic a,
                         const Algebraic b,
                         const Algebraic angle_start,
                         const Algebraic angle_end) noexcept : _center(std::move(center)),
                                                               _a(std::move(a)),
                                                               _b(std::move(b)),
                                                               _angle_start(std::move(angle_start)),
                                                               _angle_end(std::move(angle_end)) {}

EllipseData::EllipseData(const EllipseData &data) noexcept : _center(data._center),
                                                             _a(data._a),
                                                             _b(data._b),
                                                             _angle_start(data._angle_start),
                                                             _angle_end(data._angle_end) {}

const Point &EllipseData::center() const noexcept
{
    return _center;
}

const Algebraic &EllipseData::a() const noexcept
{
    return _a;
}

const Algebraic &EllipseData::b() const noexcept
{
    return _b;
}

const Algebraic &EllipseData::angle_start() const noexcept
{
    return _angle_start;
}

const Algebraic &EllipseData::angle_end() const noexcept
{
    return _angle_end;
}

using Vector = AlgebraicKernel::Vector_3;
using Point3 = AlgebraicKernel::Point_3;

const double RADIAN = 3.14159265358979323846 / 180;

std::unique_ptr<EllipseData> get_ellipse_data(const XMonotoneCurve &curve)
{
    const CORE_ANT an_traits;
    const Algebraic h = -curve.alg_u() / (2 * curve.alg_r());
    const Algebraic k = -curve.alg_v() / (2 * curve.alg_s());
    const Algebraic a_sqr = (curve.alg_s() * k * k + curve.alg_r() * h * h - curve.alg_w()) / curve.alg_r();
    const Algebraic a = an_traits.sqrt(a_sqr);
    const Algebraic b_sqr = curve.alg_r() * a_sqr / curve.alg_s();
    const Algebraic b = an_traits.sqrt(b_sqr);

    const auto angle_with_x = [&](const Point &point)
    {
        return CGAL::approximate_angle(Vector(point.x() - h, point.y() - k, 0),
                                       Vector(Point3(), Point3(1, 0, 0)));
    };

    const Algebraic r_angle = angle_with_x(curve.right());
    const Algebraic l_angle = angle_with_x(curve.left());

    Algebraic angle_start = curve.is_upper() ? r_angle : 360 - l_angle;
    Algebraic angle_end = curve.is_upper() ? l_angle : 360 - r_angle;
    if (curve.orientation() == CGAL::CLOCKWISE)
    {
        std::swap(angle_start, angle_end);
    }

    return std::make_unique<EllipseData>(Point(h, k),
                                         a, b,
                                         angle_start * RADIAN,
                                         angle_end * RADIAN);
}

#define POINT_AT_C(C1, C2, CONSTRUCTOR, A, B)                                                         \
    std::unique_ptr<Point> point_at_##C1(const XMonotoneCurve &curve,                                 \
                                         const Algebraic &C1)                                         \
    {                                                                                                 \
        const ConicTraits traits;                                                                     \
        if (curve.is_special_segment())                                                               \
        {                                                                                             \
            const ConicCurve::Extra_data *const data = curve.extra_data();                            \
            const Algebraic C2 = -(data->c + data->A * C1) / data->B;                                 \
            const Point p = CONSTRUCTOR(C1, C2);                                                      \
            if (traits.contains_point(curve, p))                                                      \
            {                                                                                         \
                return std::make_unique<Point>(std::move(p));                                         \
            }                                                                                         \
        }                                                                                             \
        else                                                                                          \
        {                                                                                             \
            std::array<Algebraic, 2> coords = {0};                                                    \
            const std::uint8_t count = traits.conic_get_##C2##_coordinates(curve, C1, coords.data()); \
            for (std::uint8_t i = 0; i < count; i++)                                                  \
            {                                                                                         \
                const Point p = CONSTRUCTOR(C1, coords.at(i));                                        \
                if (traits.contains_point(curve, p))                                                  \
                {                                                                                     \
                    return std::make_unique<Point>(std::move(p));                                     \
                }                                                                                     \
            }                                                                                         \
        }                                                                                             \
                                                                                                      \
        std::ostringstream msg_stream;                                                                \
        CGAL::IO::set_pretty_mode(msg_stream);                                                        \
        msg_stream << "Could not find point at " #C1 " = " << C1                                      \
                   << " in curve " << curve;                                                          \
        throw std::runtime_error(msg_stream.str());                                                   \
    }

#define POINT_XY_CONSTRUCTOR(X, Y) Point(X, Y);
#define POINT_YX_CONSTRUCTOR(Y, X) Point(X, Y);

POINT_AT_C(x, y, POINT_XY_CONSTRUCTOR, a, b)
POINT_AT_C(y, x, POINT_YX_CONSTRUCTOR, b, a)

#undef POINT_XY_CONSTRUCTOR
#undef POINT_YX_CONSTRUCTOR
#undef POINT_AT_C

std::unique_ptr<std::string> to_string(const XMonotoneCurve &curve)
{
    std::ostringstream stream;
    stream << curve;
    return std::make_unique<std::string>(stream.str());
}