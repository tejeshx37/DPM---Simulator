#pragma once
#include <memory>
#include <utility>

template <typename F, typename S>
F first(const std::pair<F, S> &pair)
{
    return pair.first;
}

template <typename F, typename S>
std::unique_ptr<F> first(const std::pair<F, S> &pair)
{
    return std::make_unique<F>(pair.first);
}

template <typename F, typename S>
const F &first(const std::pair<F, S> &pair)
{
    return pair.first;
}

template <typename F, typename S>
S second(const std::pair<F, S> &pair)
{
    return pair.second;
}

template <typename F, typename S>
std::unique_ptr<S> second(const std::pair<F, S> &pair)
{
    return std::make_unique<S>(pair.second);
}

template <typename F, typename S>
const S &second(const std::pair<F, S> &pair)
{
    return pair.second;
}