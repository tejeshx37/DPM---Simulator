#pragma once

#include <array>
#include <vector>
#include <memory>
#include "kernel.h"

namespace Triangulation
{
    using Kernel = EpicKernel;
    using EpickPoint = Kernel::Point_3;
    using PointPair = std::pair<EpickPoint, EpickPoint>;
    using Constraints = std::vector<EpickPoint>; // Simplified for 3D: just a point cloud for convex hull
    using IndexPair = std::pair<std::size_t, std::size_t>;
    using Face = std::array<std::size_t, 4>; // Tetrahedron
    using Vertex = std::pair<EpickPoint, std::vector<std::size_t>>;

    class Data
    {
    public:
        std::vector<Face> &faces() noexcept;
        const std::vector<Face> &faces() const noexcept;

        std::vector<IndexPair> &edges() noexcept;
        const std::vector<IndexPair> &edges() const noexcept;

        std::vector<Vertex> &vertices() noexcept;
        const std::vector<Vertex> &vertices() const noexcept;

    private:
        std::vector<Face> m_faces;
        std::vector<IndexPair> m_edges;
        std::vector<Vertex> m_vertices;
    };

    std::unique_ptr<EpickPoint> create_epick_point(const double x, const double y, const double z);

    std::unique_ptr<Data> triangulate(const Constraints &constraints,
                                      const double aspect_bound,
                                      const double size_bound);

    inline double get_x(const EpickPoint &p) { return p.x(); }
    inline double get_y(const EpickPoint &p) { return p.y(); }
    inline double get_z(const EpickPoint &p) { return p.z(); }
}

