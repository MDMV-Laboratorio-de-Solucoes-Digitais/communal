//! Quality functions for evaluating community partitions.
//!
//! Provides implementations of the [`QualityMetric`] trait from `communal-core`
//! including Modularity Q and the Constant Potts Model (CPM).

use communal_core::error::MetricsError;
use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;
use communal_core::partition::Partition;
use communal_core::quality::QualityMetric;

/// Internal dispatch enum for quality functions.
///
/// Used by algorithms to select which quality function to optimize
/// without dynamic dispatch overhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityFunction {
    /// Newman-Girvan modularity.
    Modularity,
    /// Constant Potts Model.
    Cpm,
    /// Map Equation (for Infomap).
    MapEquation,
}

/// Modularity Q metric.
///
/// Measures the density of edges inside communities compared to edges between
/// communities, with a resolution parameter `gamma` controlling community size.
#[derive(Debug, Clone)]
pub struct Modularity {
    gamma: f64,
}

impl Modularity {
    /// Creates a new Modularity metric with the given resolution parameter.
    ///
    /// # Arguments
    ///
    /// * `gamma` — Resolution parameter; higher values produce smaller communities.
    #[must_use]
    pub const fn new(gamma: f64) -> Self {
        Self { gamma }
    }

    /// Computes the quality gain ΔQ for moving a node to a target community.
    ///
    /// Uses the standard modularity gain formula from the Leiden paper:
    /// `ΔQ(v→C) = (1/2m) * [gain_target - loss_current]`
    ///
    /// # Arguments
    ///
    /// * `graph` — The graph view.
    /// * `partition` — The current partition.
    /// * `node` — The node to move.
    /// * `target_community` — The target community ID.
    /// * `total_weight_m` — The total edge weight (m).
    /// * `node_degrees` — Weighted degree for each node (indexed by `NodeId::index()`).
    /// * `community_degree_sums` — Sum of degrees for each community.
    #[must_use]
    #[expect(clippy::too_many_arguments, reason = "Required by the Leiden algorithm interface")]
    pub fn delta_q(
        &self,
        graph: &impl GraphView,
        partition: &Partition,
        node: NodeId,
        target_community: u32,
        total_weight_m: f64,
        node_degrees: &[f64],
        community_degree_sums: &[f64],
    ) -> f64 {
        if total_weight_m == 0.0 {
            return 0.0;
        }

        let node_idx = node.index();
        let k_v = node_degrees.get(node_idx).copied().unwrap_or(0.0);

        // Compute w_to_target: sum of edge weights from node to nodes in target_community
        let w_to_target: f64 = graph
            .neighbors(node)
            .filter(|n| partition.community_of(*n) == Some(&target_community))
            .filter_map(|n| graph.edge_weight(node, n))
            .sum();

        // Get current community of the node
        let current_community = partition.community_of(node).copied().unwrap_or(0);

        // Compute w_to_current: sum of edge weights from node to nodes in
        // current_community, excluding self-loops
        let w_to_current: f64 = graph
            .neighbors(node)
            .filter(|n| *n != node)
            .filter(|n| partition.community_of(*n) == Some(&current_community))
            .filter_map(|n| graph.edge_weight(node, n))
            .sum();

        let sum_degree_target = community_degree_sums
            .get(target_community as usize)
            .copied()
            .unwrap_or(0.0);
        let sum_degree_current = community_degree_sums
            .get(current_community as usize)
            .copied()
            .unwrap_or(0.0);

        let two_m = 2.0 * total_weight_m;
        let gain_target = w_to_target - (k_v * sum_degree_target) / two_m;
        let loss_current = w_to_current - (k_v * (sum_degree_current - k_v)) / two_m;

        (gain_target - loss_current) / two_m
    }
}

