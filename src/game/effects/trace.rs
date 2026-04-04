//! Trace types — re-exported from vera-effects, specialized for our Effect type.

use super::Effect;

/// Concrete trace types for saltglass-steppe.
pub type Trace = vera_effects::Trace<Effect>;
pub type TraceEntry = vera_effects::TraceEntry<Effect>;
pub type TraceSource = vera_effects::TraceSource<Effect>;
