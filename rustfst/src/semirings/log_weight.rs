use std::borrow::Borrow;
use std::f32;
use std::hash::{Hash, Hasher};

use anyhow::Result;

use ordered_float::OrderedFloat;

use crate::parsers::bin_fst::utils_serialization::write_bin_f32;
use crate::semirings::{
    CompleteSemiring, DivideType, ReverseBack, Semiring, SemiringProperties, SerializableSemiring,
    StarSemiring, WeaklyDivisibleSemiring, WeightQuantize,
};
use nom::number::complete::{float, le_f32};
use nom::IResult;
use std::io::Write;

/// Log semiring: (log(e^-x + e^-y), +, inf, 0).
#[derive(Clone, Debug, PartialOrd, Default, Copy, Eq)]
pub struct LogWeight {
    value: OrderedFloat<f32>,
}

fn ln_pos_exp(x: f32) -> f32 {
    ((-x).exp()).ln_1p()
}

impl Semiring for LogWeight {
    type Type = f32;
    type ReverseWeight = LogWeight;

    fn zero() -> Self {
        Self {
            value: OrderedFloat(f32::INFINITY),
        }
    }
    fn one() -> Self {
        Self {
            value: OrderedFloat(0.0),
        }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        LogWeight {
            value: OrderedFloat(value),
        }
    }

    fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        let f1 = self.value();
        let f2 = rhs.borrow().value();
        self.value.0 = if f1.eq(&f32::INFINITY) {
            *f2
        } else if f2.eq(&f32::INFINITY) {
            *f1
        } else if f1 > f2 {
            f2 - ln_pos_exp(f1 - f2)
        } else {
            f1 - ln_pos_exp(f2 - f1)
        };
        Ok(())
    }

    fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        let f1 = self.value();
        let f2 = rhs.borrow().value();
        if f1.eq(&f32::INFINITY) {
        } else if f2.eq(&f32::INFINITY) {
            self.value.0 = *f2;
        } else {
            self.value.0 += f2;
        }
        Ok(())
    }

    fn value(&self) -> &Self::Type {
        self.value.as_ref()
    }

    fn take_value(self) -> Self::Type {
        self.value.into_inner()
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value.0 = value
    }

    fn reverse(&self) -> Result<Self::ReverseWeight> {
        Ok(*self)
    }

    fn properties() -> SemiringProperties {
        SemiringProperties::LEFT_SEMIRING
            | SemiringProperties::RIGHT_SEMIRING
            | SemiringProperties::COMMUTATIVE
    }
}

impl ReverseBack<LogWeight> for LogWeight {
    fn reverse_back(&self) -> Result<LogWeight> {
        Ok(self.clone())
    }
}

impl AsRef<LogWeight> for LogWeight {
    fn as_ref(&self) -> &LogWeight {
        &self
    }
}

display_semiring!(LogWeight);

impl CompleteSemiring for LogWeight {}

impl StarSemiring for LogWeight {
    fn closure(&self) -> Self {
        if self.value.0 >= 0.0 && self.value.0 < 1.0 {
            Self::new((1.0 - self.value.0).ln())
        } else {
            Self::new(f32::NEG_INFINITY)
        }
    }
}

impl WeaklyDivisibleSemiring for LogWeight {
    fn divide_assign(&mut self, rhs: &Self, _divide_type: DivideType) -> Result<()> {
        self.value.0 -= rhs.value.0;
        Ok(())
    }
}

impl_quantize_f32!(LogWeight);

partial_eq_and_hash_f32!(LogWeight);

impl SerializableSemiring for LogWeight {
    fn weight_type() -> String {
        "log".to_string()
    }

    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, weight) = le_f32(i)?;
        Ok((i, Self::new(weight)))
    }

    fn write_binary<F: Write>(&self, file: &mut F) -> Result<()> {
        write_bin_f32(file, *self.value())
    }

    fn parse_text(i: &str) -> IResult<&str, Self> {
        let (i, f) = float(i)?;
        Ok((i, Self::new(f)))
    }
}

test_semiring_serializable!(
    tests_log_weight_serializable,
    LogWeight,
    LogWeight::new(0.3) LogWeight::new(0.5) LogWeight::new(0.0) LogWeight::new(-1.2)
);

impl Into<LogWeight> for f32 {
    fn into(self) -> LogWeight {
        LogWeight::new(self)
    }
}
