use crate::algorithms::compose::matchers::SortedMatcher;

// TODO: Change this to use InitMatcher once supported.
pub type GenericMatcher<W, F> = SortedMatcher<W, F>;
