#include "polygon_set.h"

std::unique_ptr<PolygonSet> create_polygon_set()
{
    return std::make_unique<PolygonSet>();
}

std::unique_ptr<PolygonSet> create_polygon_set(const PolygonSet &polygon_set)
{
    return std::make_unique<PolygonSet>(polygon_set);
}

void split_curve(PolygonSet &polygon_set,
                 const XMonotoneCurve &ref_curve,
                 const Point &point)
{
    using Arrangement = PolygonSet::Arrangement_2;
    using FaceHandle = Arrangement::Face_handle;
    using CCBHalfEdgeCirculator = Arrangement::Ccb_halfedge_circulator;
    using HoleIterator = Arrangement::Hole_iterator;

    CGAL::IO::set_pretty_mode(std::cout);
    Arrangement &arr = polygon_set.arrangement();
    const ConicTraits traits;
    const ConicTraits::Equal_2 equals = traits.equal_2_object();
    auto traverse_and_split = [&](const CCBHalfEdgeCirculator ctr)
    {
        CCBHalfEdgeCirculator handle = ctr;
        do
        {
            if (!equals(handle->curve(), ref_curve))
            {
                continue;
            }
            XMonotoneCurve c1, c2;
            traits.split_2_object()(handle->curve(), point, c1, c2);
            arr.split_edge(handle, c1, c2);
            return true;
        } while (++handle != ctr);
        return false;
    };
    for (const FaceHandle face : arr.face_handles())
    {
        if (face->has_outer_ccb() && traverse_and_split(face->outer_ccb()))
        {
            return;
        }
        for (HoleIterator it = face->holes_begin(); it != face->holes_end(); it++)
        {
            if (traverse_and_split(*it))
            {
                return;
            }
        }
    }
    std::ostringstream msg_stream;
    CGAL::IO::set_pretty_mode(msg_stream);
    msg_stream << "Could not find any curves containing " << point;
    throw std::runtime_error(msg_stream.str());
}

std::unique_ptr<std::vector<PolygonWithHoles>> polygon_with_holes(const PolygonSet &polygon_set)
{
    auto polygons = std::make_unique<std::vector<PolygonWithHoles>>();
    polygon_set.polygons_with_holes(std::back_inserter(*polygons));
    return polygons;
}