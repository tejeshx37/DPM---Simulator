#pragma once
#include <utility>

template <typename F, typename S>
F first(const std::pair<F, S> &pair)
{
    return pair.first;
}

template <typename F, typename S>
const F &first_ref(const std::pair<F, S> &pair)
{
    return pair.first;
}

template <typename F, typename S>
S second(const std::pair<F, S> &pair)
{
    return pair.second;
}

template <typename F, typename S>
const S &second_ref(const std::pair<F, S> &pair)
{
    return pair.second;
}