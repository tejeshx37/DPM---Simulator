#define CGAL_USE_CORE
#include "polyhedron_set.h"
#include <CGAL/Nef_polyhedron_3.h>
#include <CGAL/Polyhedron_3.h>
#include <CGAL/Polyhedron_incremental_builder_3.h>
#include <CGAL/Polygon_mesh_processing/triangulate_faces.h>
#include <CGAL/boost/graph/convert_nef_polyhedron_to_polygon_mesh.h>
#include <CGAL/convex_hull_3.h>
#include <map>
#include <cmath>
#include <vector>
#include <cstdint>

#include "cgal-sys/src/lib.rs.h"

using namespace rust;

// All geometry uses the exact-constructions kernel (NefKernel = EPECK)
// so that Nef_polyhedron_3 operations are numerically reliable.
typedef CGAL::Polyhedron_3<NefKernel>   Polyhedron;
typedef NefKernel::Point_3              Point_3;
typedef NefKernel::Vector_3             Vector_3;
typedef Polyhedron::HalfedgeDS          HDS;

// ---------------------------------------------------------------------------
// Generic incremental mesh builder
// Avoids halfspace-plane Nef construction which does not work with EPICK/EPECK
// standard kernels (would trigger "Constructor not available for this Kernel").
// ---------------------------------------------------------------------------
template<class HDS_>
struct MeshBuilder : public CGAL::Modifier_base<HDS_> {
    std::vector<Point_3>                   vertices;
    std::vector<std::vector<std::size_t>>  faces;

    void operator()(HDS_& hds) override {
        // Set check_planarity to false to avoid rejection of slightly non-planar faces
        // although we are now moving to triangles for primitives anyway.
        CGAL::Polyhedron_incremental_builder_3<HDS_> B(hds, false);
        B.begin_surface(vertices.size(), faces.size());
        for (auto& p : vertices) B.add_vertex(p);
        for (auto& f : faces) {
            B.begin_facet();
            for (auto i : f) B.add_vertex_to_facet(i);
            B.end_facet();
        }
        B.end_surface();
    }
};

// Triangulate any quads and wrap in Nef_polyhedron
static Nef_polyhedron polyhedron_to_nef(Polyhedron& P) {
    CGAL::Polygon_mesh_processing::triangulate_faces(P);
    return Nef_polyhedron(P);
}

// ---------------------------------------------------------------------------
// PolyhedronSet3
// ---------------------------------------------------------------------------
PolyhedronSet3::PolyhedronSet3() : nef(Nef_polyhedron::EMPTY) {}
PolyhedronSet3::PolyhedronSet3(const Nef_polyhedron& n) : nef(n) {}

bool PolyhedronSet3::is_empty() const  { return nef.is_empty(); }

void PolyhedronSet3::join(const PolyhedronSet3& o) {
    try {
        nef += o.nef;
    } catch (const std::exception& e) {
        throw std::runtime_error(std::string("CGAL Join failed: ") + e.what());
    }
}

void PolyhedronSet3::difference(const PolyhedronSet3& o) {
    try {
        nef -= o.nef;
    } catch (const std::exception& e) {
        throw std::runtime_error(std::string("CGAL Difference failed: ") + e.what());
    }
}

void PolyhedronSet3::intersection(const PolyhedronSet3& o) {
    try {
        nef *= o.nef;
    } catch (const std::exception& e) {
        throw std::runtime_error(std::string("CGAL Intersection failed: ") + e.what());
    }
}

static void extract_mesh(const Nef_polyhedron& nef, Polyhedron& P) {
    if (nef.is_empty()) return;
    // convert_nef_polyhedron_to_polygon_mesh is more robust than nef.convert_to_polyhedron(P)
    // as it handles non-simple Nef polyhedra (e.g. multiple disjoint components).
    CGAL::convert_nef_polyhedron_to_polygon_mesh(nef, P);
}

void PolyhedronSet3::get_mesh_data(
    std::vector<double>&   out_vertices,
    std::vector<uint32_t>& out_triangles) const
{
    if (nef.is_empty()) return;

    Polyhedron P;
    extract_mesh(nef, P);
    CGAL::Polygon_mesh_processing::triangulate_faces(P);

    // Build vertex list and index map in one pass
    std::map<Polyhedron::Vertex_const_handle, uint32_t> vmap;
    uint32_t idx = 0;
    for (auto vi = P.vertices_begin(); vi != P.vertices_end(); ++vi) {
        auto& p = vi->point();
        out_vertices.push_back(CGAL::to_double(p.x()));
        out_vertices.push_back(CGAL::to_double(p.y()));
        out_vertices.push_back(CGAL::to_double(p.z()));
        vmap[vi] = idx++;
    }

    // Fan-triangulate every face
    for (auto fi = P.facets_begin(); fi != P.facets_end(); ++fi) {
        auto h       = fi->facet_begin();
        auto h_start = h;
        uint32_t v0  = vmap[h->vertex()]; ++h;
        uint32_t v1  = vmap[h->vertex()]; ++h;
        while (h != h_start) {
            uint32_t v2 = vmap[h->vertex()];
            out_triangles.push_back(v0);
            out_triangles.push_back(v1);
            out_triangles.push_back(v2);
            v1 = v2;
            ++h;
        }
    }
}

