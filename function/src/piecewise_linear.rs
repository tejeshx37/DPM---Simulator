use derive_getters::Getters;
use std::sync::OnceLock;
use typed_builder::TypedBuilder;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone, Copy, Getters, TypedBuilder)]
pub struct Piece {
    end_value: f32,
    width: f32,
}

impl Piece {
    pub fn scale_amplitude(self, scale: f32) -> Self {
        Self {
            end_value: self.end_value * scale,
            ..self
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone, Getters)]
pub struct PiecewiseLinear {
    pieces: Vec<Piece>,
    #[getter(skip)]
    #[cfg_attr(feature = "serde", serde(skip))]
    total_width: OnceLock<f32>,
}

impl PiecewiseLinear {
    pub fn scale_amplitude(self, scale: f32) -> Self {
        Self {
            pieces: self
                .pieces
                .into_iter()
                .map(|piece| piece.scale_amplitude(scale))
                .collect(),
            ..self
        }
    }

    pub fn of(&self, x: f32) -> Option<f32> {
        if self.pieces.is_empty() || x < 0.0 {
            return None;
        }
        let total_width = self
            .total_width
            .get_or_init(|| self.pieces.iter().map(|piece| piece.width).sum());
        if x > *total_width {
            return None;
        }
        let mut total_width = 0.0;
        let mut start_value = 0.0;
        let mut piece_index = 0;
        for (i, piece) in self.pieces.iter().enumerate() {
            if total_width + piece.width > x {
                piece_index = i;
                break;
            } else {
                total_width += piece.width;
                start_value = piece.end_value;
            }
        }
        let delta_x = x - total_width;
        let piece = &self.pieces[piece_index];
        let slope = (piece.end_value - start_value) / piece.width;
        Some(start_value + slope * delta_x)
    }
}

mod builder {
    use super::*;

    #[derive(Debug, Default, Clone)]
    pub struct PiecewiseLinearBuilder {
        pieces: Vec<Piece>,
    }

    impl PiecewiseLinearBuilder {
        pub fn piece(mut self, piece: Piece) -> PiecewiseLinearBuilder {
            self.pieces.push(piece);
            self
        }

        pub fn build(self) -> PiecewiseLinear {
            PiecewiseLinear {
                pieces: self.pieces,
                total_width: OnceLock::default(),
            }
        }
    }

    impl PiecewiseLinear {
        pub fn builder() -> PiecewiseLinearBuilder {
            PiecewiseLinearBuilder::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Piece, PiecewiseLinear};

    #[test]
    fn ramp_evaluates_correctly() {
        let f = PiecewiseLinear::builder()
            .piece(Piece::builder().end_value(1.0).width(1.0).build())
            .piece(Piece::builder().end_value(4.0).width(1.0).build())
            .build();
        assert_eq!(f.of(-0.1), None);
        assert_eq!(f.of(0.0), Some(0.0));
        assert_eq!(f.of(0.5), Some(0.5));
        assert_eq!(f.of(1.0), Some(1.0));
        assert_eq!(f.of(1.5), Some(2.5));
        assert_eq!(f.of(2.0), Some(4.0));
        assert_eq!(f.of(2.1), None);
    }

    #[test]
    fn empty_function_always_returns_none() {
        let f = PiecewiseLinear::builder().build();
        assert_eq!(f.of(0.0), None);
        assert_eq!(f.of(1.0), None);
    }
}
