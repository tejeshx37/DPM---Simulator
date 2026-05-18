#pragma once

#include <array>
#include <vector>
#include <memory>
#include "kernel.h"

namespace Triangulation2
{
    using Kernel = EpicKernel;
    using EpickPoint2 = Kernel::Point_2;
    using Constraints = std::vector<EpickPoint2>;
    using IndexPair = std::pair<std::size_t, std::size_t>;
    using Face2D = std::array<std::size_t, 3>; // Triangle
    using Vertex2D = std::pair<EpickPoint2, std::vector<std::size_t>>;

    class Data2D
    {
    public:
        std::vector<Face2D> &faces() noexcept;
        const std::vector<Face2D> &faces() const noexcept;

        std::vector<IndexPair> &edges() noexcept;
        const std::vector<IndexPair> &edges() const noexcept;

        std::vector<Vertex2D> &vertices() noexcept;
        const std::vector<Vertex2D> &vertices() const noexcept;

    private:
        std::vector<Face2D> m_faces;
        std::vector<IndexPair> m_edges;
        std::vector<Vertex2D> m_vertices;
    };

    std::unique_ptr<EpickPoint2> create_epick_point_2(const double x, const double y);

    std::unique_ptr<Data2D> triangulate_2(const Constraints &constraints,
                                          const double aspect_bound,
                                          const double size_bound);

    inline double get_x_2(const EpickPoint2 &p) { return p.x(); }
    inline double get_y_2(const EpickPoint2 &p) { return p.y(); }
}
