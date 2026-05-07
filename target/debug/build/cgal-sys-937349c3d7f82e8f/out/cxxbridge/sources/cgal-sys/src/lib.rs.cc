#include "cgal-sys/cpp/curve.h"
#include "cgal-sys/cpp/kernel.h"
#include "cgal-sys/cpp/num.h"
#include "cgal-sys/cpp/pair_utils.h"
#include "cgal-sys/cpp/point.h"
#include "cgal-sys/cpp/polygon_set.h"
#include "cgal-sys/cpp/polygon_with_holes.h"
#include "cgal-sys/cpp/polygon.h"
#include "cgal-sys/cpp/vector_utils.h"
#include <cstddef>
#include <cstdint>
#include <exception>
#include <memory>
#include <new>
#include <string>
#include <type_traits>
#include <utility>
#include <vector>

namespace rust {
inline namespace cxxbridge1 {
// #include "rust/cxx.h"

#ifndef CXXBRIDGE1_IS_COMPLETE
#define CXXBRIDGE1_IS_COMPLETE
namespace detail {
namespace {
template <typename T, typename = std::size_t>
struct is_complete : std::false_type {};
template <typename T>
struct is_complete<T, decltype(sizeof(T))> : std::true_type {};
} // namespace
} // namespace detail
#endif // CXXBRIDGE1_IS_COMPLETE

namespace repr {
struct PtrLen final {
  void *ptr;
  ::std::size_t len;
};
} // namespace repr

namespace detail {
class Fail final {
  ::rust::repr::PtrLen &throw$;
public:
  Fail(::rust::repr::PtrLen &throw$) noexcept : throw$(throw$) {}
  void operator()(char const *) noexcept;
  void operator()(std::string const &) noexcept;
};
} // namespace detail

namespace {
template <typename T>
void destroy(T *ptr) {
  ptr->~T();
}

template <bool> struct deleter_if {
  template <typename T> void operator()(T *) {}
};

template <> struct deleter_if<true> {
  template <typename T> void operator()(T *ptr) { ptr->~T(); }
};
} // namespace
} // namespace cxxbridge1

namespace behavior {
class missing {};
missing trycatch(...);

template <typename Try, typename Fail>
static typename ::std::enable_if<
    ::std::is_same<decltype(trycatch(::std::declval<Try>(), ::std::declval<Fail>())),
                 missing>::value>::type
trycatch(Try &&func, Fail &&fail) noexcept try {
  func();
} catch (::std::exception const &e) {
  fail(e.what());
}
} // namespace behavior
} // namespace rust

using Algebraic = ::Algebraic;
using Rational = ::Rational;
using Integer = ::Integer;
using Point = ::Point;
using ComparisonResult = ::ComparisonResult;
using Orientation = ::Orientation;
using DoublePair = ::DoublePair;
using ConicCurve = ::ConicCurve;
using XMonotoneCurve = ::XMonotoneCurve;
using EllipseData = ::EllipseData;
using Polygon = ::Polygon;
using CurveIterator = ::CurveIterator;
using PolygonWithHoles = ::PolygonWithHoles;
using HoleIterator = ::HoleIterator;
using PolygonSet = ::PolygonSet;

static_assert(::std::is_enum<Orientation>::value, "expected enum");
static_assert(sizeof(Orientation) == sizeof(::std::int32_t), "incorrect size");
static_assert(static_cast<::std::int32_t>(Orientation::CLOCKWISE) == -1, "disagrees with the value in #[cxx::bridge]");
static_assert(static_cast<::std::int32_t>(Orientation::COLLINEAR) == 0, "disagrees with the value in #[cxx::bridge]");
static_assert(static_cast<::std::int32_t>(Orientation::COUNTERCLOCKWISE) == 1, "disagrees with the value in #[cxx::bridge]");

static_assert(::std::is_enum<ComparisonResult>::value, "expected enum");
static_assert(sizeof(ComparisonResult) == sizeof(::std::int32_t), "incorrect size");
static_assert(static_cast<::std::int32_t>(ComparisonResult::SMALLER) == -1, "disagrees with the value in #[cxx::bridge]");
static_assert(static_cast<::std::int32_t>(ComparisonResult::EQUAL) == 0, "disagrees with the value in #[cxx::bridge]");
static_assert(static_cast<::std::int32_t>(ComparisonResult::LARGER) == 1, "disagrees with the value in #[cxx::bridge]");

