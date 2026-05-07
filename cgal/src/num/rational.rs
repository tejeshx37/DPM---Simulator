use super::Integer;
use cxx::UniquePtr;
use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

pub struct Rational(UniquePtr<cgal_sys::Rational>);

impl From<UniquePtr<cgal_sys::Rational>> for Rational {
    fn from(value: UniquePtr<cgal_sys::Rational>) -> Self {
        Self(value)
    }
}

impl From<&cgal_sys::Rational> for Rational {
    fn from(value: &cgal_sys::Rational) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_rational(value).into()
    }
}

impl FromStr for Rational {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (num, den) = parser::parse(s)?;
        Self::new_fraction_integer(&num, &den)
    }
}

impl From<i32> for Rational {
    fn from(value: i32) -> Self {
        let _lock = crate::lock();
        cgal_sys::create_rational_from_i32(value, 1).into()
    }
}

impl TryFrom<f64> for Rational {
    type Error = String;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            Err(format!("{value} is not a finite floating point"))
        } else {
            let _lock = crate::lock();
            Ok(Rational::from(cgal_sys::create_rational_from_f64(value)))
        }
    }
}

impl Deref for Rational {
    type Target = cgal_sys::Rational;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Clone for Rational {
    fn clone(&self) -> Self {
        let _lock = crate::lock();
        Self::from(cgal_sys::clone_rational(self))
    }
}

impl Drop for Rational {
    fn drop(&mut self) {
        let _lock = crate::lock();
        let mut inner = cxx::UniquePtr::null();
        std::mem::swap(&mut self.0, &mut inner);
        drop(inner);
    }
}

impl PartialEq for Rational {
    fn eq(&self, other: &Self) -> bool {
        cgal_sys::rational_eq(self, other)
    }
}

impl Eq for Rational {}

unsafe impl Send for Rational {}
unsafe impl Sync for Rational {}

impl Debug for Rational {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Rational({self})"))
    }
}

impl Display for Rational {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _lock = crate::lock();
        f.write_str(&cgal_sys::rational_to_string(self).to_string())
    }
}

impl Default for Rational {
    fn default() -> Self {
        Self::from(0)
    }
}

impl Rational {
    pub fn new_fraction_i32(num: i32, den: i32) -> Result<Self, String> {
        if den == 0 {
            Err(String::from("Rationals cannot have 0 as a denominator"))
        } else {
            let _lock = crate::lock();
            Ok(cgal_sys::create_rational_from_i32(num, den).into())
        }
    }

    pub fn new_fraction_integer(num: &Integer, den: &Integer) -> Result<Self, String> {
        let _lock = crate::lock();
        if cgal_sys::is_zero(den) {
            Err(String::from("Rationals cannot have 0 as a denominator"))
        } else {
            Ok(cgal_sys::create_rational_from_integer(num, den).into())
        }
    }
}

mod parser {
    use super::Integer;
    use regex::Regex;

    fn is_integer(num: &str) -> bool {
        Regex::new(r"^[-]?[0-9]+$")
            .expect("Regex is valid")
            .is_match(num)
    }

    fn is_positive_integer(num: &str) -> bool {
        Regex::new(r"^[0-9]+$")
            .expect("Regex is valid")
            .is_match(num)
    }

    fn is_float(num: &str) -> bool {
        Regex::new(r"^[-]?[0-9]+\.[0-9]+$")
            .expect("Regex is valid")
            .is_match(num)
    }

    pub fn parse(input: &str) -> Result<(Integer, Integer), String> {
        // Numbers like 1/2
        if let Some((num, den)) = input.split_once('/') {
            return parse_fraction(num, den);
        }
        // Numbers like 1e-5, ignoring case of 'e'
        let opt = input.split_once('e').or_else(|| input.split_once('E'));
        if let Some((significand, exponent)) = opt {
            return parse_exp_float(significand, exponent);
        }
        // Regular decimal numbers, supported types : -0.5, 0.5
        if let Some((integral, fractional)) = input.split_once('.') {
            return parse_decimal(integral, fractional);
        }
        input
            .parse()
            .map(|num| (num, Integer::from(1)))
            .map_err(|err| {
                format!("Expected an integer, found {input}. Reason for parse failure : {err}")
            })
    }