// CXX Bridge Helpers
Mesh3D PolyhedronSet3::get_mesh() const {
    std::vector<double> v;
    std::vector<uint32_t> t;
    get_mesh_data(v, t);
    
    Mesh3D out;
    for (size_t i = 0; i < v.size(); i += 3) {
        out.vertices.push_back({v[i], v[i+1], v[i+2]});
    }
    out.triangles.reserve(t.size());
    for (auto val : t) {
        out.triangles.push_back(val);
    }
    return out;
}

std::unique_ptr<std::vector<Point3D>> PolyhedronSet3::get_vertices() const {
    std::vector<double> v;
    std::vector<uint32_t> t;
    get_mesh_data(v, t);
    auto out = std::make_unique<std::vector<Point3D>>();
    for (size_t i = 0; i < v.size(); i += 3) {
        out->push_back({v[i], v[i+1], v[i+2]});
    }
    return out;
}

std::unique_ptr<std::vector<uint32_t>> PolyhedronSet3::get_triangles() const {
    std::vector<double> v;
    auto t = std::make_unique<std::vector<uint32_t>>();
    get_mesh_data(v, *t);
    return t;
}


// ---------------------------------------------------------------------------
// CXX Bridge free-function wrappers
// (CXX expects free functions for non-self-method declarations)
// ---------------------------------------------------------------------------
Mesh3D get_mesh(const PolyhedronSet3& set) { return set.get_mesh(); }
std::unique_ptr<std::vector<Point3D>> get_vertices(const PolyhedronSet3& set) { return set.get_vertices(); }
std::unique_ptr<std::vector<uint32_t>> get_triangles(const PolyhedronSet3& set) { return set.get_triangles(); }

// ---------------------------------------------------------------------------
// Factory helpers
// ---------------------------------------------------------------------------
std::unique_ptr<PolyhedronSet3> create_polyhedron_set() {
    return std::make_unique<PolyhedronSet3>();
}
std::unique_ptr<PolyhedronSet3> create_polyhedron_set_clone(const PolyhedronSet3& other) {
    return std::make_unique<PolyhedronSet3>(other.nef);
}

// Cuboid: 8-vertex closed mesh — no halfspace planes, no Nef sphere assertion
std::unique_ptr<PolyhedronSet3> create_cuboid(
    const Rational& cx, const Rational& cy, const Rational& cz,
    const Rational& width, const Rational& height, const Rational& depth)
{
    double ox = CGAL::to_double(cx), oy = CGAL::to_double(cy), oz = CGAL::to_double(cz);
    double hw = CGAL::to_double(width)/2.0, hh = CGAL::to_double(height)/2.0, hd = CGAL::to_double(depth)/2.0;

    std::vector<Point_3> pts = {
        Point_3(ox-hw, oy-hh, oz-hd), Point_3(ox+hw, oy-hh, oz-hd),
        Point_3(ox+hw, oy+hh, oz-hd), Point_3(ox-hw, oy+hh, oz-hd),
        Point_3(ox-hw, oy-hh, oz+hd), Point_3(ox+hw, oy-hh, oz+hd),
        Point_3(ox+hw, oy+hh, oz+hd), Point_3(ox-hw, oy+hh, oz+hd),
    };

    Polyhedron P;
    CGAL::convex_hull_3(pts.begin(), pts.end(), P);
    return std::make_unique<PolyhedronSet3>(polyhedron_to_nef(P));
}

std::unique_ptr<PolyhedronSet3> create_cube(
    const Rational& cx, const Rational& cy, const Rational& cz, const Rational& size)
{
    return create_cuboid(cx, cy, cz, size, size, size);
}

