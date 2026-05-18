#pragma once

#include <memory>
#include <string>
#include <cstdint>

#include <CGAL/CORE_algebraic_number_traits.h>

typedef CGAL::CORE_algebraic_number_traits CORE_ANT;
typedef CORE_ANT::Rational Rational;
typedef CORE_ANT::Algebraic Algebraic;
typedef CORE_ANT::Integer Integer;

template <typename T>
std::unique_ptr<Algebraic> create_algebraic(const T &value)
{
    return std::make_unique<Algebraic>(value);
}

// Explicit by-value overloads needed for CXX bridge function pointer matching
inline std::unique_ptr<Algebraic> create_algebraic(const std::int32_t value) { return std::make_unique<Algebraic>(value); }
inline std::unique_ptr<Algebraic> create_algebraic(const std::uint32_t value) { return std::make_unique<Algebraic>(value); }
inline std::unique_ptr<Algebraic> create_algebraic(const double value) { return std::make_unique<Algebraic>(value); }

std::unique_ptr<Rational> create_rational(const std::int32_t num, const std::int32_t den);
std::unique_ptr<Rational> create_rational(const Integer &num, const Integer &den);
std::unique_ptr<Rational> create_rational(const double value);
std::unique_ptr<Rational> create_rational(const Rational &value);
double rational_to_double(const Rational &value);

template <typename T>
std::unique_ptr<Integer> create_integer(const T value)
{
    return std::make_unique<Integer>(value);
}
std::unique_ptr<Integer> create_integer(const Integer &value);
std::unique_ptr<Integer> pow_integer(const Integer &base, const std::uint32_t exp);

template <typename T>
bool equals(const T &a, const T &b)
{
    return a == b;
}

template <typename T>
std::unique_ptr<T> abs(const T &value);

template <typename T>
std::unique_ptr<T> from_string(const std::string &str)
{
    return std::make_unique<T>(str);
}

template <typename T>
std::unique_ptr<std::string> to_string(const T &value);

template <typename T>
std::unique_ptr<T> add(const T &lhs, const T &rhs)
{
    return std::make_unique<T>(lhs + rhs);
}

template <typename T>
std::unique_ptr<T> sub(const T &lhs, const T &rhs)
{
    return std::make_unique<T>(lhs - rhs);
}

template <typename T>
std::unique_ptr<T> mul(const T &lhs, const T &rhs)
{
    return std::make_unique<T>(lhs * rhs);
}

template <typename T>
std::unique_ptr<T> div(const T &lhs, const T &rhs)
{
    return std::make_unique<T>(lhs / rhs);
}

template <typename T>
std::unique_ptr<T> neg(const T &value)
{
    return std::make_unique<T>(-value);
}