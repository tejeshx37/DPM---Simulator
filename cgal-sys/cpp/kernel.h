#pragma once

#include <CGAL/Cartesian.h>
#include <CGAL/Exact_predicates_inexact_constructions_kernel.h>

#include "num.h"

typedef CGAL::Cartesian<Rational> RationalKernel;
typedef CGAL::Cartesian<Algebraic> AlgebraicKernel;
typedef CGAL::Exact_predicates_inexact_constructions_kernel EpicKernel;