    fn parse_fraction(num: &str, den: &str) -> Result<(Integer, Integer), String> {
        if !is_integer(num) {
            return Err(format!(
                "There are non digit characters in the numerator {num}"
            ));
        }
        let num = num
            .parse()
            .map_err(|err| format!("Failed to parse numerator {num}. Reason: {err}"))?;

        if !is_integer(den) {
            return Err(format!(
                "There are non digit characters in the denominator {den}"
            ));
        }
        let den: Integer = den
            .parse()
            .map_err(|err| format!("Failed to parse denominator {den}. Reason: {err}"))?;

        if cgal_sys::is_zero(&den) {
            Err(String::from(
                "0 cannot be the denominator of a rational number",
            ))
        } else {
            Ok((num, den))
        }
    }

    fn parse_exp_float(significand: &str, exponent: &str) -> Result<(Integer, Integer), String> {
        let exponent: i32 = exponent
            .parse()
            .map_err(|_| format!("Exponent {exponent} is not an integer"))?;

        if is_integer(significand) {
            let significand = significand.parse().map_err(|err| {
                format!("Failed to parse significand {significand}. Reason: {err}")
            })?;
            let multiplier = Integer::from(10).pow(exponent.unsigned_abs());
            Ok(if exponent.is_negative() {
                (significand, multiplier)
            } else {
                (significand * multiplier, Integer::from(1))
            })
        } else if is_float(significand) {
            let (integral, fractional) = significand
                .split_once('.')
                .expect("Floating point should be splittable with single .");
            let (num, den) = parse_decimal(integral, fractional)?;
            let multiplier = Integer::from(10).pow(exponent.unsigned_abs());
            Ok(if exponent.is_negative() {
                (num, den * multiplier)
            } else {
                (num * multiplier, den)
            })
        } else {
            Err(format!("Significand {significand} is not a number"))
        }
    }

