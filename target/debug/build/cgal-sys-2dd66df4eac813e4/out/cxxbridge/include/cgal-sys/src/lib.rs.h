#pragma once
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
#include <memory>
#include <string>
#include <type_traits>
#include <vector>

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