impl QualityMetric for Modularity {
    fn evaluate(&self, graph: &impl GraphView, partition: &Partition) -> Result<f64, MetricsError> {
        let node_count = graph.node_count();

        // Compute m (total edge weight, counting each undirected edge once).
        // Iterate over all nodes and their neighbors, summing weights where
        // neighbor_index >= node_index to avoid double counting.
        // Self-loops (where neighbor == node) are counted once.
        let mut m = 0.0;
        // Valid node indices are 1..=node_count (NodeId(0) is the niche value).
        let max_i = node_count;
        let mut i = 1_usize;
        while i <= max_i {
            let Some(node_i) = NodeId::new(u32::try_from(i).unwrap_or(0)) else {
                i += 1;
                continue;
            };
            for neighbor in graph.neighbors(node_i) {
                if neighbor.index() >= i && let Some(w) = graph.edge_weight(node_i, neighbor) {
                    m += w;
                }
            }
            i += 1;
        }

        if m == 0.0 {
            return Ok(0.0);
        }

        // Compute weighted degrees k_i for all nodes.
        // k_i = sum of edge weights incident to node i (including self-loops once).
        let mut degrees = vec![0.0_f64; node_count + 1];
        let mut i = 1_usize;
        while i <= max_i {
            let Some(node_i) = NodeId::new(u32::try_from(i).unwrap_or(0)) else {
                i += 1;
                continue;
            };
            for neighbor in graph.neighbors(node_i) {
                if let Some(w) = graph.edge_weight(node_i, neighbor) {
                    degrees[i] += w;
                }
            }
            i += 1;
        }

        // Compute modularity: Q = (1/2m) * Σ_ij [A_ij - γ * k_i * k_j / 2m] * δ(c_i, c_j)
        // Iterate over all edges (counting each once) where both endpoints
        // are in the same community.
        let mut q = 0.0;
        let two_m = 2.0 * m;

        let mut i = 1_usize;
        while i <= max_i {
            let Some(node_i) = NodeId::new(u32::try_from(i).unwrap_or(0)) else {
                i += 1;
                continue;
            };
            let Some(&comm_i) = partition.community_of(node_i) else {
                i += 1;
                continue;
            };
            for neighbor in graph.neighbors(node_i) {
                if neighbor.index() >= i
                    && let Some(&comm_j) = partition.community_of(neighbor)
                    && comm_i == comm_j
                    && let Some(w) = graph.edge_weight(node_i, neighbor)
                {
                    q += w - self.gamma * degrees[i] * degrees[neighbor.index()] / two_m;
                }
            }
            i += 1;
        }

        Ok(q / two_m)
    }

    fn name(&self) -> &'static str {
        "Modularity Q"
    }

    fn range(&self) -> (f64, f64) {
        (-1.0, 1.0)
    }
}

/// Constant Potts Model metric.
///
/// A resolution-limit-free quality function that penalizes intra-community
/// edges based on a resolution parameter.
#[derive(Debug, Clone)]
pub struct Cpm {
    gamma: f64,
}

impl Cpm {
    /// Creates a new CPM metric with the given resolution parameter.
    ///
    /// # Arguments
    ///
    /// * `gamma` — Resolution parameter controlling the null model penalty.
    #[must_use]
    pub const fn new(gamma: f64) -> Self {
        Self { gamma }
    }

