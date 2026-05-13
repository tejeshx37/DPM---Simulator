use super::Rational;
use cgal_sys::ComparisonResult;
use cxx::UniquePtr;
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    ops::{Add, Deref, Div, Mul, Neg, Sub},
    str::FromStr,
};

pub struct Algebraic(UniquePtr<cgal_sys::Algebraic>);

impl From<UniquePtr<cgal_sys::Algebraic>> for Algebraic {
    fn from(value: UniquePtr<cgal_sys::Algebraic>) -> Self {
        Self(value)
    }
}

impl From<&cgal_sys::Algebraic> for Algebraic {
    fn from(value: &cgal_sys::Algebraic) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_algebraic(value).into()
    }
}

impl FromStr for Algebraic {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _lock = crate::lock();
        cxx::let_cxx_string!(c_str = s);
        cgal_sys::algebraic_from_string(&c_str)
            .map(Into::into)
            .map_err(|err| err.to_string())
    }
}

impl TryFrom<f64> for Algebraic {
    type Error = String;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            Err(format!("{value} is not a finite floating point"))
        } else {
            let _lock = crate::lock();
            Ok(Algebraic::from(cgal_sys::create_algebraic_from_f64(value)))
        }
    }
}

impl From<Rational> for Algebraic {
    fn from(value: Rational) -> Self {
        Algebraic::from(&value)
    }
}

impl From<&Rational> for Algebraic {
    fn from(value: &Rational) -> Self {
        let _lock = crate::lock();
        Self::from(cgal_sys::create_algebraic_from_rational(value))
    }
}

impl Clone for Algebraic {
    fn clone(&self) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_algebraic(self).into()
    }
}

impl Deref for Algebraic {
    type Target = cgal_sys::Algebraic;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl From<Algebraic> for f64 {
    fn from(value: Algebraic) -> Self {
        (&value).into()
    }
}

impl From<&Algebraic> for f64 {
    fn from(value: &Algebraic) -> Self {
        let _lock = crate::lock();
        value.double_value()
    }
}

impl From<i32> for Algebraic {
    fn from(value: i32) -> Self {
        let _lock = crate::lock();
        cgal_sys::create_algebraic_from_i32(value).into()
    }
}

impl From<u32> for Algebraic {
    fn from(value: u32) -> Self {
        let _lock = crate::lock();
        cgal_sys::create_algebraic_from_u32(value).into()
    }
}

impl PartialEq for Algebraic {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Algebraic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Algebraic {}

impl Ord for Algebraic {
    fn cmp(&self, other: &Self) -> Ordering {
        let _lock = crate::lock();
        match cgal_sys::compare_algebraic(self, other) {
            ComparisonResult::SMALLER => Ordering::Less,
            ComparisonResult::EQUAL => Ordering::Equal,
            ComparisonResult::LARGER => Ordering::Greater,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test_impls {
    use super::Algebraic;
    use approx::AbsDiffEq;

    impl AbsDiffEq for Algebraic {
        type Epsilon = Self;

        fn default_epsilon() -> Self::Epsilon {
            Algebraic::try_from(1e-14).expect("Epsilon is valid fp")
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            (self - other).abs() <= epsilon
        }
    }
}

impl<'a> Add<&'a Algebraic> for &'a Algebraic {
    type Output = Algebraic;

    fn add(self, rhs: Self) -> Self::Output {
        let _lock = crate::lock();
        cgal_sys::add_algebraic(self, rhs).into()
    }
}

impl<T> Add<T> for Algebraic
where
    T: Into<Algebraic>,
{
    type Output = Algebraic;

    fn add(self, rhs: T) -> Self::Output {
        let _lock = crate::lock();
        cgal_sys::add_algebraic(&self, &rhs.into()).into()
    }
}

impl<'a> Sub<&'a Algebraic> for &'a Algebraic {
    type Output = Algebraic;

    fn sub(self, rhs: Self) -> Self::Output {
        let _lock = crate::lock();
        cgal_sys::sub_algebraic(self, rhs).into()
    }
}

impl<T> Sub<T> for Algebraic
where
    T: Into<Algebraic>,
{
    type Output = Algebraic;

    fn sub(self, rhs: T) -> Self::Output {
        let _lock = crate::lock();
        cgal_sys::sub_algebraic(&self, &rhs.into()).into()
    }
}

impl<'a> Mul<&'a Algebraic> for &'a Algebraic {
    type Output = Algebraic;

    fn mul(self, rhs: Self) -> Self::Output {
        let _lock = crate::lock();
        cgal_sys::mul_algebraic(self, rhs).into()
    }
}

impl<T> Mul<T> for Algebraic
where
    T: Into<Algebraic>,
{
    type Output = Algebraic;

    fn mul(self, rhs: T) -> Self::Output {
        let _lock = crate::lock();
        cgal_sys::mul_algebraic(&self, &rhs.into()).into()
    }
}

impl<'a> Div<&'a Algebraic> for &'a Algebraic {
    type Output = Algebraic;

    fn div(self, rhs: Self) -> Self::Output {
        let _lock = crate::lock();
        cgal_sys::div_algebraic(self, rhs).into()
    }
}

impl<T> Div<T> for Algebraic
where
    T: Into<Algebraic>,
{
    type Output = Algebraic;

    fn div(self, rhs: T) -> Self::Output {
        let _lock = crate::lock();
        cgal_sys::div_algebraic(&self, &rhs.into()).into()
    }
}

impl Neg for Algebraic {
    type Output = Algebraic;

    fn neg(self) -> Self::Output {
        let _lock = crate::lock();
        cgal_sys::neg_algebraic(&self).into()
    }
}

impl Debug for Algebraic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Algebraic").field(&self.to_string()).finish()
    }
}

impl Display for Algebraic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&cgal_sys::algebraic_to_string(self).to_string())
    }
}

impl Algebraic {
    pub fn abs(&self) -> Self {
        let _lock = crate::lock();
        cgal_sys::abs_algebraic(self).into()
    }
}

impl Drop for Algebraic {
    fn drop(&mut self) {
        let _lock = crate::lock();
        let mut inner = UniquePtr::null();
        std::mem::swap(&mut self.0, &mut inner);
        drop(inner);
    }
}

#[cfg(test)]
mod tests {
    use crate::num::Algebraic;

    #[test]
    fn abs_works() {
        assert_eq!(Algebraic::from(-1).abs(), Algebraic::from(1));
    }
}
