use anyhow::Context;
use anyhow::Result;

use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::macros::test_eq_fst;
use crate::tests_openfst::FstTestData;

pub fn test_const_fst_bin_deserializer<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let parsed_fst_bin = ConstFst::<W>::read(&test_data.raw_const_bin_path)
        .with_context(|| format_err!("Failed parsing ConstFst Aligned"))?;
    let raw_const: ConstFst<_> = test_data.raw.clone().into();

    test_eq_fst(&raw_const, &parsed_fst_bin, "Deserializer ConstFst Bin");
    Ok(())
}

pub fn test_const_fst_aligned_bin_deserializer<W>(
    test_data: &FstTestData<W, VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let parsed_fst_bin = ConstFst::<W>::read(&test_data.raw_const_aligned_bin_path)
        .with_context(|| format_err!("Failed parsing ConstFst Aligned Bin"))?;
    let raw_const: ConstFst<_> = test_data.raw.clone().into();

    test_eq_fst(
        &raw_const,
        &parsed_fst_bin,
        "Deserializer ConstFst Aligned Bin",
    );

    Ok(())
}