    /// Computes the quality gain ΔQ for moving a node to a target community.
    ///
    /// Uses the CPM gain formula for a single node (`n_v` = 1):
    /// `ΔQ(v→C) = [w(v,C) + w(v,v) - γ * (2*n_C + 1)]`
    ///           `- [w(v,σ_v) - w(v,v) - γ * (2*n_σ_v - 1)]`
    ///
    /// # Arguments
    ///
    /// * `graph` — The graph view.
    /// * `partition` — The current partition.
    /// * `node` — The node to move.
    /// * `target_community` — The target community ID.
    /// * `community_sizes` — Number of nodes in each community.
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "Community sizes are small values that fit precisely in f64")]
    pub fn delta_q(
        &self,
        graph: &impl GraphView,
        partition: &Partition,
        node: NodeId,
        target_community: u32,
        community_sizes: &[usize],
    ) -> f64 {
        // Compute w_to_target: sum of edge weights from node to nodes in
        // target_community, excluding self-loops
        let w_to_target: f64 = graph
            .neighbors(node)
            .filter(|n| *n != node)
            .filter(|n| partition.community_of(*n) == Some(&target_community))
            .filter_map(|n| graph.edge_weight(node, n))
            .sum();

        // Get current community of the node
        let current_community = partition.community_of(node).copied().unwrap_or(0);

        // Compute w_to_current: sum of edge weights from node to nodes in
        // current_community, excluding self-loops
        let w_to_current: f64 = graph
            .neighbors(node)
            .filter(|n| *n != node)
            .filter(|n| partition.community_of(*n) == Some(&current_community))
            .filter_map(|n| graph.edge_weight(node, n))
            .sum();

        // Self-loop weight of the node
        let w_self = graph.edge_weight(node, node).unwrap_or(0.0);

        let n_c = community_sizes
            .get(target_community as usize)
            .copied()
            .unwrap_or(0);
        let n_sigma = community_sizes
            .get(current_community as usize)
            .copied()
            .unwrap_or(0);

        // Convert community sizes to f64 for the formula
        let n_c_f64 = n_c as f64;
        let n_sigma_f64 = n_sigma as f64;

        // gain_target = w(v,C) + w(v,v) - γ * (2*n_C + 1)
        let gain_target = w_to_target + w_self - self.gamma * (2.0 * n_c_f64 + 1.0);
        // loss_current = w(v,σ_v) - w(v,v) - γ * (2*n_σ_v - 1)
        let loss_current = w_to_current - w_self - self.gamma * (2.0 * n_sigma_f64 - 1.0);

        gain_target - loss_current
    }
}

impl QualityMetric for Cpm {
    #[expect(clippy::cast_precision_loss, reason = "Community sizes are small values that fit precisely in f64")]
    fn evaluate(&self, graph: &impl GraphView, partition: &Partition) -> Result<f64, MetricsError> {
        let node_count = graph.node_count();
        let comm_count = partition.community_count();

        // Compute e_c (intra-community edge weight) and n_c (community size)
        // for each community.
        let mut e_c = vec![0.0_f64; comm_count];
        let mut n_c = vec![0_usize; comm_count];

        // Valid node indices are 1..=node_count (NodeId(0) is the niche value).
        let max_i = node_count;
        let mut i = 1_usize;
        while i <= max_i {
            let Some(node_i) = NodeId::new(u32::try_from(i).unwrap_or(0)) else {
                i += 1;
                continue;
            };
            let Some(&comm) = partition.community_of(node_i) else {
                i += 1;
                continue;
            };
            let comm_idx = comm as usize;
            if comm_idx >= comm_count {
                i += 1;
                continue;
            }
            n_c[comm_idx] += 1;

            // Count intra-community edges (each edge once, including self-loops once)
            for neighbor in graph.neighbors(node_i) {
                if neighbor.index() >= i
                    && let Some(&neighbor_comm) = partition.community_of(neighbor)
                    && comm == neighbor_comm
                    && let Some(w) = graph.edge_weight(node_i, neighbor)
                {
                    e_c[comm_idx] += w;
                }
            }
            i += 1;
        }

        // Compute Q = Σ_c [e_c - γ * n_c * (n_c - 1) / 2]
        let mut q = 0.0;
        for c in 0..comm_count {
            let n = n_c[c] as f64;
            q += e_c[c] - self.gamma * n * (n - 1.0) / 2.0;
        }

        Ok(q)
    }

    fn name(&self) -> &'static str {
        "Constant Potts Model"
    }

    fn range(&self) -> (f64, f64) {
        (-1.0, 0.0)
    }
}
