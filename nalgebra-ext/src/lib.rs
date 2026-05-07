pub mod matrix2 {
    use enum_map::Enum;
    use nalgebra::{indexing::MatrixIndex, RawStorage, U2};
    use strum::{AsRefStr, Display, EnumIter};

    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(
        Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Enum, EnumIter, Display, AsRefStr,
    )]
    pub enum Component {
        #[default]
        XX,
        XY,
        YX,
        YY,
    }

    impl<'a, T: 'a, S> MatrixIndex<'a, T, U2, U2, S> for Component
    where
        S: RawStorage<T, U2, U2>,
    {
        type Output = &'a T;

        fn contained_by(&self, _matrix: &nalgebra::Matrix<T, U2, U2, S>) -> bool {
            true
        }

        unsafe fn get_unchecked(self, matrix: &'a nalgebra::Matrix<T, U2, U2, S>) -> Self::Output {
            let (irow, icol) = match self {
                Component::XX => (0, 0),
                Component::XY => (0, 1),
                Component::YX => (1, 0),
                Component::YY => (1, 1),
            };
            matrix.data.get_unchecked(irow, icol)
        }
    }
}

pub mod matrix3 {
    use enum_map::Enum;
    use nalgebra::{indexing::MatrixIndex, RawStorage, U3};
    use strum::{AsRefStr, Display, EnumIter};

    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(
        Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Enum, EnumIter, Display, AsRefStr,
    )]
    pub enum Component {
        #[default]
        XX,
        XY,
        XZ,
        YX,
        YY,
        YZ,
        ZX,
        ZY,
        ZZ,
    }

    impl<'a, T: 'a, S> MatrixIndex<'a, T, U3, U3, S> for Component
    where
        S: RawStorage<T, U3, U3>,
    {
        type Output = &'a T;

        fn contained_by(&self, _matrix: &nalgebra::Matrix<T, U3, U3, S>) -> bool {
            true
        }

        unsafe fn get_unchecked(self, matrix: &'a nalgebra::Matrix<T, U3, U3, S>) -> Self::Output {
            let (irow, icol) = match self {
                Component::XX => (0, 0),
                Component::XY => (0, 1),
                Component::XZ => (0, 2),
                Component::YX => (1, 0),
                Component::YY => (1, 1),
                Component::YZ => (1, 2),
                Component::ZX => (2, 0),
                Component::ZY => (2, 1),
                Component::ZZ => (2, 2),
            };
            matrix.data.get_unchecked(irow, icol)
        }
    }
}

