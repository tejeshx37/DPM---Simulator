#include <unordered_map>
#include <CGAL/Cartesian_converter.h>
#include <CGAL/Delaunay_triangulation_3.h>

#include "triangulation.h"

namespace Triangulation
{
    typedef CGAL::Delaunay_triangulation_3<Kernel> DT3;
    typedef DT3::Vertex_handle VertexHandle;
    typedef DT3::Cell_handle CellHandle;
    typedef DT3::Edge Edge;

    std::vector<Face> &Data::faces() noexcept { return m_faces; }
    const std::vector<Face> &Data::faces() const noexcept { return m_faces; }

    std::vector<IndexPair> &Data::edges() noexcept { return m_edges; }
    const std::vector<IndexPair> &Data::edges() const noexcept { return m_edges; }

    std::vector<Vertex> &Data::vertices() noexcept { return m_vertices; }
    const std::vector<Vertex> &Data::vertices() const noexcept { return m_vertices; }

    std::unique_ptr<EpickPoint> create_epick_point(const double x, const double y, const double z)
    {
        return std::make_unique<EpickPoint>(x, y, z);
    }

    std::unique_ptr<Data> triangulate(const Constraints &constraints,
                                      const double aspect_bound,
                                      const double size_bound)
    {
        DT3 dt;
        dt.insert(constraints.begin(), constraints.end());

        auto data = std::make_unique<Data>();

        std::unordered_map<CellHandle, std::size_t> cell_index_map;
        cell_index_map.reserve(dt.number_of_finite_cells());
        {
            std::size_t index = 0;
            for (auto it = dt.finite_cells_begin(); it != dt.finite_cells_end(); ++it)
            {
                cell_index_map.emplace(it, index++);
            }
        }

        std::unordered_map<VertexHandle, std::size_t> vertex_index_map;
        data->vertices().reserve(dt.number_of_vertices());
        vertex_index_map.reserve(dt.number_of_vertices());
        {
            std::size_t index = 0;
            for (auto it = dt.finite_vertices_begin(); it != dt.finite_vertices_end(); ++it)
            {
                vertex_index_map.emplace(it, index++);
                std::vector<CellHandle> incident_cells;
                dt.incident_cells(it, std::back_inserter(incident_cells));
                
                std::vector<std::size_t> incident_cell_indices;
                for (auto cell : incident_cells) {
                    if (!dt.is_infinite(cell)) {
                        incident_cell_indices.push_back(cell_index_map.at(cell));
                    }
                }
                data->vertices().emplace_back(it->point(), std::move(incident_cell_indices));
            }
        }

        for (auto it = dt.finite_edges_begin(); it != dt.finite_edges_end(); ++it)
        {
            const auto vertex_index = [&](const int e) {
                return vertex_index_map.at(it->first->vertex(e));
            };
            data->edges().emplace_back(vertex_index(it->second), vertex_index(it->third));
        }

        data->faces().reserve(dt.number_of_finite_cells());
        for (auto it = dt.finite_cells_begin(); it != dt.finite_cells_end(); ++it)
        {
            const auto point_index = [&](const int i) {
                return vertex_index_map.at(it->vertex(i));
            };
            data->faces().push_back({point_index(0), point_index(1), point_index(2), point_index(3)});
        }

        return data;
    }
}