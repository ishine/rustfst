use std::cell::RefCell;

use anyhow::{Context, Result};

use crate::algorithms::encode::{EncodeTable, EncodeTableMut};
use crate::algorithms::{FinalTr, MapFinalAction, TrMapper};
use crate::fst_traits::MutableFst;
use crate::{Semiring, Tr};

struct EncodeMapper<W: Semiring> {
    encode_table: EncodeTable<W>,
}

impl<W: Semiring> EncodeMapper<W> {
    pub fn new(encode_labels: bool, encode_weights: bool) -> Self {
        EncodeMapper {
            encode_table: EncodeTable(RefCell::new(EncodeTableMut::new(
                encode_labels,
                encode_weights,
            ))),
        }
    }
}

impl<W: Semiring> TrMapper<W> for EncodeMapper<W> {
    fn tr_map(&self, tr: &mut Tr<W>) -> Result<()> {
        let tuple = self.encode_table.0.borrow().tr_to_tuple(tr);
        let label = self.encode_table.0.borrow_mut().encode(tuple);
        tr.ilabel = label;
        if self.encode_table.0.borrow().encode_labels {
            tr.olabel = label;
        }
        if self.encode_table.0.borrow().encode_weights {
            tr.weight.set_value(W::one().take_value());
        }
        Ok(())
    }

    fn final_tr_map(&self, final_tr: &mut FinalTr<W>) -> Result<()> {
        if self.encode_table.0.borrow().encode_weights {
            let tuple = self.encode_table.0.borrow().final_tr_to_tuple(final_tr);
            let label = self.encode_table.0.borrow_mut().encode(tuple);
            final_tr.ilabel = label;
            if self.encode_table.0.borrow().encode_labels {
                final_tr.olabel = label;
            }
            if self.encode_table.0.borrow().encode_weights {
                final_tr.weight.set_value(W::one().take_value());
            }
        }
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        if self.encode_table.0.borrow().encode_weights {
            MapFinalAction::MapRequireSuperfinal
        } else {
            MapFinalAction::MapNoSuperfinal
        }
    }
}

/// The `encode` operation allows the representation of a weighted transducer as a weighted automaton,
/// an unweighted transducer or an unweighted automaton by considering the pair
/// (input label, output), the pair (input label, weight) or the triple (input label,
/// output label, weight) as a single label depending on the value
/// of the encode flags: `encode_labels` and `encode_weights`.
///
/// The encoding of each pair or triple of labels and/or weights as a unique key is stored
/// in an `EncodeTable` object.
pub fn encode<W, F>(
    fst: &mut F,
    encode_labels: bool,
    encode_weights: bool,
) -> Result<EncodeTable<W>>
where
    W: Semiring,
    F: MutableFst<W>,
{
    let mut encode_mapper = EncodeMapper::new(encode_labels, encode_weights);
    fst.tr_map(&mut encode_mapper)
        .with_context(|| format_err!("Error calling TrMap with EncodeMapper."))?;
    Ok(encode_mapper.encode_table)
}
