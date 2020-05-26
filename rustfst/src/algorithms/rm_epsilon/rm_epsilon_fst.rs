// /// Removes epsilon-transitions (when both the input and output label are an
// /// epsilon) from a transducer. The result will be an equivalent FST that has no
// /// such epsilon transitions. This version is a delayed FST.
// pub type RmEpsilonFst<W, F, B> = LazyFst<RmEpsilonImpl<W, F, B>>;
// // impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> RmEpsilonFst<W, F, B>
// // {
// //     pub fn new(fst: B) -> Self {
// //         let isymt = fst.borrow().input_symbols().cloned();
// //         let osymt = fst.borrow().output_symbols().cloned();
// //         Self::from_impl(RmEpsilonImpl::new(fst), isymt, osymt)
// //     }
// // }

use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::lazy_fst_revamp::{LazyFst2, SimpleHashMapCache};
use crate::algorithms::rm_epsilon::rm_epsilon_op::RmEpsilonOp;
use crate::fst_traits::{CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::{Semiring, SymbolTable, TrsVec};

/// The result of weight factoring is a transducer equivalent to the
/// input whose path weights have been factored according to the FactorIterator.
/// States and transitions will be added as necessary. The algorithm is a
/// generalization to arbitrary weights of the second step of the input
/// epsilon-normalization algorithm. This version is a Delayed FST.
pub struct RmEpsilonFst<W: Semiring, F: MutableFst<W>, B: Borrow<F>>(
    LazyFst2<W, RmEpsilonOp<W, F, B>, SimpleHashMapCache<W>>,
);

impl<W, F, B> CoreFst<W> for RmEpsilonFst<W, F, B>
where
    W: Semiring,
    F: MutableFst<W>,
    B: Borrow<F>,
{
    type TRS = TrsVec<W>;

    fn start(&self) -> Option<usize> {
        self.0.start()
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<W>> {
        self.0.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<W> {
        self.0.final_weight_unchecked(state_id)
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        self.0.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.0.get_trs_unchecked(state_id)
    }
}

impl<'a, W, F, B> StateIterator<'a> for RmEpsilonFst<W, F, B>
where
    W: Semiring,
    F: MutableFst<W> + 'a,
    B: Borrow<F> + 'a,
{
    type Iter =
        <LazyFst2<W, RmEpsilonOp<W, F, B>, SimpleHashMapCache<W>> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, W, F, B> FstIterator<'a, W> for RmEpsilonFst<W, F, B>
where
    W: Semiring,
    F: MutableFst<W> + 'a,
    B: Borrow<F> + 'a,
{
    type FstIter =
        <LazyFst2<W, RmEpsilonOp<W, F, B>, SimpleHashMapCache<W>> as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

impl<W, F, B> Fst<W> for RmEpsilonFst<W, F, B>
where
    W: Semiring,
    F: MutableFst<W> + 'static,
    B: Borrow<F> + 'static,
{
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.0.input_symbols()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.0.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.0.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.0.set_output_symbols(symt)
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.0.take_input_symbols()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.0.take_output_symbols()
    }
}

impl<W, F, B> Debug for RmEpsilonFst<W, F, B>
where
    W: Semiring,
    F: MutableFst<W>,
    B: Borrow<F>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a, W, F, B> RmEpsilonFst<W, F, B>
where
    W: Semiring,
    F: MutableFst<W>,
    B: Borrow<F>,
{
    pub fn new(fst: B) -> Result<Self> {
        let isymt = fst.borrow().input_symbols().cloned();
        let osymt = fst.borrow().output_symbols().cloned();
        let fst_op = RmEpsilonOp::new(fst);
        let fst_cache = SimpleHashMapCache::new();
        let lazy_fst = LazyFst2::from_op_and_cache(fst_op, fst_cache, isymt, osymt);
        Ok(RmEpsilonFst(lazy_fst))
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }
}