extern "C" {
::Algebraic *cxxbridge1$create_algebraic_from_i32(::std::int32_t value) noexcept {
  ::std::unique_ptr<::Algebraic> (*create_algebraic_from_i32$)(::std::int32_t) = ::create_algebraic;
  return create_algebraic_from_i32$(value).release();
}

::Algebraic *cxxbridge1$create_algebraic_from_u32(::std::uint32_t value) noexcept {
  ::std::unique_ptr<::Algebraic> (*create_algebraic_from_u32$)(::std::uint32_t) = ::create_algebraic;
  return create_algebraic_from_u32$(value).release();
}

::Algebraic *cxxbridge1$create_algebraic_from_f64(double value) noexcept {
  ::std::unique_ptr<::Algebraic> (*create_algebraic_from_f64$)(double) = ::create_algebraic;
  return create_algebraic_from_f64$(value).release();
}

::Algebraic *cxxbridge1$create_algebraic_from_rational(::Rational const &value) noexcept {
  ::std::unique_ptr<::Algebraic> (*create_algebraic_from_rational$)(::Rational const &) = ::create_algebraic;
  return create_algebraic_from_rational$(value).release();
}

::Algebraic *cxxbridge1$create_algebraic_from_integer(::Integer const &value) noexcept {
  ::std::unique_ptr<::Algebraic> (*create_algebraic_from_integer$)(::Integer const &) = ::create_algebraic;
  return create_algebraic_from_integer$(value).release();
}

::Algebraic *cxxbridge1$clone_algebraic(::Algebraic const &value) noexcept {
  ::std::unique_ptr<::Algebraic> (*clone_algebraic$)(::Algebraic const &) = ::create_algebraic;
  return clone_algebraic$(value).release();
}

::Algebraic *cxxbridge1$abs_algebraic(::Algebraic const &value) noexcept {
  ::std::unique_ptr<::Algebraic> (*abs_algebraic$)(::Algebraic const &) = ::abs;
  return abs_algebraic$(value).release();
}

double cxxbridge1$Algebraic$double_value(::Algebraic const &self) noexcept {
  double (::Algebraic::*double_value$)() const = &::Algebraic::doubleValue;
  return (self.*double_value$)();
}

::std::string *cxxbridge1$algebraic_to_string(::Algebraic const &value) noexcept {
  ::std::unique_ptr<::std::string> (*algebraic_to_string$)(::Algebraic const &) = ::to_string;
  return algebraic_to_string$(value).release();
}

::rust::repr::PtrLen cxxbridge1$algebraic_from_string(::std::string const &str, ::Algebraic **return$) noexcept {
  ::std::unique_ptr<::Algebraic> (*algebraic_from_string$)(::std::string const &) = ::from_string;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Algebraic *(algebraic_from_string$(str).release());
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::Algebraic *cxxbridge1$add_algebraic(::Algebraic const &lhs, ::Algebraic const &rhs) noexcept {
  ::std::unique_ptr<::Algebraic> (*add_algebraic$)(::Algebraic const &, ::Algebraic const &) = ::add;
  return add_algebraic$(lhs, rhs).release();
}

::Algebraic *cxxbridge1$sub_algebraic(::Algebraic const &lhs, ::Algebraic const &rhs) noexcept {
  ::std::unique_ptr<::Algebraic> (*sub_algebraic$)(::Algebraic const &, ::Algebraic const &) = ::sub;
  return sub_algebraic$(lhs, rhs).release();
}

::Algebraic *cxxbridge1$mul_algebraic(::Algebraic const &lhs, ::Algebraic const &rhs) noexcept {
  ::std::unique_ptr<::Algebraic> (*mul_algebraic$)(::Algebraic const &, ::Algebraic const &) = ::mul;
  return mul_algebraic$(lhs, rhs).release();
}

::Algebraic *cxxbridge1$div_algebraic(::Algebraic const &lhs, ::Algebraic const &rhs) noexcept {
  ::std::unique_ptr<::Algebraic> (*div_algebraic$)(::Algebraic const &, ::Algebraic const &) = ::div;
  return div_algebraic$(lhs, rhs).release();
}

::Algebraic *cxxbridge1$neg_algebraic(::Algebraic const &value) noexcept {
  ::std::unique_ptr<::Algebraic> (*neg_algebraic$)(::Algebraic const &) = ::neg;
  return neg_algebraic$(value).release();
}

::Rational *cxxbridge1$create_rational_from_f64(double value) noexcept {
  ::std::unique_ptr<::Rational> (*create_rational_from_f64$)(double) = ::create_rational;
  return create_rational_from_f64$(value).release();
}

::Rational *cxxbridge1$create_rational_from_i32(::std::int32_t num, ::std::int32_t den) noexcept {
  ::std::unique_ptr<::Rational> (*create_rational_from_i32$)(::std::int32_t, ::std::int32_t) = ::create_rational;
  return create_rational_from_i32$(num, den).release();
}

::Rational *cxxbridge1$create_rational_from_integer(::Integer const &num, ::Integer const &den) noexcept {
  ::std::unique_ptr<::Rational> (*create_rational_from_integer$)(::Integer const &, ::Integer const &) = ::create_rational;
  return create_rational_from_integer$(num, den).release();
}

::Rational *cxxbridge1$clone_rational(::Rational const &value) noexcept {
  ::std::unique_ptr<::Rational> (*clone_rational$)(::Rational const &) = ::create_rational;
  return clone_rational$(value).release();
}

::rust::repr::PtrLen cxxbridge1$rational_from_string(::std::string const &str, ::Rational **return$) noexcept {
  ::std::unique_ptr<::Rational> (*rational_from_string$)(::std::string const &) = ::from_string;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Rational *(rational_from_string$(str).release());
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::std::string *cxxbridge1$rational_to_string(::Rational const &value) noexcept {
  ::std::unique_ptr<::std::string> (*rational_to_string$)(::Rational const &) = ::to_string;
  return rational_to_string$(value).release();
}

bool cxxbridge1$rational_eq(::Rational const &a, ::Rational const &b) noexcept {
  bool (*rational_eq$)(::Rational const &, ::Rational const &) = ::equals;
  return rational_eq$(a, b);
}

::Integer *cxxbridge1$create_integer_from_i32(::std::int32_t value) noexcept {
  ::std::unique_ptr<::Integer> (*create_integer_from_i32$)(::std::int32_t) = ::create_integer;
  return create_integer_from_i32$(value).release();
}

::Integer *cxxbridge1$create_integer_from_u32(::std::uint32_t value) noexcept {
  ::std::unique_ptr<::Integer> (*create_integer_from_u32$)(::std::uint32_t) = ::create_integer;
  return create_integer_from_u32$(value).release();
}

::Integer *cxxbridge1$clone_integer(::Integer const &value) noexcept {
  ::std::unique_ptr<::Integer> (*clone_integer$)(::Integer const &) = ::create_integer;
  return clone_integer$(value).release();
}

::rust::repr::PtrLen cxxbridge1$integer_from_string(::std::string const &str, ::Integer **return$) noexcept {
  ::std::unique_ptr<::Integer> (*integer_from_string$)(::std::string const &) = ::from_string;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Integer *(integer_from_string$(str).release());
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::std::string *cxxbridge1$integer_to_string(::Integer const &value) noexcept {
  ::std::unique_ptr<::std::string> (*integer_to_string$)(::Integer const &) = ::to_string;
  return integer_to_string$(value).release();
}

bool cxxbridge1$integer_eq(::Integer const &a, ::Integer const &b) noexcept {
  bool (*integer_eq$)(::Integer const &, ::Integer const &) = ::equals;
  return integer_eq$(a, b);
}
} // extern "C"

namespace CGAL {
extern "C" {
bool CGAL$cxxbridge1$is_zero(::Integer const &value) noexcept {
  bool (*is_zero$)(::Integer const &) = ::CGAL::is_zero;
  return is_zero$(value);
}

bool CGAL$cxxbridge1$is_negative(::Integer const &value) noexcept {
  bool (*is_negative$)(::Integer const &) = ::CGAL::is_negative;
  return is_negative$(value);
}
} // extern "C"
} // namespace CGAL

extern "C" {
::Integer *cxxbridge1$pow_integer(::Integer const &base, ::std::uint32_t exp) noexcept {
  ::std::unique_ptr<::Integer> (*pow_integer$)(::Integer const &, ::std::uint32_t) = ::pow_integer;
  return pow_integer$(base, exp).release();
}

::Integer *cxxbridge1$abs_integer(::Integer const &value) noexcept {
  ::std::unique_ptr<::Integer> (*abs_integer$)(::Integer const &) = ::abs;
  return abs_integer$(value).release();
}

::Integer *cxxbridge1$mul_integer(::Integer const &lhs, ::Integer const &rhs) noexcept {
  ::std::unique_ptr<::Integer> (*mul_integer$)(::Integer const &, ::Integer const &) = ::mul;
  return mul_integer$(lhs, rhs).release();
}

::Point *cxxbridge1$create_point(::Algebraic const &x, ::Algebraic const &y) noexcept {
  ::std::unique_ptr<::Point> (*create_point$)(::Algebraic const &, ::Algebraic const &) = ::create_point;
  return create_point$(x, y).release();
}

::Point *cxxbridge1$clone_point(::Point const &point) noexcept {
  ::std::unique_ptr<::Point> (*clone_point$)(::Point const &) = ::create_point;
  return clone_point$(point).release();
}

::Algebraic const *cxxbridge1$Point$x(::Point const &self) noexcept {
  ::Algebraic const &(::Point::*x$)() const = &::Point::x;
  return &(self.*x$)();
}

::Algebraic const *cxxbridge1$Point$y(::Point const &self) noexcept {
  ::Algebraic const &(::Point::*y$)() const = &::Point::y;
  return &(self.*y$)();
}

bool cxxbridge1$points_eq(::Point const &first, ::Point const &second) noexcept {
  bool (*points_eq$)(::Point const &, ::Point const &) = ::points_eq;
  return points_eq$(first, second);
}
} // extern "C"

namespace CGAL {
extern "C" {
::ComparisonResult CGAL$cxxbridge1$compare_algebraic(::Algebraic const &a, ::Algebraic const &b) noexcept {
  ::ComparisonResult (*compare_algebraic$)(::Algebraic const &, ::Algebraic const &) = ::CGAL::compare;
  return compare_algebraic$(a, b);
}
} // extern "C"
} // namespace CGAL

extern "C" {
double cxxbridge1$get_x(::DoublePair const &pair) noexcept {
  double (*get_x$)(::DoublePair const &) = ::first;
  return get_x$(pair);
}

double cxxbridge1$get_y(::DoublePair const &pair) noexcept {
  double (*get_y$)(::DoublePair const &) = ::second;
  return get_y$(pair);
}

::rust::repr::PtrLen cxxbridge1$construct_conic_curve(::Rational const &h, ::Rational const &k, ::Rational const &width, ::Rational const &height, ::ConicCurve **return$) noexcept {
  ::std::unique_ptr<::ConicCurve> (*construct_conic_curve$)(::Rational const &, ::Rational const &, ::Rational const &, ::Rational const &) = ::construct_conic_curve;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::ConicCurve *(construct_conic_curve$(h, k, width, height).release());
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

void cxxbridge1$ConicCurve$set_endpoints(::ConicCurve &self, ::Point const &source, ::Point const &target) noexcept {
  void (::ConicCurve::*set_endpoints$)(::Point const &, ::Point const &) = &::ConicCurve::set_endpoints;
  (self.*set_endpoints$)(source, target);
}

void cxxbridge1$ConicCurve$set_orientation(::ConicCurve &self, ::Orientation orientation) noexcept {
  void (::ConicCurve::*set_orientation$)(::Orientation) = &::ConicCurve::set_orientation;
  (self.*set_orientation$)(orientation);
}

::Point const *cxxbridge1$XMonotoneCurve$source(::XMonotoneCurve const &self) noexcept {
  ::Point const &(::XMonotoneCurve::*source$)() const = &::XMonotoneCurve::source;
  return &(self.*source$)();
}

::Point const *cxxbridge1$XMonotoneCurve$target(::XMonotoneCurve const &self) noexcept {
  ::Point const &(::XMonotoneCurve::*target$)() const = &::XMonotoneCurve::target;
  return &(self.*target$)();
}

bool cxxbridge1$XMonotoneCurve$is_upper(::XMonotoneCurve const &self) noexcept {
  bool (::XMonotoneCurve::*is_upper$)() const = &::XMonotoneCurve::is_upper;
  return (self.*is_upper$)();
}

::std::vector<::DoublePair> *cxxbridge1$polyline_approximation(::XMonotoneCurve const &curve, ::std::size_t num_points) noexcept {
  ::std::unique_ptr<::std::vector<::DoublePair>> (*polyline_approximation$)(::XMonotoneCurve const &, ::std::size_t) = ::polyline_approximation;
  return polyline_approximation$(curve, num_points).release();
}

::Orientation cxxbridge1$XMonotoneCurve$orientation(::XMonotoneCurve const &self) noexcept {
  ::Orientation (::XMonotoneCurve::*orientation$)() const = &::XMonotoneCurve::orientation;
  return (self.*orientation$)();
}

bool cxxbridge1$XMonotoneCurve$is_special_segment(::XMonotoneCurve const &self) noexcept {
  bool (::XMonotoneCurve::*is_special_segment$)() const = &::XMonotoneCurve::is_special_segment;
  return (self.*is_special_segment$)();
}

bool cxxbridge1$is_horizontal(::XMonotoneCurve const &curve) noexcept {
  bool (*is_horizontal$)(::XMonotoneCurve const &) = ::is_horizontal;
  return is_horizontal$(curve);
}

bool cxxbridge1$XMonotoneCurve$is_vertical(::XMonotoneCurve const &self) noexcept {
  bool (::XMonotoneCurve::*is_vertical$)() const = &::XMonotoneCurve::is_vertical;
  return (self.*is_vertical$)();
}

bool cxxbridge1$equals(::XMonotoneCurve const &lhs, ::XMonotoneCurve const &rhs) noexcept {
  bool (*equals$)(::XMonotoneCurve const &, ::XMonotoneCurve const &) = ::equals;
  return equals$(lhs, rhs);
}

::rust::repr::PtrLen cxxbridge1$construct_linear_curve(::Point const &source, ::Point const &target, ::XMonotoneCurve **return$) noexcept {
  ::std::unique_ptr<::XMonotoneCurve> (*construct_linear_curve$)(::Point const &, ::Point const &) = ::construct_linear_curve;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::XMonotoneCurve *(construct_linear_curve$(source, target).release());
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$split_conic_curve(::ConicCurve const &curve, ::std::vector<::XMonotoneCurve> **return$) noexcept {
  ::std::unique_ptr<::std::vector<::XMonotoneCurve>> (*split_conic_curve$)(::ConicCurve const &) = ::split_conic_curve;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::std::vector<::XMonotoneCurve> *(split_conic_curve$(curve).release());
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::XMonotoneCurve *cxxbridge1$clone_x_monotone_curve(::XMonotoneCurve const &curve) noexcept {
  ::std::unique_ptr<::XMonotoneCurve> (*clone_x_monotone_curve$)(::XMonotoneCurve const &) = ::clone_x_monotone_curve;
  return clone_x_monotone_curve$(curve).release();
}

::std::string *cxxbridge1$curve_to_string(::XMonotoneCurve const &curve) noexcept {
  ::std::unique_ptr<::std::string> (*curve_to_string$)(::XMonotoneCurve const &) = ::to_string;
  return curve_to_string$(curve).release();
}

::EllipseData *cxxbridge1$get_ellipse_data(::XMonotoneCurve const &curve) noexcept {
  ::std::unique_ptr<::EllipseData> (*get_ellipse_data$)(::XMonotoneCurve const &) = ::get_ellipse_data;
  return get_ellipse_data$(curve).release();
}

::Point const *cxxbridge1$EllipseData$center(::EllipseData const &self) noexcept {
  ::Point const &(::EllipseData::*center$)() const = &::EllipseData::center;
  return &(self.*center$)();
}

::Algebraic const *cxxbridge1$EllipseData$a(::EllipseData const &self) noexcept {
  ::Algebraic const &(::EllipseData::*a$)() const = &::EllipseData::a;
  return &(self.*a$)();
}

::Algebraic const *cxxbridge1$EllipseData$b(::EllipseData const &self) noexcept {
  ::Algebraic const &(::EllipseData::*b$)() const = &::EllipseData::b;
  return &(self.*b$)();
}

::Algebraic const *cxxbridge1$EllipseData$angle_start(::EllipseData const &self) noexcept {
  ::Algebraic const &(::EllipseData::*angle_start$)() const = &::EllipseData::angle_start;
  return &(self.*angle_start$)();
}

::Algebraic const *cxxbridge1$EllipseData$angle_end(::EllipseData const &self) noexcept {
  ::Algebraic const &(::EllipseData::*angle_end$)() const = &::EllipseData::angle_end;
  return &(self.*angle_end$)();
}

::rust::repr::PtrLen cxxbridge1$point_at_x(::XMonotoneCurve const &curve, ::Algebraic const &x, ::Point **return$) noexcept {
  ::std::unique_ptr<::Point> (*point_at_x$)(::XMonotoneCurve const &, ::Algebraic const &) = ::point_at_x;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Point *(point_at_x$(curve, x).release());
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$point_at_y(::XMonotoneCurve const &curve, ::Algebraic const &y, ::Point **return$) noexcept {
  ::std::unique_ptr<::Point> (*point_at_y$)(::XMonotoneCurve const &, ::Algebraic const &) = ::point_at_y;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Point *(point_at_y$(curve, y).release());
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::Polygon *cxxbridge1$create_polygon() noexcept {
  ::std::unique_ptr<::Polygon> (*create_polygon$)() = ::create_polygon;
  return create_polygon$().release();
}

::Polygon *cxxbridge1$clone_polygon(::Polygon const &polygon) noexcept {
  ::std::unique_ptr<::Polygon> (*clone_polygon$)(::Polygon const &) = ::create_polygon;
  return clone_polygon$(polygon).release();
}

::rust::repr::PtrLen cxxbridge1$Polygon$push_back(::Polygon &self, ::XMonotoneCurve const &curve) noexcept {
  void (::Polygon::*push_back$)(::XMonotoneCurve const &) = &::Polygon::push_back;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*push_back$)(curve);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::Orientation cxxbridge1$Polygon$orientation(::Polygon const &self) noexcept {
  ::Orientation (::Polygon::*orientation$)() const = &::Polygon::orientation;
  return (self.*orientation$)();
}

void cxxbridge1$Polygon$reverse_orientation(::Polygon &self) noexcept {
  void (::Polygon::*reverse_orientation$)() = &::Polygon::reverse_orientation;
  (self.*reverse_orientation$)();
}

::std::uint32_t cxxbridge1$Polygon$size(::Polygon const &self) noexcept {
  ::std::uint32_t (::Polygon::*size$)() const = &::Polygon::size;
  return (self.*size$)();
}

::Point *cxxbridge1$centroid(::Polygon const &polygon) noexcept {
  ::std::unique_ptr<::Point> (*centroid$)(::Polygon const &) = ::centroid;
  return centroid$(polygon).release();
}

::CurveIterator *cxxbridge1$curve_iterator(::Polygon const &polygon) noexcept {
  ::std::unique_ptr<::CurveIterator> (*curve_iterator$)(::Polygon const &) = ::curve_iterator;
  return curve_iterator$(polygon).release();
}

bool cxxbridge1$CurveIterator$has_next(::CurveIterator const &self) noexcept {
  bool (::CurveIterator::*has_next$)() const = &::CurveIterator::has_next;
  return (self.*has_next$)();
}

::XMonotoneCurve const *cxxbridge1$CurveIterator$next(::CurveIterator &self) noexcept {
  ::XMonotoneCurve const &(::CurveIterator::*next$)() = &::CurveIterator::next;
  return &(self.*next$)();
}

::PolygonWithHoles *cxxbridge1$create_polygon_with_holes() noexcept {
  ::std::unique_ptr<::PolygonWithHoles> (*create_polygon_with_holes$)() = ::create_polygon_with_holes;
  return create_polygon_with_holes$().release();
}

::PolygonWithHoles *cxxbridge1$clone_polygon_with_holes(::PolygonWithHoles const &polygon) noexcept {
  ::std::unique_ptr<::PolygonWithHoles> (*clone_polygon_with_holes$)(::PolygonWithHoles const &) = ::create_polygon_with_holes;
  return clone_polygon_with_holes$(polygon).release();
}

::Polygon const *cxxbridge1$PolygonWithHoles$outer_boundary(::PolygonWithHoles const &self) noexcept {
  ::Polygon const &(::PolygonWithHoles::*outer_boundary$)() const = &::PolygonWithHoles::outer_boundary;
  return &(self.*outer_boundary$)();
}

::std::uint32_t cxxbridge1$PolygonWithHoles$number_of_holes(::PolygonWithHoles const &self) noexcept {
  ::std::uint32_t (::PolygonWithHoles::*number_of_holes$)() const = &::PolygonWithHoles::number_of_holes;
  return (self.*number_of_holes$)();
}

::HoleIterator *cxxbridge1$hole_iterator(::PolygonWithHoles const &polygon) noexcept {
  ::std::unique_ptr<::HoleIterator> (*hole_iterator$)(::PolygonWithHoles const &) = ::hole_iterator;
  return hole_iterator$(polygon).release();
}

bool cxxbridge1$HoleIterator$has_next(::HoleIterator const &self) noexcept {
  bool (::HoleIterator::*has_next$)() const = &::HoleIterator::has_next;
  return (self.*has_next$)();
}

::Polygon const *cxxbridge1$HoleIterator$next(::HoleIterator &self) noexcept {
  ::Polygon const &(::HoleIterator::*next$)() = &::HoleIterator::next;
  return &(self.*next$)();
}

::PolygonSet *cxxbridge1$create_polygon_set() noexcept {
  ::std::unique_ptr<::PolygonSet> (*create_polygon_set$)() = ::create_polygon_set;
  return create_polygon_set$().release();
}

::PolygonSet *cxxbridge1$clone_polygon_set(::PolygonSet const &polygon_set) noexcept {
  ::std::unique_ptr<::PolygonSet> (*clone_polygon_set$)(::PolygonSet const &) = ::create_polygon_set;
  return clone_polygon_set$(polygon_set).release();
}

::rust::repr::PtrLen cxxbridge1$PolygonSet$insert(::PolygonSet &self, ::Polygon const &polygon) noexcept {
  void (::PolygonSet::*insert$)(::Polygon const &) = &::PolygonSet::insert;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*insert$)(polygon);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$PolygonSet$join(::PolygonSet &self, ::Polygon const &polygon) noexcept {
  void (::PolygonSet::*join$)(::Polygon const &) = &::PolygonSet::join;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*join$)(polygon);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$PolygonSet$difference(::PolygonSet &self, ::Polygon const &polygon) noexcept {
  void (::PolygonSet::*difference$)(::Polygon const &) = &::PolygonSet::difference;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*difference$)(polygon);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$split_curve(::PolygonSet &polygon_set, ::XMonotoneCurve const &ref_curve, ::Point const &point) noexcept {
  void (*split_curve$)(::PolygonSet &, ::XMonotoneCurve const &, ::Point const &) = ::split_curve;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        split_curve$(polygon_set, ref_curve, point);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

bool cxxbridge1$PolygonSet$is_empty(::PolygonSet const &self) noexcept {
  bool (::PolygonSet::*is_empty$)() const = &::PolygonSet::is_empty;
  return (self.*is_empty$)();
}

::std::vector<::PolygonWithHoles> *cxxbridge1$polygon_with_holes(::PolygonSet const &polygon_set) noexcept {
  ::std::unique_ptr<::std::vector<::PolygonWithHoles>> (*polygon_with_holes$)(::PolygonSet const &) = ::polygon_with_holes;
  return polygon_with_holes$(polygon_set).release();
}

void cxxbridge1$PolygonSet$clear(::PolygonSet &self) noexcept {
  void (::PolygonSet::*clear$)() = &::PolygonSet::clear;
  (self.*clear$)();
}

static_assert(::rust::detail::is_complete<::Algebraic>::value, "definition of Algebraic is required");
static_assert(sizeof(::std::unique_ptr<::Algebraic>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::Algebraic>) == alignof(void *), "");
void cxxbridge1$unique_ptr$Algebraic$null(::std::unique_ptr<::Algebraic> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::Algebraic>();
}
void cxxbridge1$unique_ptr$Algebraic$raw(::std::unique_ptr<::Algebraic> *ptr, ::Algebraic *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::Algebraic>(raw);
}
::Algebraic const *cxxbridge1$unique_ptr$Algebraic$get(::std::unique_ptr<::Algebraic> const &ptr) noexcept {
  return ptr.get();
}
::Algebraic *cxxbridge1$unique_ptr$Algebraic$release(::std::unique_ptr<::Algebraic> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$Algebraic$drop(::std::unique_ptr<::Algebraic> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::Algebraic>::value>{}(ptr);
}

static_assert(::rust::detail::is_complete<::Rational>::value, "definition of Rational is required");
static_assert(sizeof(::std::unique_ptr<::Rational>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::Rational>) == alignof(void *), "");
void cxxbridge1$unique_ptr$Rational$null(::std::unique_ptr<::Rational> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::Rational>();
}
void cxxbridge1$unique_ptr$Rational$raw(::std::unique_ptr<::Rational> *ptr, ::Rational *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::Rational>(raw);
}
::Rational const *cxxbridge1$unique_ptr$Rational$get(::std::unique_ptr<::Rational> const &ptr) noexcept {
  return ptr.get();
}
::Rational *cxxbridge1$unique_ptr$Rational$release(::std::unique_ptr<::Rational> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$Rational$drop(::std::unique_ptr<::Rational> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::Rational>::value>{}(ptr);
}

static_assert(::rust::detail::is_complete<::Integer>::value, "definition of Integer is required");
static_assert(sizeof(::std::unique_ptr<::Integer>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::Integer>) == alignof(void *), "");
void cxxbridge1$unique_ptr$Integer$null(::std::unique_ptr<::Integer> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::Integer>();
}
void cxxbridge1$unique_ptr$Integer$raw(::std::unique_ptr<::Integer> *ptr, ::Integer *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::Integer>(raw);
}
::Integer const *cxxbridge1$unique_ptr$Integer$get(::std::unique_ptr<::Integer> const &ptr) noexcept {
  return ptr.get();
}
::Integer *cxxbridge1$unique_ptr$Integer$release(::std::unique_ptr<::Integer> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$Integer$drop(::std::unique_ptr<::Integer> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::Integer>::value>{}(ptr);
}

static_assert(::rust::detail::is_complete<::Point>::value, "definition of Point is required");
static_assert(sizeof(::std::unique_ptr<::Point>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::Point>) == alignof(void *), "");
void cxxbridge1$unique_ptr$Point$null(::std::unique_ptr<::Point> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::Point>();
}
void cxxbridge1$unique_ptr$Point$raw(::std::unique_ptr<::Point> *ptr, ::Point *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::Point>(raw);
}
::Point const *cxxbridge1$unique_ptr$Point$get(::std::unique_ptr<::Point> const &ptr) noexcept {
  return ptr.get();
}
::Point *cxxbridge1$unique_ptr$Point$release(::std::unique_ptr<::Point> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$Point$drop(::std::unique_ptr<::Point> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::Point>::value>{}(ptr);
}

static_assert(::rust::detail::is_complete<::ConicCurve>::value, "definition of ConicCurve is required");
static_assert(sizeof(::std::unique_ptr<::ConicCurve>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::ConicCurve>) == alignof(void *), "");
void cxxbridge1$unique_ptr$ConicCurve$null(::std::unique_ptr<::ConicCurve> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::ConicCurve>();
}
void cxxbridge1$unique_ptr$ConicCurve$raw(::std::unique_ptr<::ConicCurve> *ptr, ::ConicCurve *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::ConicCurve>(raw);
}
::ConicCurve const *cxxbridge1$unique_ptr$ConicCurve$get(::std::unique_ptr<::ConicCurve> const &ptr) noexcept {
  return ptr.get();
}
::ConicCurve *cxxbridge1$unique_ptr$ConicCurve$release(::std::unique_ptr<::ConicCurve> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$ConicCurve$drop(::std::unique_ptr<::ConicCurve> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::ConicCurve>::value>{}(ptr);
}

::std::vector<::DoublePair> *cxxbridge1$std$vector$DoublePair$new() noexcept {
  return new ::std::vector<::DoublePair>();
}
::std::size_t cxxbridge1$std$vector$DoublePair$size(::std::vector<::DoublePair> const &s) noexcept {
  return s.size();
}
::DoublePair *cxxbridge1$std$vector$DoublePair$get_unchecked(::std::vector<::DoublePair> *s, ::std::size_t pos) noexcept {
  return &(*s)[pos];
}
static_assert(sizeof(::std::unique_ptr<::std::vector<::DoublePair>>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::std::vector<::DoublePair>>) == alignof(void *), "");
void cxxbridge1$unique_ptr$std$vector$DoublePair$null(::std::unique_ptr<::std::vector<::DoublePair>> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::DoublePair>>();
}
void cxxbridge1$unique_ptr$std$vector$DoublePair$raw(::std::unique_ptr<::std::vector<::DoublePair>> *ptr, ::std::vector<::DoublePair> *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::DoublePair>>(raw);
}
::std::vector<::DoublePair> const *cxxbridge1$unique_ptr$std$vector$DoublePair$get(::std::unique_ptr<::std::vector<::DoublePair>> const &ptr) noexcept {
  return ptr.get();
}
::std::vector<::DoublePair> *cxxbridge1$unique_ptr$std$vector$DoublePair$release(::std::unique_ptr<::std::vector<::DoublePair>> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$std$vector$DoublePair$drop(::std::unique_ptr<::std::vector<::DoublePair>> *ptr) noexcept {
  ptr->~unique_ptr();
}

static_assert(::rust::detail::is_complete<::XMonotoneCurve>::value, "definition of XMonotoneCurve is required");
static_assert(sizeof(::std::unique_ptr<::XMonotoneCurve>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::XMonotoneCurve>) == alignof(void *), "");
void cxxbridge1$unique_ptr$XMonotoneCurve$null(::std::unique_ptr<::XMonotoneCurve> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::XMonotoneCurve>();
}
void cxxbridge1$unique_ptr$XMonotoneCurve$raw(::std::unique_ptr<::XMonotoneCurve> *ptr, ::XMonotoneCurve *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::XMonotoneCurve>(raw);
}
::XMonotoneCurve const *cxxbridge1$unique_ptr$XMonotoneCurve$get(::std::unique_ptr<::XMonotoneCurve> const &ptr) noexcept {
  return ptr.get();
}
::XMonotoneCurve *cxxbridge1$unique_ptr$XMonotoneCurve$release(::std::unique_ptr<::XMonotoneCurve> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$XMonotoneCurve$drop(::std::unique_ptr<::XMonotoneCurve> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::XMonotoneCurve>::value>{}(ptr);
}

::std::vector<::XMonotoneCurve> *cxxbridge1$std$vector$XMonotoneCurve$new() noexcept {
  return new ::std::vector<::XMonotoneCurve>();
}
::std::size_t cxxbridge1$std$vector$XMonotoneCurve$size(::std::vector<::XMonotoneCurve> const &s) noexcept {
  return s.size();
}
::XMonotoneCurve *cxxbridge1$std$vector$XMonotoneCurve$get_unchecked(::std::vector<::XMonotoneCurve> *s, ::std::size_t pos) noexcept {
  return &(*s)[pos];
}
static_assert(sizeof(::std::unique_ptr<::std::vector<::XMonotoneCurve>>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::std::vector<::XMonotoneCurve>>) == alignof(void *), "");
void cxxbridge1$unique_ptr$std$vector$XMonotoneCurve$null(::std::unique_ptr<::std::vector<::XMonotoneCurve>> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::XMonotoneCurve>>();
}
void cxxbridge1$unique_ptr$std$vector$XMonotoneCurve$raw(::std::unique_ptr<::std::vector<::XMonotoneCurve>> *ptr, ::std::vector<::XMonotoneCurve> *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::XMonotoneCurve>>(raw);
}
::std::vector<::XMonotoneCurve> const *cxxbridge1$unique_ptr$std$vector$XMonotoneCurve$get(::std::unique_ptr<::std::vector<::XMonotoneCurve>> const &ptr) noexcept {
  return ptr.get();
}
::std::vector<::XMonotoneCurve> *cxxbridge1$unique_ptr$std$vector$XMonotoneCurve$release(::std::unique_ptr<::std::vector<::XMonotoneCurve>> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$std$vector$XMonotoneCurve$drop(::std::unique_ptr<::std::vector<::XMonotoneCurve>> *ptr) noexcept {
  ptr->~unique_ptr();
}

static_assert(::rust::detail::is_complete<::EllipseData>::value, "definition of EllipseData is required");
static_assert(sizeof(::std::unique_ptr<::EllipseData>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::EllipseData>) == alignof(void *), "");
void cxxbridge1$unique_ptr$EllipseData$null(::std::unique_ptr<::EllipseData> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::EllipseData>();
}
void cxxbridge1$unique_ptr$EllipseData$raw(::std::unique_ptr<::EllipseData> *ptr, ::EllipseData *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::EllipseData>(raw);
}
::EllipseData const *cxxbridge1$unique_ptr$EllipseData$get(::std::unique_ptr<::EllipseData> const &ptr) noexcept {
  return ptr.get();
}
::EllipseData *cxxbridge1$unique_ptr$EllipseData$release(::std::unique_ptr<::EllipseData> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$EllipseData$drop(::std::unique_ptr<::EllipseData> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::EllipseData>::value>{}(ptr);
}

static_assert(::rust::detail::is_complete<::Polygon>::value, "definition of Polygon is required");
static_assert(sizeof(::std::unique_ptr<::Polygon>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::Polygon>) == alignof(void *), "");
void cxxbridge1$unique_ptr$Polygon$null(::std::unique_ptr<::Polygon> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::Polygon>();
}
void cxxbridge1$unique_ptr$Polygon$raw(::std::unique_ptr<::Polygon> *ptr, ::Polygon *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::Polygon>(raw);
}
::Polygon const *cxxbridge1$unique_ptr$Polygon$get(::std::unique_ptr<::Polygon> const &ptr) noexcept {
  return ptr.get();
}
::Polygon *cxxbridge1$unique_ptr$Polygon$release(::std::unique_ptr<::Polygon> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$Polygon$drop(::std::unique_ptr<::Polygon> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::Polygon>::value>{}(ptr);
}

static_assert(::rust::detail::is_complete<::CurveIterator>::value, "definition of CurveIterator is required");
static_assert(sizeof(::std::unique_ptr<::CurveIterator>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::CurveIterator>) == alignof(void *), "");
void cxxbridge1$unique_ptr$CurveIterator$null(::std::unique_ptr<::CurveIterator> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::CurveIterator>();
}
void cxxbridge1$unique_ptr$CurveIterator$raw(::std::unique_ptr<::CurveIterator> *ptr, ::CurveIterator *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::CurveIterator>(raw);
}
::CurveIterator const *cxxbridge1$unique_ptr$CurveIterator$get(::std::unique_ptr<::CurveIterator> const &ptr) noexcept {
  return ptr.get();
}
::CurveIterator *cxxbridge1$unique_ptr$CurveIterator$release(::std::unique_ptr<::CurveIterator> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$CurveIterator$drop(::std::unique_ptr<::CurveIterator> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::CurveIterator>::value>{}(ptr);
}

static_assert(::rust::detail::is_complete<::PolygonWithHoles>::value, "definition of PolygonWithHoles is required");
static_assert(sizeof(::std::unique_ptr<::PolygonWithHoles>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::PolygonWithHoles>) == alignof(void *), "");
void cxxbridge1$unique_ptr$PolygonWithHoles$null(::std::unique_ptr<::PolygonWithHoles> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::PolygonWithHoles>();
}
void cxxbridge1$unique_ptr$PolygonWithHoles$raw(::std::unique_ptr<::PolygonWithHoles> *ptr, ::PolygonWithHoles *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::PolygonWithHoles>(raw);
}
::PolygonWithHoles const *cxxbridge1$unique_ptr$PolygonWithHoles$get(::std::unique_ptr<::PolygonWithHoles> const &ptr) noexcept {
  return ptr.get();
}
::PolygonWithHoles *cxxbridge1$unique_ptr$PolygonWithHoles$release(::std::unique_ptr<::PolygonWithHoles> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$PolygonWithHoles$drop(::std::unique_ptr<::PolygonWithHoles> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::PolygonWithHoles>::value>{}(ptr);
}

static_assert(::rust::detail::is_complete<::HoleIterator>::value, "definition of HoleIterator is required");
static_assert(sizeof(::std::unique_ptr<::HoleIterator>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::HoleIterator>) == alignof(void *), "");
void cxxbridge1$unique_ptr$HoleIterator$null(::std::unique_ptr<::HoleIterator> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::HoleIterator>();
}
void cxxbridge1$unique_ptr$HoleIterator$raw(::std::unique_ptr<::HoleIterator> *ptr, ::HoleIterator *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::HoleIterator>(raw);
}
::HoleIterator const *cxxbridge1$unique_ptr$HoleIterator$get(::std::unique_ptr<::HoleIterator> const &ptr) noexcept {
  return ptr.get();
}
::HoleIterator *cxxbridge1$unique_ptr$HoleIterator$release(::std::unique_ptr<::HoleIterator> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$HoleIterator$drop(::std::unique_ptr<::HoleIterator> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::HoleIterator>::value>{}(ptr);
}

static_assert(::rust::detail::is_complete<::PolygonSet>::value, "definition of PolygonSet is required");
static_assert(sizeof(::std::unique_ptr<::PolygonSet>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::PolygonSet>) == alignof(void *), "");
void cxxbridge1$unique_ptr$PolygonSet$null(::std::unique_ptr<::PolygonSet> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::PolygonSet>();
}
void cxxbridge1$unique_ptr$PolygonSet$raw(::std::unique_ptr<::PolygonSet> *ptr, ::PolygonSet *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::PolygonSet>(raw);
}
::PolygonSet const *cxxbridge1$unique_ptr$PolygonSet$get(::std::unique_ptr<::PolygonSet> const &ptr) noexcept {
  return ptr.get();
}
::PolygonSet *cxxbridge1$unique_ptr$PolygonSet$release(::std::unique_ptr<::PolygonSet> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$PolygonSet$drop(::std::unique_ptr<::PolygonSet> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::PolygonSet>::value>{}(ptr);
}

::std::vector<::PolygonWithHoles> *cxxbridge1$std$vector$PolygonWithHoles$new() noexcept {
  return new ::std::vector<::PolygonWithHoles>();
}
::std::size_t cxxbridge1$std$vector$PolygonWithHoles$size(::std::vector<::PolygonWithHoles> const &s) noexcept {
  return s.size();
}
::PolygonWithHoles *cxxbridge1$std$vector$PolygonWithHoles$get_unchecked(::std::vector<::PolygonWithHoles> *s, ::std::size_t pos) noexcept {
  return &(*s)[pos];
}
static_assert(sizeof(::std::unique_ptr<::std::vector<::PolygonWithHoles>>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::std::vector<::PolygonWithHoles>>) == alignof(void *), "");
void cxxbridge1$unique_ptr$std$vector$PolygonWithHoles$null(::std::unique_ptr<::std::vector<::PolygonWithHoles>> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::PolygonWithHoles>>();
}
void cxxbridge1$unique_ptr$std$vector$PolygonWithHoles$raw(::std::unique_ptr<::std::vector<::PolygonWithHoles>> *ptr, ::std::vector<::PolygonWithHoles> *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::PolygonWithHoles>>(raw);
}
::std::vector<::PolygonWithHoles> const *cxxbridge1$unique_ptr$std$vector$PolygonWithHoles$get(::std::unique_ptr<::std::vector<::PolygonWithHoles>> const &ptr) noexcept {
  return ptr.get();
}
::std::vector<::PolygonWithHoles> *cxxbridge1$unique_ptr$std$vector$PolygonWithHoles$release(::std::unique_ptr<::std::vector<::PolygonWithHoles>> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$std$vector$PolygonWithHoles$drop(::std::unique_ptr<::std::vector<::PolygonWithHoles>> *ptr) noexcept {
  ptr->~unique_ptr();
}
} // extern "C"