// Sphere: lat/lon grid → convex hull (exact, no winding issues)
std::unique_ptr<PolyhedronSet3> create_approximated_sphere(
    const Rational& cx, const Rational& cy, const Rational& cz,
    const Rational& radius, uint32_t num_lat, uint32_t num_lon)
{
    double ox = CGAL::to_double(cx);
    double oy = CGAL::to_double(cy);
    double oz = CGAL::to_double(cz);
    double r  = CGAL::to_double(radius);
    if (num_lat < 2) num_lat = 8;
    if (num_lon < 3) num_lon = 12;

    std::vector<Point_3> pts;
    pts.push_back(Point_3(ox, oy, oz - r)); // south pole
    for (uint32_t la = 1; la < num_lat; ++la) {
        double phi = M_PI * la / num_lat - M_PI / 2.0;
        double z = r * std::sin(phi), rc = r * std::cos(phi);
        for (uint32_t lo = 0; lo < num_lon; ++lo) {
            double th = 2.0 * M_PI * lo / num_lon;
            pts.push_back(Point_3(ox + rc * std::cos(th), oy + rc * std::sin(th), oz + z));
        }
    }
    pts.push_back(Point_3(ox, oy, oz + r)); // north pole

    Polyhedron P;
    CGAL::convex_hull_3(pts.begin(), pts.end(), P);
    return std::make_unique<PolyhedronSet3>(polyhedron_to_nef(P));
}

// Cone: apex + base ring + base cap
std::unique_ptr<PolyhedronSet3> create_approximated_cone(
    const Rational& cx, const Rational& cy, const Rational& cz,
    const Rational& radius, const Rational& height, uint32_t num_segments)
{
    double ox = CGAL::to_double(cx), oy = CGAL::to_double(cy), oz = CGAL::to_double(cz);
    double r  = CGAL::to_double(radius), h = CGAL::to_double(height);
    if (num_segments < 3) num_segments = 16;
    uint32_t N = num_segments;

    std::vector<Point_3> pts;
    pts.push_back(Point_3(ox, oy, oz + h/2.0)); // apex
    for (uint32_t i = 0; i < N; ++i) {
        double th = 2.0 * M_PI * i / N;
        pts.push_back(Point_3(ox + r*std::cos(th), oy + r*std::sin(th), oz - h/2.0));
    }

    Polyhedron P;
    CGAL::convex_hull_3(pts.begin(), pts.end(), P);
    return std::make_unique<PolyhedronSet3>(polyhedron_to_nef(P));
}

// Cylinder: two rings + side quads + top/bottom caps
std::unique_ptr<PolyhedronSet3> create_approximated_cylinder(
    const Rational& cx, const Rational& cy, const Rational& cz,
    const Rational& radius, const Rational& height, uint32_t num_segments)
{
    double ox = CGAL::to_double(cx), oy = CGAL::to_double(cy), oz = CGAL::to_double(cz);
    double r  = CGAL::to_double(radius), h = CGAL::to_double(height);
    if (num_segments < 3) num_segments = 32;
    uint32_t N = num_segments;

    std::vector<Point_3> pts;
    for (uint32_t i = 0; i < N; ++i) {
        double th = 2.0 * M_PI * i / N;
        double x = ox + r*std::cos(th), y = oy + r*std::sin(th);
        pts.push_back(Point_3(x, y, oz - h/2.0));
        pts.push_back(Point_3(x, y, oz + h/2.0));
    }

    Polyhedron P;
    CGAL::convex_hull_3(pts.begin(), pts.end(), P);
    return std::make_unique<PolyhedronSet3>(polyhedron_to_nef(P));
}

// Extrude a 2D polygon (given as points) into a 3D prism
std::unique_ptr<PolyhedronSet3> create_extruded_polygon(
    rust::Slice<const double> pts_x,
    rust::Slice<const double> pts_y,
    const Rational& height)
{
    double h = CGAL::to_double(height);
    uint32_t N = pts_x.size();
    if (N < 3 || pts_x.size() != pts_y.size()) return std::make_unique<PolyhedronSet3>();

    MeshBuilder<HDS> b;
    // Bottom vertices
    for (uint32_t i = 0; i < N; ++i) {
        b.vertices.push_back(Point_3(pts_x[i], pts_y[i], -h/2.0));
    }
    // Top vertices
    for (uint32_t i = 0; i < N; ++i) {
        b.vertices.push_back(Point_3(pts_x[i], pts_y[i], h/2.0));
    }

    // Side faces (using triangles)
    for (uint32_t i = 0; i < N; ++i) {
        uint32_t next_i = (i + 1) % N;
        uint32_t b0 = i, b1 = next_i;
        uint32_t t0 = i + N, t1 = next_i + N;
        b.faces.push_back({b0, b1, t1});
        b.faces.push_back({b0, t1, t0});
    }

    // Caps
    // For arbitrary convex polygons, a fan from 0 is fine.
    // For concave polygons, this would need proper triangulation.
    // Assuming simple/convex for basic primitives right now.
    for (uint32_t i = 1; i < N - 1; ++i) {
        // Bottom cap (normal points -Z, so clockwise when looking from +Z)
        b.faces.push_back({0, i + 1, i});
        // Top cap (normal points +Z, so counter-clockwise)
        b.faces.push_back({N, N + i, N + i + 1});
    }

    Polyhedron P;
    P.delegate(b);
    return std::make_unique<PolyhedronSet3>(polyhedron_to_nef(P));
}
