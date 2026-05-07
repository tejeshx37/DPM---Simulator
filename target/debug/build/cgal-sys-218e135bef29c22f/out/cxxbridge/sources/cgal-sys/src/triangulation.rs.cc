#include "cgal-sys/cpp/pair_utils.h"
#include "cgal-sys/cpp/triangulation.h"
#include "cgal-sys/cpp/vector_utils.h"
#include <cstddef>
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

namespace Triangulation {
  using EpickPoint = ::Triangulation::EpickPoint;
  using Face = ::Triangulation::Face;
  using IndexPair = ::Triangulation::IndexPair;
  using Vertex = ::Triangulation::Vertex;
  using Data = ::Triangulation::Data;
  using Constraints = ::Triangulation::Constraints;
}

namespace Triangulation {
extern "C" {
::Triangulation::EpickPoint *Triangulation$cxxbridge1$create_epick_point(double x, double y, double z) noexcept {
  ::std::unique_ptr<::Triangulation::EpickPoint> (*create_epick_point$)(double, double, double) = ::Triangulation::create_epick_point;
  return create_epick_point$(x, y, z).release();
}

double Triangulation$cxxbridge1$x(::Triangulation::EpickPoint const &p) noexcept {
  double (*x$)(::Triangulation::EpickPoint const &) = ::Triangulation::get_x;
  return x$(p);
}

double Triangulation$cxxbridge1$y(::Triangulation::EpickPoint const &p) noexcept {
  double (*y$)(::Triangulation::EpickPoint const &) = ::Triangulation::get_y;
  return y$(p);
}

double Triangulation$cxxbridge1$z(::Triangulation::EpickPoint const &p) noexcept {
  double (*z$)(::Triangulation::EpickPoint const &) = ::Triangulation::get_z;
  return z$(p);
}

::std::size_t const *Triangulation$cxxbridge1$Face$at(::Triangulation::Face const &self, ::std::size_t index) noexcept {
  ::std::size_t const &(::Triangulation::Face::*at$)(::std::size_t) const = &::Triangulation::Face::at;
  return &(self.*at$)(index);
}
} // extern "C"
} // namespace Triangulation

extern "C" {
::std::size_t cxxbridge1$get_first_index(::Triangulation::IndexPair const &pair) noexcept {
  ::std::size_t (*get_first_index$)(::Triangulation::IndexPair const &) = ::first;
  return get_first_index$(pair);
}

::std::size_t cxxbridge1$get_second_index(::Triangulation::IndexPair const &pair) noexcept {
  ::std::size_t (*get_second_index$)(::Triangulation::IndexPair const &) = ::second;
  return get_second_index$(pair);
}

::Triangulation::EpickPoint const *cxxbridge1$get_point(::Triangulation::Vertex const &vertex) noexcept {
  ::Triangulation::EpickPoint const &(*get_point$)(::Triangulation::Vertex const &) = ::first;
  return &get_point$(vertex);
}

::std::vector<::std::size_t> const *cxxbridge1$get_incident_faces(::Triangulation::Vertex const &vertex) noexcept {
  ::std::vector<::std::size_t> const &(*get_incident_faces$)(::Triangulation::Vertex const &) = ::second;
  return &get_incident_faces$(vertex);
}
} // extern "C"

namespace Triangulation {
extern "C" {
::std::vector<::Triangulation::Face> const *Triangulation$cxxbridge1$Data$faces(::Triangulation::Data const &self) noexcept {
  ::std::vector<::Triangulation::Face> const &(::Triangulation::Data::*faces$)() const = &::Triangulation::Data::faces;
  return &(self.*faces$)();
}

::std::vector<::Triangulation::IndexPair> const *Triangulation$cxxbridge1$Data$edges(::Triangulation::Data const &self) noexcept {
  ::std::vector<::Triangulation::IndexPair> const &(::Triangulation::Data::*edges$)() const = &::Triangulation::Data::edges;
  return &(self.*edges$)();
}

::std::vector<::Triangulation::Vertex> const *Triangulation$cxxbridge1$Data$vertices(::Triangulation::Data const &self) noexcept {
  ::std::vector<::Triangulation::Vertex> const &(::Triangulation::Data::*vertices$)() const = &::Triangulation::Data::vertices;
  return &(self.*vertices$)();
}
} // extern "C"
} // namespace Triangulation

extern "C" {
::Triangulation::Constraints *cxxbridge1$create_constraints(::std::size_t capacity) noexcept {
  ::std::unique_ptr<::Triangulation::Constraints> (*create_constraints$)(::std::size_t) = ::create_vector;
  return create_constraints$(capacity).release();
}
} // extern "C"

namespace Triangulation {
extern "C" {
void Triangulation$cxxbridge1$Constraints$reserve(::Triangulation::Constraints &self, ::std::size_t capacity) noexcept {
  void (::Triangulation::Constraints::*reserve$)(::std::size_t) = &::Triangulation::Constraints::reserve;
  (self.*reserve$)(capacity);
}
} // extern "C"
} // namespace Triangulation

