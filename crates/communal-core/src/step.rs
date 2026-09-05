use crate::id::{CommunityId, NodeId};

/// Identifies which algorithm phase emitted an event.
#[derive(Debug, Clone)]
pub enum AlgorithmPhase {
    /// Local moving phase (node reassignment).
    LocalMoving,
    /// Refinement phase (community splitting).
    Refinement,
    /// Aggregation phase (graph coarsening).
    Aggregation,
    /// Convergence detection phase.
    Convergence,
}

/// Discrete algorithmic events for observability and debugging.
///
/// These events are emitted during algorithm execution to enable progress
/// tracking, logging, and visualization of the detection process.
#[derive(Debug, Clone)]
pub enum StepEvent {
    /// The local moving phase has started for a given iteration.
    LocalMovingStart {
        /// Current iteration number.
        iteration: usize,
    },
    /// The local moving phase has ended for a given iteration.
    LocalMovingEnd {
        /// Current iteration number.
        iteration: usize,
    },
    /// A single node was relocated from one community to another.
    NodeRelocation {
        /// The node that moved.
        node: NodeId,
        /// Previous community.
        from: CommunityId,
        /// New community.
        to: CommunityId,
    },
    /// A community was split during refinement.
    RefinementSplit {
        /// The community that was split.
        community: CommunityId,
        /// Number of resulting sub-communities.
        into: usize,
    },
    /// Communities were contracted during aggregation.
    AggregationContraction {
        /// Number of communities before contraction.
        from_communities: usize,
        /// Number of communities after contraction.
        to_communities: usize,
    },
    /// Two communities were merged.
    CommunityMerge {
        /// The surviving community.
        into: CommunityId,
        /// The community that was absorbed.
        merged: CommunityId,
    },
    /// A community was split into multiple communities.
    CommunitySplit {
        /// The original community.
        from: CommunityId,
        /// The resulting communities.
        into: Vec<CommunityId>,
    },
    /// A boundary between algorithm phases or iterations.
    IterationBoundary {
        /// The phase that is starting or ending.
        phase: AlgorithmPhase,
        /// The iteration number.
        iteration: usize,
    },
    /// The algorithm has plateaued below the convergence threshold.
    ConvergencePlateau {
        /// Number of consecutive iterations below threshold.
        iterations_below_threshold: usize,
    },
    /// The algorithm has converged.
    ConvergenceDetected {
        /// Total iterations performed.
        total_iterations: usize,
        /// Final quality score.
        final_quality: f64,
    },
}
