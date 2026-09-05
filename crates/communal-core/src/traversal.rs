use crate::graph_view::GraphView;
use crate::id::NodeId;
use std::collections::VecDeque;

/// BFS traversal returning all nodes reachable from the start node.
///
/// Performs a breadth-first search starting from `start`, visiting all reachable
/// nodes in breadth-first order.
///
/// # Returns
///
/// A vector of [`NodeId`] values in the order they were visited.
pub fn bfs<G: GraphView>(graph: &G, start: NodeId) -> Vec<NodeId> {
    let mut visited = vec![false; graph.node_count()];
    let mut queue = VecDeque::new();
    let mut result = Vec::new();
    queue.push_back(start);
    visited[start.index()] = true;
    while let Some(node) = queue.pop_front() {
        result.push(node);
        for neighbor in graph.neighbors(node) {
            if !visited[neighbor.index()] {
                visited[neighbor.index()] = true;
                queue.push_back(neighbor);
            }
        }
    }
    result
}

/// DFS traversal returning all nodes reachable from the start node.
///
/// Performs a depth-first search starting from `start`, visiting all reachable
/// nodes in depth-first order.
///
/// # Returns
///
/// A vector of [`NodeId`] values in the order they were visited.
pub fn dfs<G: GraphView>(graph: &G, start: NodeId) -> Vec<NodeId> {
    let mut visited = vec![false; graph.node_count()];
    let mut stack = vec![start];
    let mut result = Vec::new();
    while let Some(node) = stack.pop() {
        if visited[node.index()] {
            continue;
        }
        visited[node.index()] = true;
        result.push(node);
        for neighbor in graph.neighbors(node) {
            if !visited[neighbor.index()] {
                stack.push(neighbor);
            }
        }
    }
    result
}
