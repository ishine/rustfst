use std::marker::PhantomData;

use anyhow::Result;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::dfs_visit::{dfs_visit, Visitor};
use crate::algorithms::tr_filters::AnyTrFilter;
use crate::fst_traits::Fst;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::StateId;
use crate::Tr;
use crate::NO_STATE_ID;

/// This operation trims an FST, removing states and trs that are not on successful paths.
///
/// # Example 1
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::connect;
/// # use rustfst::fst_traits::MutableFst;
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// let fst : VectorFst<IntegerWeight> = fst![2 => 3];
///
/// // Add a state not on a successful path
/// let mut no_connected_fst = fst.clone();
/// no_connected_fst.add_state();
///
/// let mut connected_fst = no_connected_fst.clone();
/// connect(&mut connected_fst)?;
///
/// assert_eq!(connected_fst, fst);
/// # Ok(())
/// # }
/// ```
///
/// # Example 2
///
/// ## Input
///
/// ![connect_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/connect_in.svg?sanitize=true)
///
/// ## Connect
///
/// ![connect_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/connect_out.svg?sanitize=true)
///
pub fn connect<W: Semiring, F: ExpandedFst<W> + MutableFst<W>>(fst: &mut F) -> Result<()> {
    let mut visitor = ConnectVisitor::new(fst);
    dfs_visit(fst, &mut visitor, &AnyTrFilter {}, false);
    let mut dstates = Vec::with_capacity(visitor.access.len());
    for s in 0..visitor.access.len() {
        if !visitor.access[s] || !visitor.coaccess[s] {
            dstates.push(s);
        }
    }
    fst.del_states(dstates)?;
    Ok(())
}

struct ConnectVisitor<'a, W: Semiring, F: Fst<W>> {
    access: Vec<bool>,
    coaccess: Vec<bool>,
    start: usize,
    fst: &'a F,
    nstates: usize,
    dfnumber: Vec<i32>,
    lowlink: Vec<i32>,
    onstack: Vec<bool>,
    scc_stack: Vec<StateId>,
    w: PhantomData<W>,
}

impl<'a, W: Semiring, F: 'a + ExpandedFst<W>> ConnectVisitor<'a, W, F> {
    pub fn new(fst: &'a F) -> Self {
        let n = fst.num_states();
        Self {
            access: vec![false; n],
            coaccess: vec![false; n],
            start: fst.start().unwrap_or(NO_STATE_ID),
            fst,
            nstates: 0,
            dfnumber: vec![-1; n],
            lowlink: vec![-1; n],
            onstack: vec![false; n],
            scc_stack: vec![],
            w: PhantomData,
        }
    }
}

impl<'a, W: Semiring, F: 'a + ExpandedFst<W>> Visitor<'a, W, F> for ConnectVisitor<'a, W, F> {
    fn init_visit(&mut self, _fst: &'a F) {}

    fn init_state(&mut self, s: usize, root: usize) -> bool {
        self.scc_stack.push(s);
        self.dfnumber[s] = self.nstates as i32;
        self.lowlink[s] = self.nstates as i32;
        self.onstack[s] = true;
        self.access[s] = root == self.start;
        self.nstates += 1;
        true
    }

    fn tree_tr(&mut self, _s: usize, _tr: &Tr<W>) -> bool {
        true
    }

    fn back_tr(&mut self, s: usize, tr: &Tr<W>) -> bool {
        let t = tr.nextstate;
        if self.dfnumber[t] < self.lowlink[s] {
            self.lowlink[s] = self.dfnumber[t];
        }
        if self.coaccess[t] {
            self.coaccess[s] = true;
        }
        true
    }

    fn forward_or_cross_tr(&mut self, s: usize, tr: &Tr<W>) -> bool {
        let t = tr.nextstate;
        if self.dfnumber[t] < self.dfnumber[s]
            && self.onstack[t]
            && self.dfnumber[t] < self.lowlink[s]
        {
            self.lowlink[s] = self.dfnumber[t];
        }
        if self.coaccess[t] {
            self.coaccess[s] = true;
        }
        true
    }

    #[inline]
    fn finish_state(&mut self, s: usize, parent: Option<usize>, _tr: Option<&Tr<W>>) {
        if unsafe { self.fst.is_final_unchecked(s) } {
            self.coaccess[s] = true;
        }
        if self.dfnumber[s] == self.lowlink[s] {
            let mut scc_coaccess = false;
            let mut i = self.scc_stack.len();
            let mut t;
            loop {
                i -= 1;
                t = self.scc_stack[i];
                if self.coaccess[t] {
                    scc_coaccess = true;
                }
                if s == t {
                    break;
                }
            }
            loop {
                t = unsafe { *self.scc_stack.last().unsafe_unwrap() };
                if scc_coaccess {
                    self.coaccess[t] = true;
                }
                self.onstack[t] = false;
                self.scc_stack.pop();
                if s == t {
                    break;
                }
            }
        }
        if let Some(_p) = parent {
            if self.coaccess[s] {
                self.coaccess[_p] = true;
            }
            if self.lowlink[s] < self.lowlink[_p] {
                self.lowlink[_p] = self.lowlink[s];
            }
        }
    }

    #[inline]
    fn finish_visit(&mut self) {}
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::fst_properties::FstProperties;
    use crate::proptest_fst::proptest_fst;

    use super::*;

    proptest! {
        #[test]
        fn test_connect_proptest(mut fst in proptest_fst()) {
            connect(&mut fst).unwrap();
            prop_assume!(fst.properties().unwrap().intersects(
                FstProperties::ACCESSIBLE | FstProperties::COACCESSIBLE
            ));
        }
    }
}