extern "C" {
void cxxbridge1$push_back(::Triangulation::Constraints &vec, ::Triangulation::EpickPoint *constraint) noexcept {
  void (*push_back$)(::Triangulation::Constraints &, ::std::unique_ptr<::Triangulation::EpickPoint>) = ::push_back;
  push_back$(vec, ::std::unique_ptr<::Triangulation::EpickPoint>(constraint));
}
} // extern "C"

namespace Triangulation {
extern "C" {
::rust::repr::PtrLen Triangulation$cxxbridge1$triangulate(::Triangulation::Constraints const &constraints, double aspect_bound, double size_bound, ::Triangulation::Data **return$) noexcept {
  ::std::unique_ptr<::Triangulation::Data> (*triangulate$)(::Triangulation::Constraints const &, double, double) = ::Triangulation::triangulate;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Triangulation::Data *(triangulate$(constraints, aspect_bound, size_bound).release());
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}
} // extern "C"
} // namespace Triangulation

extern "C" {
static_assert(::rust::detail::is_complete<::Triangulation::EpickPoint>::value, "definition of EpickPoint is required");
static_assert(sizeof(::std::unique_ptr<::Triangulation::EpickPoint>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::Triangulation::EpickPoint>) == alignof(void *), "");
void cxxbridge1$unique_ptr$Triangulation$EpickPoint$null(::std::unique_ptr<::Triangulation::EpickPoint> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::Triangulation::EpickPoint>();
}
void cxxbridge1$unique_ptr$Triangulation$EpickPoint$raw(::std::unique_ptr<::Triangulation::EpickPoint> *ptr, ::Triangulation::EpickPoint *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::Triangulation::EpickPoint>(raw);
}
::Triangulation::EpickPoint const *cxxbridge1$unique_ptr$Triangulation$EpickPoint$get(::std::unique_ptr<::Triangulation::EpickPoint> const &ptr) noexcept {
  return ptr.get();
}
::Triangulation::EpickPoint *cxxbridge1$unique_ptr$Triangulation$EpickPoint$release(::std::unique_ptr<::Triangulation::EpickPoint> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$Triangulation$EpickPoint$drop(::std::unique_ptr<::Triangulation::EpickPoint> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::Triangulation::EpickPoint>::value>{}(ptr);
}

::std::vector<::Triangulation::Face> *cxxbridge1$std$vector$Triangulation$Face$new() noexcept {
  return new ::std::vector<::Triangulation::Face>();
}
::std::size_t cxxbridge1$std$vector$Triangulation$Face$size(::std::vector<::Triangulation::Face> const &s) noexcept {
  return s.size();
}
::Triangulation::Face *cxxbridge1$std$vector$Triangulation$Face$get_unchecked(::std::vector<::Triangulation::Face> *s, ::std::size_t pos) noexcept {
  return &(*s)[pos];
}
static_assert(sizeof(::std::unique_ptr<::std::vector<::Triangulation::Face>>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::std::vector<::Triangulation::Face>>) == alignof(void *), "");
void cxxbridge1$unique_ptr$std$vector$Triangulation$Face$null(::std::unique_ptr<::std::vector<::Triangulation::Face>> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::Triangulation::Face>>();
}
void cxxbridge1$unique_ptr$std$vector$Triangulation$Face$raw(::std::unique_ptr<::std::vector<::Triangulation::Face>> *ptr, ::std::vector<::Triangulation::Face> *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::Triangulation::Face>>(raw);
}
::std::vector<::Triangulation::Face> const *cxxbridge1$unique_ptr$std$vector$Triangulation$Face$get(::std::unique_ptr<::std::vector<::Triangulation::Face>> const &ptr) noexcept {
  return ptr.get();
}
::std::vector<::Triangulation::Face> *cxxbridge1$unique_ptr$std$vector$Triangulation$Face$release(::std::unique_ptr<::std::vector<::Triangulation::Face>> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$std$vector$Triangulation$Face$drop(::std::unique_ptr<::std::vector<::Triangulation::Face>> *ptr) noexcept {
  ptr->~unique_ptr();
}

::std::vector<::Triangulation::IndexPair> *cxxbridge1$std$vector$Triangulation$IndexPair$new() noexcept {
  return new ::std::vector<::Triangulation::IndexPair>();
}
::std::size_t cxxbridge1$std$vector$Triangulation$IndexPair$size(::std::vector<::Triangulation::IndexPair> const &s) noexcept {
  return s.size();
}
::Triangulation::IndexPair *cxxbridge1$std$vector$Triangulation$IndexPair$get_unchecked(::std::vector<::Triangulation::IndexPair> *s, ::std::size_t pos) noexcept {
  return &(*s)[pos];
}
static_assert(sizeof(::std::unique_ptr<::std::vector<::Triangulation::IndexPair>>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::std::vector<::Triangulation::IndexPair>>) == alignof(void *), "");
void cxxbridge1$unique_ptr$std$vector$Triangulation$IndexPair$null(::std::unique_ptr<::std::vector<::Triangulation::IndexPair>> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::Triangulation::IndexPair>>();
}
void cxxbridge1$unique_ptr$std$vector$Triangulation$IndexPair$raw(::std::unique_ptr<::std::vector<::Triangulation::IndexPair>> *ptr, ::std::vector<::Triangulation::IndexPair> *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::Triangulation::IndexPair>>(raw);
}
::std::vector<::Triangulation::IndexPair> const *cxxbridge1$unique_ptr$std$vector$Triangulation$IndexPair$get(::std::unique_ptr<::std::vector<::Triangulation::IndexPair>> const &ptr) noexcept {
  return ptr.get();
}
::std::vector<::Triangulation::IndexPair> *cxxbridge1$unique_ptr$std$vector$Triangulation$IndexPair$release(::std::unique_ptr<::std::vector<::Triangulation::IndexPair>> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$std$vector$Triangulation$IndexPair$drop(::std::unique_ptr<::std::vector<::Triangulation::IndexPair>> *ptr) noexcept {
  ptr->~unique_ptr();
}

::std::vector<::Triangulation::Vertex> *cxxbridge1$std$vector$Triangulation$Vertex$new() noexcept {
  return new ::std::vector<::Triangulation::Vertex>();
}
::std::size_t cxxbridge1$std$vector$Triangulation$Vertex$size(::std::vector<::Triangulation::Vertex> const &s) noexcept {
  return s.size();
}
::Triangulation::Vertex *cxxbridge1$std$vector$Triangulation$Vertex$get_unchecked(::std::vector<::Triangulation::Vertex> *s, ::std::size_t pos) noexcept {
  return &(*s)[pos];
}
static_assert(sizeof(::std::unique_ptr<::std::vector<::Triangulation::Vertex>>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::std::vector<::Triangulation::Vertex>>) == alignof(void *), "");
void cxxbridge1$unique_ptr$std$vector$Triangulation$Vertex$null(::std::unique_ptr<::std::vector<::Triangulation::Vertex>> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::Triangulation::Vertex>>();
}
void cxxbridge1$unique_ptr$std$vector$Triangulation$Vertex$raw(::std::unique_ptr<::std::vector<::Triangulation::Vertex>> *ptr, ::std::vector<::Triangulation::Vertex> *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::std::vector<::Triangulation::Vertex>>(raw);
}
::std::vector<::Triangulation::Vertex> const *cxxbridge1$unique_ptr$std$vector$Triangulation$Vertex$get(::std::unique_ptr<::std::vector<::Triangulation::Vertex>> const &ptr) noexcept {
  return ptr.get();
}
::std::vector<::Triangulation::Vertex> *cxxbridge1$unique_ptr$std$vector$Triangulation$Vertex$release(::std::unique_ptr<::std::vector<::Triangulation::Vertex>> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$std$vector$Triangulation$Vertex$drop(::std::unique_ptr<::std::vector<::Triangulation::Vertex>> *ptr) noexcept {
  ptr->~unique_ptr();
}

static_assert(::rust::detail::is_complete<::Triangulation::Constraints>::value, "definition of Constraints is required");
static_assert(sizeof(::std::unique_ptr<::Triangulation::Constraints>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::Triangulation::Constraints>) == alignof(void *), "");
void cxxbridge1$unique_ptr$Triangulation$Constraints$null(::std::unique_ptr<::Triangulation::Constraints> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::Triangulation::Constraints>();
}
void cxxbridge1$unique_ptr$Triangulation$Constraints$raw(::std::unique_ptr<::Triangulation::Constraints> *ptr, ::Triangulation::Constraints *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::Triangulation::Constraints>(raw);
}
::Triangulation::Constraints const *cxxbridge1$unique_ptr$Triangulation$Constraints$get(::std::unique_ptr<::Triangulation::Constraints> const &ptr) noexcept {
  return ptr.get();
}
::Triangulation::Constraints *cxxbridge1$unique_ptr$Triangulation$Constraints$release(::std::unique_ptr<::Triangulation::Constraints> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$Triangulation$Constraints$drop(::std::unique_ptr<::Triangulation::Constraints> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::Triangulation::Constraints>::value>{}(ptr);
}

static_assert(::rust::detail::is_complete<::Triangulation::Data>::value, "definition of Data is required");
static_assert(sizeof(::std::unique_ptr<::Triangulation::Data>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::Triangulation::Data>) == alignof(void *), "");
void cxxbridge1$unique_ptr$Triangulation$Data$null(::std::unique_ptr<::Triangulation::Data> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::Triangulation::Data>();
}
void cxxbridge1$unique_ptr$Triangulation$Data$raw(::std::unique_ptr<::Triangulation::Data> *ptr, ::Triangulation::Data *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::Triangulation::Data>(raw);
}
::Triangulation::Data const *cxxbridge1$unique_ptr$Triangulation$Data$get(::std::unique_ptr<::Triangulation::Data> const &ptr) noexcept {
  return ptr.get();
}
::Triangulation::Data *cxxbridge1$unique_ptr$Triangulation$Data$release(::std::unique_ptr<::Triangulation::Data> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$Triangulation$Data$drop(::std::unique_ptr<::Triangulation::Data> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::Triangulation::Data>::value>{}(ptr);
}
} // extern "C"