    fn parse_decimal(integral: &str, fractional: &str) -> Result<(Integer, Integer), String> {
        if !is_integer(integral) {
            return Err(format!(
                "There are non digit characters in the integral part {integral}"
            ));
        }
        if !is_positive_integer(fractional) {
            return Err(format!(
            "There are non digit characters or a negative sign at the front in the fractional part {integral}"
        ));
        }

        let num = (integral.to_string() + fractional)
            .parse()
            .map_err(|err| format!("Failed to create numerator of the rational. Reason: {err}"))?;
        let decimals = u32::try_from(fractional.len())
            .map_err(|_| String::from("There are too many decimals"))?;
        let den = Integer::from(10).pow(decimals);

        Ok((num, den))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use test_case::test_case;

        #[test_case("0" => true)]
        #[test_case("-1" => true)]
        #[test_case("anahjdf" => false)]
        #[test_case("0ab0" => false)]
        #[test_case("0.0" => false)]
        #[test_case("0a.a0" => false)]
        #[test_case("-100" => true)]
        fn is_integer_works(input: &str) -> bool {
            super::is_integer(input)
        }

        #[test_case("0" => true)]
        #[test_case("-1" => false)]
        #[test_case("anahjdf" => false)]
        #[test_case("0ab0" => false)]
        #[test_case("0.0" => false)]
        #[test_case("0a.a0" => false)]
        #[test_case("-100" => false)]
        #[test_case("100" => true)]
        fn is_positive_integer_works(input: &str) -> bool {
            super::is_positive_integer(input)
        }

        #[test_case("0" => false)]
        #[test_case("-1" => false)]
        #[test_case("anahjdf" => false)]
        #[test_case("0ab0" => false)]
        #[test_case("0.0" => true)]
        #[test_case("-100.567" => true)]
        #[test_case("0a.a0" => false)]
        #[test_case("-100" => false)]
        fn is_float_works(input: &str) -> bool {
            super::is_float(input)
        }

        #[test_case("0/4" => Ok((Integer::from(0), Integer::from(4))))]
        #[test_case("1/4" => Ok((Integer::from(1), Integer::from(4))))]
        #[test_case("-1/5" => Ok((Integer::from(-1), Integer::from(5))))]
        #[test_case("1/-6" => Ok((Integer::from(1), Integer::from(-6))))]
        #[test_case("-1/-3" => Ok((Integer::from(-1), Integer::from(-3))))]
        #[test_case("0e4" => Ok((Integer::from(0), Integer::from(1))))]
        #[test_case("1e4" => Ok((Integer::from(10_000), Integer::from(1))))]
        #[test_case("-1e5" => Ok((Integer::from(-100_000), Integer::from(1))))]
        #[test_case("1e-6" => Ok((Integer::from(1), Integer::from(1000_000))))]
        #[test_case("-1e-3" => Ok((Integer::from(-1), Integer::from(1000))))]
        #[test_case("-1E-2" => Ok((Integer::from(-1), Integer::from(100))))]
        #[test_case("1.86e2" => Ok((Integer::from(18600), Integer::from(100))))]
        #[test_case("1.85e-2" => Ok((Integer::from(185), Integer::from(10_000))))]
        #[test_case("0.3" => Ok((Integer::from(3), Integer::from(10))))]
        #[test_case("-0.2" => Ok((Integer::from(-2), Integer::from(10))))]
        #[test_case("-1" => Ok((Integer::from(-1), Integer::from(1))))]
        #[test_case("2" => Ok((Integer::from(2), Integer::from(1))))]
        fn parse_works(input: &str) -> Result<(Integer, Integer), String> {
            parse(input)
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::Rational;
    use serde::{de, Deserialize, Serialize};

    impl Serialize for Rational {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Rational {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            String::deserialize(deserializer).and_then(|str| {
                str.parse::<Self>().map_err(|err| {
                    de::Error::custom(format!("Failed to parse rational. Reason: {err}"))
                })
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Rational;
    use test_case::test_case;

    impl Rational {
        pub fn new_fraction_unwrapped(num: i32, den: i32) -> Self {
            Self::new_fraction_i32(num, den).expect("Denominator should not be zero")
        }
    }

    #[test_case("0/4" => Rational::new_fraction_i32(0, 4))]
    #[test_case("1/4" => Rational::new_fraction_i32(1, 4))]
    #[test_case("2/4" => Rational::new_fraction_i32(1, 2))]
    #[test_case("-1/5" => Rational::new_fraction_i32(-1, 5))]
    #[test_case("1/-6" => Rational::new_fraction_i32(-1, 6))]
    #[test_case("-1/-3" => Rational::new_fraction_i32(1, 3))]
    #[test_case("0e4" => Rational::new_fraction_i32(0, 1))]
    #[test_case("1e4" => Rational::new_fraction_i32(10_000, 1))]
    #[test_case("-1e5" => Rational::new_fraction_i32(-100_000, 1))]
    #[test_case("1e-6" => Rational::new_fraction_i32(1, 1000_000))]
    #[test_case("-1e-3" => Rational::new_fraction_i32(-1, 1000))]
    #[test_case("-1E-2" => Rational::new_fraction_i32(-1, 100))]
    #[test_case("1.86e2" => Rational::new_fraction_i32(186, 1))]
    #[test_case("1.85e-2" => Rational::new_fraction_i32(185, 10_000))]
    #[test_case("0.3" => Rational::new_fraction_i32(3, 10))]
    #[test_case("-0.2" => Rational::new_fraction_i32(-1, 5))]
    #[test_case("-1" => Rational::new_fraction_i32(-1, 1))]
    #[test_case("2" => Rational::new_fraction_i32(2, 1))]
    fn from_str_works(input: &str) -> Result<Rational, String> {
        input.parse()
    }
}
