use cxx::UniquePtr;
use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::{Deref, Mul},
    str::FromStr,
};

pub struct Integer(UniquePtr<cgal_sys::Integer>);

impl From<UniquePtr<cgal_sys::Integer>> for Integer {
    fn from(value: UniquePtr<cgal_sys::Integer>) -> Self {
        Self(value)
    }
}

impl From<&cgal_sys::Integer> for Integer {
    fn from(value: &cgal_sys::Integer) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_integer(value).into()
    }
}

impl FromStr for Integer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _lock = crate::lock();
        cxx::let_cxx_string!(cxx_str = s);
        cgal_sys::integer_from_string(&cxx_str)
            .map(Into::into)
            .map_err(|err| err.to_string())
    }
}

impl From<i32> for Integer {
    fn from(value: i32) -> Self {
        let _lock = crate::lock();
        cgal_sys::create_integer_from_i32(value).into()
    }
}

impl From<u32> for Integer {
    fn from(value: u32) -> Self {
        let _lock = crate::lock();
        cgal_sys::create_integer_from_u32(value).into()
    }
}

impl Deref for Integer {
    type Target = cgal_sys::Integer;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl PartialEq for Integer {
    fn eq(&self, other: &Self) -> bool {
        let _lock = crate::lock();
        cgal_sys::integer_eq(self, other)
    }
}

impl Eq for Integer {}

impl Clone for Integer {
    fn clone(&self) -> Self {
        let _lock = crate::lock();
        Self::from(cgal_sys::clone_integer(self))
    }
}

impl Drop for Integer {
    fn drop(&mut self) {
        let _lock = crate::lock();
        let mut inner = cxx::UniquePtr::null();
        std::mem::swap(&mut self.0, &mut inner);
        drop(inner);
    }
}

impl Debug for Integer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Integer({self})"))
    }
}

impl Display for Integer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _lock = crate::lock();
        f.write_str(&cgal_sys::integer_to_string(self).to_string())
    }
}

unsafe impl Send for Integer {}
unsafe impl Sync for Integer {}

impl Default for Integer {
    fn default() -> Self {
        Self::from(0)
    }
}

impl Mul for Integer {
    type Output = Integer;

    fn mul(self, rhs: Self) -> Self::Output {
        let _lock = crate::lock();
        Self::from(cgal_sys::mul_integer(&self, &rhs))
    }
}

impl Integer {
    pub fn pow(&self, exp: u32) -> Self {
        let _lock = crate::lock();
        Self::from(cgal_sys::pow_integer(self, exp))
    }

    pub fn abs(&self) -> Self {
        let _lock = crate::lock();
        Self::from(cgal_sys::abs_integer(self))
    }
}

#[cfg(test)]
mod tests {
    use super::Integer;
    use test_case::test_case;

    #[test_case(0, 10 => Integer::from(0))]
    #[test_case(2, 0 => Integer::from(1))]
    #[test_case(2, 1 => Integer::from(2))]
    #[test_case(2, 4 => Integer::from(16))]
    #[test_case(-3, 0 => Integer::from(1))]
    #[test_case(-3, 1 => Integer::from(-3))]
    #[test_case(-3, 2 => Integer::from(9))]
    fn pow_works(base: i32, exp: u32) -> Integer {
        Integer::from(base).pow(exp)
    }

    #[test_case(0 => Integer::from(0))]
    #[test_case(-1 => Integer::from(1))]
    #[test_case(2 => Integer::from(2))]
    fn abs_works(num: i32) -> Integer {
        Integer::from(num).abs()
    }
}
