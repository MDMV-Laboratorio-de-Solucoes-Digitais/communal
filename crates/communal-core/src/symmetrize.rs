/// Symmetrizes edge weights using arithmetic mean: `(w_ij + w_ji) / 2`.
///
/// This function computes the symmetric weight for a pair of directed edges
/// `(i, j)` and `(j, i)`, producing a single undirected weight.
///
/// # Examples
///
/// ```
/// use communal_core::symmetrize::symmetrize_weight;
///
/// let w = symmetrize_weight(0.8, 1.2);
/// assert_eq!(w, 1.0);
/// ```
#[must_use]
pub fn symmetrize_weight(w_ij: f64, w_ji: f64) -> f64 {
    f64::midpoint(w_ij, w_ji)
}
