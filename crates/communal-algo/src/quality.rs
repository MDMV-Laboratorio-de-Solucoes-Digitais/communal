//! Quality functions for evaluating community partitions.
//!
//! Provides implementations of the [`QualityMetric`] trait from `communal-core`
//! including Modularity Q and the Constant Potts Model (CPM).

use std::collections::HashMap;

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
    pub fn new(gamma: f64) -> Self {
        Self { gamma }
    }

    /// Converts a 0-based node index to a `NodeId`.
    ///
    /// `NodeId` values are 1-based (they wrap `NonZeroU32`), so index 0 maps
    /// to raw value 1, index 1 to raw value 2, and so on.
    fn index_to_node(index: usize) -> Option<NodeId> {
        let raw_id = u32::try_from(index).ok()?.checked_add(1)?;
        NodeId::new(raw_id)
    }

    /// Converts a `NodeId` to a 0-based node index.
    fn node_to_index(node: NodeId) -> usize {
        node.index().saturating_sub(1)
    }

    /// Computes the weighted degree of a node.
    fn weighted_degree<G: GraphView>(graph: &G, node: NodeId) -> f64 {
        graph
            .neighbors(node)
            .filter_map(|neighbor| graph.edge_weight(node, neighbor))
            .sum()
    }

    /// Computes the total weight of all directed edges (2m).
    fn total_edge_weight<G: GraphView>(graph: &G) -> f64 {
        let mut total = 0.0;
        for node_idx in 0..graph.node_count() {
            let Some(node) = Self::index_to_node(node_idx) else {
                continue;
            };
            for neighbor in graph.neighbors(node) {
                if let Some(w) = graph.edge_weight(node, neighbor) {
                    total += w;
                }
            }
        }
        total
    }

    /// Computes the quality gain for moving a node to a target community.
    ///
    /// Uses the formula:
    /// ```text
    /// ΔQ(v→C) = (1/2m) * [(w(v,C) - w(v,σ_v)) - γ * k_v * (K_C - K_σ_v) / 2m]
    /// ```
    ///
    /// Where:
    /// * `w(v,C)` — sum of edge weights from `v` to nodes in `C`
    /// * `w(v,σ_v)` — sum of edge weights from `v` to nodes in its current community
    /// * `k_v` — weighted degree of `v`
    /// * `K_C` — sum of weighted degrees of nodes in `C`
    /// * `K_σ_v` — sum of weighted degrees of nodes in `σ_v`
    /// * `2m` — total edge weight
    pub fn delta_q<G: GraphView>(
        &self,
        graph: &G,
        membership: &[u32],
        node: NodeId,
        target_community: u32,
    ) -> f64 {
        let node_idx = Self::node_to_index(node);
        if node_idx >= membership.len() {
            return 0.0;
        }

        let current_community = membership[node_idx];
        if current_community == target_community {
            return 0.0;
        }

        let k_v = Self::weighted_degree(graph, node);
        let total_weight = Self::total_edge_weight(graph);

        if total_weight == 0.0 {
            return 0.0;
        }

        // Compute w(v, C) and w(v, σ_v)
        let mut w_to_target = 0.0_f64;
        let mut w_to_current = 0.0_f64;

        for neighbor in graph.neighbors(node) {
            let neighbor_idx = Self::node_to_index(neighbor);
            if neighbor_idx >= membership.len() {
                continue;
            }
            let Some(w) = graph.edge_weight(node, neighbor) else {
                continue;
            };
            if membership[neighbor_idx] == target_community {
                w_to_target += w;
            }
            if membership[neighbor_idx] == current_community {
                w_to_current += w;
            }
        }

        // Compute K_C and K_σ_v (sum of weighted degrees in each community)
        let mut k_target = 0.0_f64;
        let mut k_current = 0.0_f64;

        for (i, &community) in membership.iter().enumerate() {
            let Some(n) = Self::index_to_node(i) else {
                continue;
            };
            let degree = Self::weighted_degree(graph, n);
            if community == target_community {
                k_target += degree;
            }
            if community == current_community {
                k_current += degree;
            }
        }

        let delta_e = w_to_target - w_to_current;
        let delta_k = k_target - k_current;

        (delta_e - self.gamma * k_v * delta_k / total_weight) / total_weight
    }
}

