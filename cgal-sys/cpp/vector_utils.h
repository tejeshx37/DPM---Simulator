#include <memory>
#include <vector>

template <typename T>
std::unique_ptr<std::vector<T>> create_vector()
{
    return std::make_unique<std::vector<T>>();
}

template <typename T>
std::unique_ptr<std::vector<T>> create_vector(const std::size_t capacity)
{
    auto vec = std::make_unique<std::vector<T>>();
    vec->reserve(capacity);
    return vec;
}

template <typename T>
void push_back(std::vector<T> &vec, std::unique_ptr<T> value)
{
    vec.push_back(std::move(*value.release()));
}

template <typename T>
void push_back(std::vector<T> &vec, const T &value)
{
    vec.push_back(value);
}
