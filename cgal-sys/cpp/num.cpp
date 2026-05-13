#include "num.h"
#include <iostream>
#include <sstream>
#include <vector>
#include <cmath>

template <>
std::unique_ptr<Algebraic> abs(const Algebraic &value)
{
    return std::make_unique<Algebraic>(value.abs());
}

template <>
std::unique_ptr<std::string> to_string(const Algebraic &value)
{
    return std::make_unique<std::string>(value.toString());
}

std::unique_ptr<Rational> create_rational(const std::int32_t num, const std::int32_t den)
{
    return std::make_unique<Rational>(num, den);
}

std::unique_ptr<Rational> create_rational(const Integer &num, const Integer &den)
{
    return std::make_unique<Rational>(num, den);
}

std::unique_ptr<Rational> create_rational(const double value)
{
    return std::make_unique<Rational>(value);
}

std::unique_ptr<Rational> create_rational(const Rational &value)
{
    return std::make_unique<Rational>(value);
}

double rational_to_double(const Rational &value)
{
    return CGAL::to_double(value);
}


template <>
std::unique_ptr<std::string> to_string(const Rational &value)
{
    std::stringstream ss;
    ss << value;
    return std::make_unique<std::string>(ss.str());
}

std::unique_ptr<Integer> create_integer(const Integer &value)
{
    return std::make_unique<Integer>(value);
}

std::unique_ptr<Integer> pow_integer(const Integer &base, const std::uint32_t exp)
{
    Integer result;
    mpz_pow_ui(result.get_mp(), base.get_mp(), exp);
    return std::make_unique<Integer>(std::move(result));
}

template <>
std::unique_ptr<Integer> abs(const Integer &value)
{
    const Integer abs = CGAL::is_negative(value) ? -value : value;
    return std::make_unique<Integer>(std::move(abs));
}

template <>
std::unique_ptr<std::string> to_string(const Integer &value)
{
    std::stringstream ss;
    ss << value;
    return std::make_unique<std::string>(ss.str());
}