use fst::{Fst, ExpandedFst, MutableFst};
use StateId;
use semirings::Semiring;
use arc::StdArc;
use Label;
use arc::Arc;

#[derive(Debug)]
pub struct VectorFst<W: Semiring> {
    states: Vec<VectorFstState<W>>,
    start_state: Option<StateId>,
}

impl<W: Semiring> Fst<W> for VectorFst<W> {
    type Arc = StdArc<W>;
    type Iter = VectorArcIterator<W>;

    fn start(&self) -> Option<StateId> {
        self.start_state
    }

    fn final_weight(&self, state_id: &StateId) -> Option<W> {
        if let Some(state) = self.states.get(*state_id) {
            state.final_weight.clone()
        }
        else {
            None
        }
    }

    fn is_final(&self, state_id: &StateId) -> bool {
        self.final_weight(state_id).is_some()
    }

    fn arc_iter(&self, state_id: &StateId) -> Self::Iter {
        VectorArcIterator {state : self.states[*state_id].clone(), arcindex: 0}
    }

    fn num_arcs(&self) -> usize {
        self.states.iter().map(|state| state.num_arcs()).sum()
    }
}

impl<W: Semiring> ExpandedFst<W> for VectorFst<W> {
    fn num_states(&self) -> usize {
        self.states.len()
    }
}

impl<W: Semiring> MutableFst<W> for VectorFst<W> {
    fn new() -> Self {
        VectorFst {
            states: vec![],
            start_state: None,
        }
    }

    fn set_start(&mut self, state_id: &StateId) {
        assert!(self.states.get(*state_id).is_some());
        self.start_state = Some(*state_id);
    }

    fn set_final(&mut self, state_id: &StateId, final_weight: W) {
        if let Some(state) = self.states.get_mut(*state_id) {
            state.final_weight = Some(final_weight);
        }
        else {
            panic!("Stateid {:?} doesn't exist", state_id);
        }
    }

    fn add_state(&mut self) -> StateId {
        let id = self.states.len();
        self.states.insert(id, VectorFstState::new());
        id
    }

    fn add_arc(&mut self, source: &StateId, target: &StateId, ilabel: Label, olabel: Label, weight: W) {
        if let Some(state) = self.states.get_mut(*source) {
            state.arcs.push(StdArc::new(ilabel, olabel, weight, *target));
        }
        else {
            panic!("State {:?} doesn't exist", source);
        }
    }

    fn del_state(&mut self, state_to_remove: &StateId) {
        // Remove the state from the vector
        // Check the arcs for arcs going to this state

        assert!(*state_to_remove < self.states.len());
        self.states.remove(*state_to_remove);
        for state in self.states.iter_mut() {
            let mut to_delete = vec![];
            for (arc_id, arc) in state.arcs.iter_mut().enumerate() {
                if arc.nextstate() == *state_to_remove {
                    to_delete.push(arc_id);
                }
                else if arc.nextstate() > *state_to_remove {
                    arc.nextstate -= 1;
                }
            }

            for id in to_delete.iter().rev() {
                state.arcs.remove(*id);
            }
        }
    }


    fn del_states<T: IntoIterator<Item=StateId>>(&mut self, states: T) {
        let mut v: Vec<_> = states.into_iter().collect();
        v.sort();
        for j in (0..v.len()).rev() {
            self.del_state(&v[j]);
        }
    }
}

#[derive(Debug, Clone)]
pub struct VectorFstState<W: Semiring> {
    final_weight: Option<W>,
    arcs: Vec<StdArc<W>>,
}

impl<W: Semiring> VectorFstState<W> {
    pub fn new() -> Self {
        VectorFstState {
            final_weight: None,
            arcs: vec![],
        }
    }

    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }
}

#[derive(Debug)]
pub struct VectorArcIterator<W: Semiring> {
    state: VectorFstState<W>,
    arcindex: usize,
}

impl<W: Semiring> Iterator for VectorArcIterator<W> {
    type Item = StdArc<W>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.arcindex;
        let res = if i < self.state.num_arcs() {
            Some(self.state.arcs[i].clone())
        } else {
            None
        };
        self.arcindex += 1;
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semirings::integer_weight::IntegerWeight;
    use arc::Arc;

    #[test]
    fn test_1() {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(&s1);
        fst.add_arc(&s1, &s2, 3, 5, IntegerWeight::new(10));
        fst.add_arc(&s1, &s2, 5, 7, IntegerWeight::new(18));

        assert_eq!(fst.num_states(), 2);
        assert_eq!(fst.num_arcs(), 2);
        assert_eq!(fst.arc_iter(&s1).count(), 2);

        let mut it = fst.arc_iter(&s1);

        let a = it.next();
        assert!(a.is_some());
        let a = a.unwrap();
        assert_eq!(a.ilabel(), 3);
        assert_eq!(a.olabel(), 5);
        assert_eq!(a.nextstate(), s2);
        assert_eq!(a.weight(), IntegerWeight::new(10));

        let b = it.next();
        assert!(b.is_some());
        let b = b.unwrap();
        assert_eq!(b.ilabel(), 5);
        assert_eq!(b.olabel(), 7);
        assert_eq!(b.nextstate(), s2);
        assert_eq!(b.weight(), IntegerWeight::new(18));

        let c = it.next();
        assert!(c.is_none());
        // assert!(!it.done());
    }
}