impl QualityMetric for Modularity {
    /// Evaluates the Newman-Girvan modularity of a partition.
    ///
    /// Uses the formula:
    /// ```text
    /// Q = (1/2m) * Σ_ij [A_ij - γ * k_i * k_j / 2m] * δ(c_i, c_j)
    /// ```
    ///
    /// The sum is computed by iterating over all directed edges; each undirected
    /// edge is visited twice (once from each endpoint), which matches the
    /// formula's double counting.
    fn evaluate(&self, graph: &impl GraphView, partition: &Partition) -> Result<f64, MetricsError> {
        let node_count = graph.node_count();
        let membership = partition.membership_vec();

        if membership.is_empty() {
            return Err(MetricsError::EmptyPartition);
        }

        // Compute weighted degree for every node and the total edge weight (2m).
        let mut degrees = vec![0.0_f64; node_count];
        let mut total_weight = 0.0_f64;

        for (node_idx, degree) in degrees.iter_mut().enumerate().take(node_count) {
            let node = Self::index_to_node(node_idx).ok_or(MetricsError::NumericalOverflow)?;
            for neighbor in graph.neighbors(node) {
                let Some(w) = graph.edge_weight(node, neighbor) else {
                    continue;
                };
                *degree += w;
                total_weight += w;
            }
        }

        if total_weight == 0.0 {
            return Ok(0.0);
        }

        // Accumulate the modularity sum over all directed edges.
        let mut q = 0.0_f64;

        for node_idx in 0..node_count {
            let node = Self::index_to_node(node_idx).ok_or(MetricsError::NumericalOverflow)?;
            let c_i = membership.get(node_idx).ok_or(MetricsError::EmptyPartition)?;

            for neighbor in graph.neighbors(node) {
                let neighbor_idx = Self::node_to_index(neighbor);
                let c_j = membership.get(neighbor_idx).ok_or(MetricsError::EmptyPartition)?;

                if c_i != c_j {
                    continue;
                }

                let Some(w) = graph.edge_weight(node, neighbor) else {
                    continue;
                };

                if neighbor_idx >= node_count {
                    continue;
                }

                q += w - self.gamma * degrees[node_idx] * degrees[neighbor_idx] / total_weight;
            }
        }

        Ok(q / total_weight)
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
    pub fn new(gamma: f64) -> Self {
        Self { gamma }
    }

    /// Converts a 0-based node index to a `NodeId`.
    fn index_to_node(index: usize) -> Option<NodeId> {
        let raw_id = u32::try_from(index).ok()?.checked_add(1)?;
        NodeId::new(raw_id)
    }

    /// Converts a `NodeId` to a 0-based node index.
    fn node_to_index(node: NodeId) -> usize {
        node.index().saturating_sub(1)
    }

    /// Computes the quality gain for moving a node to a target community.
    ///
    /// Uses the formula:
    /// ```text
    /// ΔQ(v→C) = [w(v,C) + w(v,v) - γ·n_v·(2·n_C + n_v)]
    ///         - [w(v,σ_v) - w(v,v) - γ·n_v·(2·n_σ_v - n_v)]
    /// ```
    ///
    /// With `n_v = 1` for a single node this simplifies to:
    /// ```text
    /// ΔQ(v→C) = w(v,C) - w(v,σ_v) + 2·w(v,v) - 2·γ·(n_C - n_σ_v + 1)
    /// ```
    pub fn delta_q<G: GraphView>(
        &self,
        graph: &G,
        membership: &[u32],
        node: NodeId,
        target_community: u32,
    ) -> f64 {
        let node_idx = Self::node_to_index(node);
        if node_idx >= membership.len() {
            return 0.0;
        }

        let current_community = membership[node_idx];
        if current_community == target_community {
            return 0.0;
        }

        // Compute w(v, C), w(v, σ_v), and w(v, v)
        let mut w_to_target = 0.0_f64;
        let mut w_to_current = 0.0_f64;
        let mut w_self = 0.0_f64;

        for neighbor in graph.neighbors(node) {
            let neighbor_idx = Self::node_to_index(neighbor);
            let Some(w) = graph.edge_weight(node, neighbor) else {
                continue;
            };

            if neighbor_idx < membership.len() {
                if membership[neighbor_idx] == target_community {
                    w_to_target += w;
                }
                if membership[neighbor_idx] == current_community {
                    w_to_current += w;
                }
            }

            if neighbor.index() == node.index() {
                w_self = w;
            }
        }

        // Count nodes in each community
        let mut n_target = 0_u64;
        let mut n_current = 0_u64;

        for &community in membership {
            if community == target_community {
                n_target += 1;
            }
            if community == current_community {
                n_current += 1;
            }
        }

        #[expect(clippy::cast_precision_loss, reason = "u64 to f64 conversion is safe for community counts which are bounded by node count")]
        let n_target_f = n_target as f64;
        #[expect(clippy::cast_precision_loss, reason = "u64 to f64 conversion is safe for community counts which are bounded by node count")]
        let n_current_f = n_current as f64;

        // ΔQ = w(v,C) - w(v,σ_v) + 2·w(v,v) - 2·γ·(n_C - n_σ_v + 1)
        w_to_target - w_to_current + 2.0 * w_self
            - 2.0 * self.gamma * (n_target_f - n_current_f + 1.0)
    }
}

impl QualityMetric for Cpm {
    /// Evaluates the Constant Potts Model quality of a partition.
    ///
    /// Uses the formula:
    /// ```text
    /// Q = Σ_c [e_c - γ * (n_c choose 2)]
    /// ```
    ///
    /// Where `e_c` is the total weight of intra-community edges (each edge
    /// counted once, self-loops counted once) and `n_c` is the number of nodes
    /// in community `c`.
    fn evaluate(&self, graph: &impl GraphView, partition: &Partition) -> Result<f64, MetricsError> {
        let node_count = graph.node_count();
        let membership = partition.membership_vec();

        if membership.is_empty() {
            return Err(MetricsError::EmptyPartition);
        }

        // Accumulate intra-community edge weight and node count per community.
        let mut community_weight: HashMap<u32, f64> = HashMap::new();
        let mut community_size: HashMap<u32, usize> = HashMap::new();

        for node_idx in 0..node_count {
            let node = Self::index_to_node(node_idx).ok_or(MetricsError::NumericalOverflow)?;
            let c_i = membership.get(node_idx).ok_or(MetricsError::EmptyPartition)?;

            *community_size.entry(*c_i).or_insert(0) += 1;

            for neighbor in graph.neighbors(node) {
                let neighbor_idx = Self::node_to_index(neighbor);
                let c_j = membership.get(neighbor_idx).ok_or(MetricsError::EmptyPartition)?;

                if c_i != c_j {
                    continue;
                }

                let Some(w) = graph.edge_weight(node, neighbor) else {
                    continue;
                };

                // Self-loops are stored once in the CSR; regular edges twice.
                // We want each edge counted exactly once, so halve regular edges.
                let contribution = if node.index() == neighbor.index() {
                    w
                } else {
                    w / 2.0
                };

                *community_weight.entry(*c_i).or_insert(0.0) += contribution;
            }
        }

        let mut q = 0.0_f64;
        for (community, &size) in &community_size {
            let e_c = community_weight.get(community).copied().unwrap_or(0.0);
            #[expect(clippy::cast_precision_loss, reason = "usize to f64 conversion is safe for community sizes which are bounded by node count")]
            let n_c = size as f64;
            q += e_c - self.gamma * n_c * (n_c - 1.0) / 2.0;
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
