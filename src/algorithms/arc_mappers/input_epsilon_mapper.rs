use crate::algorithms::ArcMapper;
use crate::semirings::Semiring;
use crate::Arc;
use crate::EPS_LABEL;

/// Mapper that converts all input symbols to epsilon.
pub struct InputEpsilonMapper {}

impl<S: Semiring> ArcMapper<S> for InputEpsilonMapper {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        arc.ilabel = EPS_LABEL;
    }

    fn final_weight_map(&mut self, _weight: &mut S) {}
}
