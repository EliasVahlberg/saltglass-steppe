pub mod analysis;
pub mod constraints;
pub mod evaluation;
pub mod metrics;

// Re-export main types from analysis module
pub use analysis::{ConnectivityAnalysis, TileDistribution};
pub use constraints::ConstraintResult;
pub use evaluation::MapEvaluation;
pub use metrics::MapMetrics;

// Re-export main functions
pub use analysis::*;
pub use constraints::*;
pub use evaluation::*;
pub use metrics::*;
