#pragma once

#include <CGAL/Arr_conic_traits_2.h>
#include <CGAL/Gps_traits_2.h>

#include "kernel.h"

typedef CGAL::Arr_conic_traits_2<RationalKernel, AlgebraicKernel, CORE_ANT> ConicTraits;
typedef CGAL::Gps_traits_2<ConicTraits> GPSTraits;