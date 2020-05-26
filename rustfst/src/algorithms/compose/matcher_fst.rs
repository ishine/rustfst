use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::lookahead_matchers::{LabelLookAheadRelabeler, LookaheadMatcher};
use crate::algorithms::compose::matchers::MatchType;
use crate::algorithms::compose::FstAddOn;
use crate::algorithms::compose::LabelReachableData;
use crate::fst_traits::{
    CoreFst, ExpandedFst, Fst, FstIntoIterator, FstIterator, MutableFst, StateIterator,
};
use crate::semirings::Semiring;
use crate::SymbolTable;

#[derive(Clone, PartialEq, Debug)]
pub struct MatcherFst<W, F, M, T> {
    fst_add_on: FstAddOn<F, (Option<Arc<T>>, Option<Arc<T>>)>,
    matcher: PhantomData<M>,
    w: PhantomData<W>,
}

impl<W, F, M, T> MatcherFst<W, F, M, T> {
    pub fn fst(&self) -> &F {
        self.fst_add_on.fst()
    }

    pub fn addon(&self) -> &(Option<Arc<T>>, Option<Arc<T>>) {
        self.fst_add_on.add_on()
    }

    pub fn data(&self, match_type: MatchType) -> Option<&Arc<T>> {
        let data = self.fst_add_on.add_on();
        if match_type == MatchType::MatchInput {
            data.0.as_ref()
        } else {
            data.1.as_ref()
        }
    }
}

// TODO: To be generalized
impl<W, F, M> MatcherFst<W, F, M, M::MatcherData>
where
    W: Semiring,
    F: MutableFst<W>,
    M: LookaheadMatcher<W, F = F, MatcherData = LabelReachableData>,
{
    pub fn new(mut fst: F) -> Result<Self> {
        let imatcher_data = M::create_data(&fst, MatchType::MatchInput)?;
        let omatcher_data = M::create_data(&fst, MatchType::MatchOutput)?;

        let mut add_on = (imatcher_data, omatcher_data);
        LabelLookAheadRelabeler::init(&mut fst, &mut add_on)?;

        let add_on = (add_on.0.map(Arc::new), add_on.1.map(Arc::new));

        let fst_add_on = FstAddOn::new(fst, add_on);
        Ok(Self {
            fst_add_on,
            matcher: PhantomData,
            w: PhantomData,
        })
    }

    // Construct a new Matcher Fst intended for LookAhead composition and relabel fst2 wrt to the first fst.
    pub fn new_with_relabeling<F2: MutableFst<W>>(
        mut fst: F,
        fst2: &mut F2,
        relabel_input: bool,
    ) -> Result<Self> {
        let imatcher_data = M::create_data(&fst, MatchType::MatchInput)?;
        let omatcher_data = M::create_data(&fst, MatchType::MatchOutput)?;

        let mut add_on = (imatcher_data, omatcher_data);
        LabelLookAheadRelabeler::init(&mut fst, &mut add_on)?;
        LabelLookAheadRelabeler::relabel(fst2, &mut add_on, relabel_input)?;

        let add_on = (add_on.0.map(Arc::new), add_on.1.map(Arc::new));

        let fst_add_on = FstAddOn::new(fst, add_on);
        Ok(Self {
            fst_add_on,
            matcher: PhantomData,
            w: PhantomData,
        })
    }
}

impl<W: Semiring, F: CoreFst<W>, M, T> CoreFst<W> for MatcherFst<W, F, M, T> {
    type TRS = <FstAddOn<F, T> as CoreFst<W>>::TRS;

    fn start(&self) -> Option<usize> {
        self.fst_add_on.start()
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<W>> {
        self.fst_add_on.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<W> {
        self.fst_add_on.final_weight_unchecked(state_id)
    }

    fn num_trs(&self, s: usize) -> Result<usize> {
        self.fst_add_on.num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
        self.fst_add_on.num_trs_unchecked(s)
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        self.fst_add_on.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.fst_add_on.get_trs_unchecked(state_id)
    }
}

impl<'a, W, F: StateIterator<'a>, M, T> StateIterator<'a> for MatcherFst<W, F, M, T> {
    type Iter = <F as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.fst_add_on.states_iter()
    }
}

impl<'a, W, F, M, T> FstIterator<'a, W> for MatcherFst<W, F, M, T>
where
    W: Semiring,
    F: FstIterator<'a, W>,
{
    type FstIter = F::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.fst_add_on.fst_iter()
    }
}

impl<W, F, M, T> Fst<W> for MatcherFst<W, F, M, T>
where
    W: Semiring,
    F: Fst<W>,
    M: Debug,
    T: Debug,
{
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.fst_add_on.input_symbols()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.fst_add_on.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.fst_add_on.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.fst_add_on.set_output_symbols(symt)
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.fst_add_on.take_input_symbols()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.fst_add_on.take_output_symbols()
    }
}

impl<W, F, M, T> ExpandedFst<W> for MatcherFst<W, F, M, T>
where
    W: Semiring,
    F: ExpandedFst<W>,
    M: Debug + Clone + PartialEq,
    T: Debug + Clone + PartialEq,
{
    fn num_states(&self) -> usize {
        self.fst_add_on.num_states()
    }
}

impl<W, F, M, T> FstIntoIterator<W> for MatcherFst<W, F, M, T>
where
    W: Semiring,
    F: FstIntoIterator<W>,
    T: Debug,
{
    type TrsIter = F::TrsIter;
    type FstIter = F::FstIter;

    fn fst_into_iter(self) -> Self::FstIter {
        self.fst_add_on.fst_into_iter()
    }
}
