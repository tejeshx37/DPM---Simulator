// Dummy comment to invalidate compiler cache
#include <unordered_map>
#include <CGAL/Cartesian_converter.h>
#include <CGAL/Delaunay_triangulation_2.h>

#include "triangulation_2.h"

namespace Triangulation2
{
    typedef CGAL::Delaunay_triangulation_2<Kernel> DT2;
    typedef DT2::Vertex_handle VertexHandle;
    typedef DT2::Face_handle FaceHandle;
    typedef DT2::Edge Edge;

    std::vector<Face2D> &Data2D::faces() noexcept { return m_faces; }
    const std::vector<Face2D> &Data2D::faces() const noexcept { return m_faces; }

    std::vector<IndexPair> &Data2D::edges() noexcept { return m_edges; }
    const std::vector<IndexPair> &Data2D::edges() const noexcept { return m_edges; }

    std::vector<Vertex2D> &Data2D::vertices() noexcept { return m_vertices; }
    const std::vector<Vertex2D> &Data2D::vertices() const noexcept { return m_vertices; }

    std::unique_ptr<EpickPoint2> create_epick_point_2(const double x, const double y)
    {
        return std::make_unique<EpickPoint2>(x, y);
    }

    std::unique_ptr<Data2D> triangulate_2(const Constraints &constraints,
                                          const double aspect_bound,
                                          const double size_bound)
    {
        DT2 dt;
        dt.insert(constraints.begin(), constraints.end());

        auto data = std::make_unique<Data2D>();

        std::unordered_map<FaceHandle, std::size_t> face_index_map;
        face_index_map.reserve(dt.number_of_faces());
        {
            std::size_t index = 0;
            for (auto it = dt.finite_faces_begin(); it != dt.finite_faces_end(); ++it)
            {
                face_index_map.emplace(it, index++);
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
                
                std::vector<FaceHandle> incident_faces;
                DT2::Face_circulator fc = dt.incident_faces(it);
                DT2::Face_circulator done = fc;
                if (fc != nullptr) {
                    do {
                        incident_faces.push_back(fc);
                    } while (++fc != done);
                }
                
                std::vector<std::size_t> incident_face_indices;
                for (auto f : incident_faces) {
                    if (!dt.is_infinite(f)) {
                        incident_face_indices.push_back(face_index_map.at(f));
                    }
                }
                data->vertices().emplace_back(it->point(), std::move(incident_face_indices));
            }
        }

        for (auto it = dt.finite_edges_begin(); it != dt.finite_edges_end(); ++it)
        {
            const auto vertex_index = [&](const int e) {
                return vertex_index_map.at(it->first->vertex(e));
            };
            // An edge in 2D is given by a face and the index of the vertex opposite to the edge.
            // Vertices of the edge are (cw(e)) and (ccw(e)).
            int i = it->second;
            int cw = dt.cw(i);
            int ccw = dt.ccw(i);
            data->edges().emplace_back(vertex_index_map.at(it->first->vertex(cw)),
                                       vertex_index_map.at(it->first->vertex(ccw)));
        }

        data->faces().reserve(dt.number_of_faces());
        for (auto it = dt.finite_faces_begin(); it != dt.finite_faces_end(); ++it)
        {
            const auto point_index = [&](const int i) {
                return vertex_index_map.at(it->vertex(i));
            };
            data->faces().push_back({point_index(0), point_index(1), point_index(2)});
        }

        return data;
    }
}
