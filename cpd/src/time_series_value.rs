use derive_getters::Getters;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Getters)]
pub struct TimeStampedValue<T> {
    pub(crate) time_stamp: f32,
    pub(crate) value: T,
}

impl<T: Default> TimeStampedValue<T> {
    pub(crate) fn default() -> Self {
        Self {
            time_stamp: 0.0,
            value: T::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TimeSeriesValue<T> {
    Single(T),
    Series(Vec<TimeStampedValue<T>>),
}

impl<T: Default> TimeSeriesValue<T> {
    pub fn single_default() -> Self {
        Self::Single(T::default())
    }

    pub fn series_default() -> Self {
        Self::Series(vec![TimeStampedValue::default()])
    }

    pub(crate) fn default_first(&mut self) {
        match self {
            TimeSeriesValue::Single(v) => *v = T::default(),
            TimeSeriesValue::Series(series) => {
                series.clear();
                series.push(TimeStampedValue::default());
            }
        }
    }
}

impl<T> TimeSeriesValue<T> {
    pub fn latest(&self) -> &T {
        match self {
            Self::Single(value) => value,
            Self::Series(series) => {
                &series
                    .last()
                    .expect("Time series should not be empty")
                    .value
            }
        }
    }

    pub fn as_series(&self) -> Option<&[TimeStampedValue<T>]> {
        match self {
            Self::Single(_) => None,
            Self::Series(series) => Some(series),
        }
    }

    pub(crate) fn set_or_push(&mut self, time_stamp: f32, value: T) {
        match self {
            Self::Single(v) => *v = value,
            Self::Series(series) => series.push(TimeStampedValue { time_stamp, value }),
        }
    }
}

mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl<T: Serialize> Serialize for TimeSeriesValue<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            T::serialize(self.latest(), serializer)
        }
    }

    impl<'de, T: Deserialize<'de>> Deserialize<'de> for TimeSeriesValue<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            T::deserialize(deserializer).map(Self::Single)
        }
    }
}
