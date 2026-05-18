#pragma once

#include <CGAL/Nef_polyhedron_3.h>
// Nef_polyhedron_3 requires EXACT constructions (EPECK) for correctness.
// EPICK (inexact constructions) causes assertion failures in Sphere_segment
// when converting Polyhedron_3 to Nef_polyhedron_3.
#include <CGAL/Exact_predicates_exact_constructions_kernel.h>
#include "num.h"
#include <memory>
#include <vector>
#include <cstdint>

// Separate kernel just for Nef ops — must be EPECK
typedef CGAL::Exact_predicates_exact_constructions_kernel NefKernel;
typedef CGAL::Nef_polyhedron_3<NefKernel>                Nef_polyhedron;

#include <vector>
#include <memory>
#include "rust/cxx.h"

// Forward-declare CXX bridge types (defined in generated src/lib.rs.h)
struct Mesh3D;
struct Point3D;

class PolyhedronSet3 {
public:
    mutable Nef_polyhedron nef;

    PolyhedronSet3();
    explicit PolyhedronSet3(const Nef_polyhedron& n);

    bool is_empty() const;
    bool is_valid() const;
    void join(const PolyhedronSet3& other);
    void difference(const PolyhedronSet3& other);
    void intersection(const PolyhedronSet3& other);

    void get_mesh_data(std::vector<double>& out_vertices, std::vector<std::uint32_t>& out_triangles) const;
    Mesh3D get_mesh() const;
    std::unique_ptr<std::vector<Point3D>> get_vertices() const;
    std::unique_ptr<std::vector<std::uint32_t>> get_triangles() const;
};

std::unique_ptr<PolyhedronSet3> create_polyhedron_set();
std::unique_ptr<PolyhedronSet3> create_polyhedron_set_clone(const PolyhedronSet3& other);
std::unique_ptr<PolyhedronSet3> create_cube(const Rational& cx, const Rational& cy, const Rational& cz, const Rational& size);
std::unique_ptr<PolyhedronSet3> create_cuboid(const Rational& cx, const Rational& cy, const Rational& cz, const Rational& width, const Rational& height, const Rational& depth);
std::unique_ptr<PolyhedronSet3> create_approximated_sphere(const Rational& cx, const Rational& cy, const Rational& cz, const Rational& radius, std::uint32_t num_lat, std::uint32_t num_lon);
std::unique_ptr<PolyhedronSet3> create_approximated_cone(const Rational& cx, const Rational& cy, const Rational& cz, const Rational& radius, const Rational& height, std::uint32_t num_segments);
std::unique_ptr<PolyhedronSet3> create_approximated_cylinder(const Rational& cx, const Rational& cy, const Rational& cz, const Rational& radius, const Rational& height, std::uint32_t num_segments);
std::unique_ptr<PolyhedronSet3> create_extruded_polygon(rust::Slice<const double> pts_x, rust::Slice<const double> pts_y, const Rational& height);

// CXX Bridge free-function wrappers
Mesh3D get_mesh(const PolyhedronSet3& set);
std::unique_ptr<std::vector<Point3D>> get_vertices(const PolyhedronSet3& set);
std::unique_ptr<std::vector<std::uint32_t>> get_triangles(const PolyhedronSet3& set);

