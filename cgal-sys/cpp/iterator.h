#pragma once

#include <memory>

template <typename T>
class Iterator
{
public:
    Iterator(const T iter, const T end) : iter(std::move(iter)),
                                          end(std::move(end)){};

    bool has_next() const noexcept
    {
        return iter != end;
    };

    const typename T::value_type &next()
    {
        return *(iter++);
    };

private:
    T iter;
    const T end;
};