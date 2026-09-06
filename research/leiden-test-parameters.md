# Leiden Algorithm Testing: Property-Based Test Parameters & LFR Benchmark Configuration

**Research Date:** 2026-01-28
**Purpose:** Resolve checklist items CHK008, CHK009, CHK039, CHK040 for Leiden algorithm testing specification.

---

## 1. Recommended Property-Based Test Parameter Ranges

### 1.1 Node Count Ranges

| Category | Node Count | Use Case | Reference |
|----------|-----------|----------|-----------|
| Tiny | 1–10 | Edge cases, exhaustive verification | leidenalg small-graph tests |
| Small | 10–100 | Unit tests, fast feedback | leidenalg ER(100), Tree(100) |
| Medium | 100–1,000 | Standard property tests | LFR paper N=1,000 |
| Large | 1,000–10,000 | Scaling validation | LFR paper N=5,000 & N=10,000 |
| Very Large | 10,000–100,000 | Performance/stress tests | LFR paper scaling to 10⁵–10⁶ |

**Rationale:** The LFR paper (Lancichinetti et al., 2008) explicitly tested N=1,000 and N=5,000, noting that algorithms should be tested across variable sizes to reveal limits. The leidenalg test suite uses N=100 for fast unit tests and N=100 for complete graphs.

### 1.2 Edge Density Ranges

| Category | Density Parameter | Description | Generator |
|----------|------------------|-------------|-----------|
| Very Sparse | p = 1/N | Avg degree ≈ 1, tree-like | Erdős-Rényi |
| Sparse | p = 3/N to 5/N | Avg degree 3–5 | Erdős-Rényi |
| Medium | p = 10/N | Avg degree 10 | Erdős-Rényi |
| Dense | p = 0.5 | Half of all possible edges | Erdős-Rényi |
| Complete | p = 1.0 | All possible edges | Graph.Full(n) |

**Reference:** leidenalg tests use p=1/100 and p=5/100 for N=100 (sparse and medium-sparse). The LFR paper uses average degrees ⟨k⟩ = 15, 20, 25.

### 1.3 Weight Ranges

| Category | Weight Range | Edge Type | Notes |
|----------|-------------|-----------|-------|
| Unweighted | w = 1 (implicit) | All edges equal | Default case |
| Positive Uniform | w ∈ (0, 1] | Random uniform | leidenalg: `random.random()` |
| Positive Integer | w ∈ {1, 2, ..., max} | Integer weights | For exact arithmetic tests |
| Negative | w < 0 | Dissimilarities | leidenalg tests: w = -0.1 |
| Mixed Signs | w ∈ [-1, 1] | Both attractive/repulsive | Multiplex community detection |
| Zero | w = 0 | No connection | Edge case |

**Reference:** leidenalg test suite generates weighted versions of all test graphs using `random.random()` (uniform [0,1]). Negative weight tests use w=-0.1 for bipartite detection.

### 1.4 Graph Generators

| Generator | Parameters | Properties | When to Use |
|-----------|-----------|------------|-------------|
| Erdős-Rényi (G(n,p)) | n, p | Random, homogeneous | Baseline, null model |
| Erdős-Rényi (G(n,m)) | n, m | Fixed edge count | Controlled density |
| Barabási-Albert | n, m (edges per node) | Scale-free, preferential attachment | Realistic degree distribution |
| Watts-Strogatz | n, k, β | Small-world, high clustering | Social network analogs |
| LFR | See Section 2 | Heterogeneous degrees & communities | Ground-truth benchmarking |
| Random Tree | n, branching factor | Connected, acyclic | Edge case: no cycles |
| Regular Lattice | dim, nei | Grid structure | Spatial network analogs |
| Complete Graph | n | All-to-all | Upper bound tests |
| Complete Bipartite | n1, n2 | Bipartite structure | Bipartite algorithm tests |
| Power-law Cluster | n, m, p | Clustered scale-free | Hierarchical structure |

**Reference:** leidenalg uses ER, Tree, Lattice, Complete, and Bipartite generators. The LFR paper establishes LFR as the gold standard for community detection benchmarking.

### 1.5 Test Case Counts for Statistical Confidence

| Test Type | Examples per Configuration | Justification |
|-----------|---------------------------|---------------|
| Unit tests (deterministic) | 1 per graph | Fixed input, exact output |
| Property-based (fast) | 50–200 per property | Hypothesis default sufficient |
| Property-based (slow) | 20–50 per property | Larger graphs, time constraints |
| LFR benchmark (per parameter set) | 10–100 realizations | LFR paper: 100 for N=1,000; 25 for N=5,000 |
| Scaling tests | 5–10 per size | Logarithmic size progression |

**Reference:** LFR paper used 100 graph realizations for N=1,000 and 25 for N=5,000. The NetworkX property-based testing project uses `max_examples=50 * PBT_SCALE` to `max_examples=200 * PBT_SCALE` per property.

---

## 2. Full LFR Benchmark Configuration

### 2.1 Original LFR Paper Parameters (Lancichinetti, Fortunato, Radicchi 2008)

**Source:** "Benchmark graphs for testing community detection algorithms." Physical Review E, 78, 046110. arXiv:0805.4770.

| Parameter | Symbol | Value/Range | Paper Reference |
|-----------|--------|-------------|-----------------|
| Number of nodes | N | 1,000; 5,000; 10,000 | Figs 5, 6, 3 respectively |
| Mixing parameter | μ | 0.0 → 0.6 (tested range) | Fig 5–8 x-axis |
| Average degree | ⟨k⟩ | 15; 20; 25 | Fig 5 panels |
| Maximum degree | k_max | Derived from power law | Step 1 of algorithm |
| Degree distribution exponent | τ1 (gamma) | 2 ≤ τ1 ≤ 3 | "Typical values of real networks" |
| Community size distribution exponent | τ2 (beta) | 1 ≤ τ2 ≤ 2 | "Typical values of real networks" |
| Minimum community size | c_min (s_min) | > k_min | Constraint: s_min > k_min |
| Maximum community size | c_max (s_max) | > k_max | Constraint: s_max > k_max |

### 2.2 LFR Algorithm Constraints (from original paper)

From the paper (Section II):
1. Each node shares fraction (1−μ) of links with its own community, fraction μ with others
2. Degree distribution: power law with exponent τ1
3. Community size distribution: power law with exponent τ2
4. **Critical constraint:** s_min > k_min and s_max > k_max (ensures every node fits in at least one community)
5. Typical exponent ranges: 2 ≤ τ1 ≤ 3, 1 ≤ τ2 ≤ 2

### 2.3 Standard LFR Configurations Used in Practice

**Configuration A (Easy — well-defined communities):**
- N = 1,000
- τ1 = 3, τ2 = 1
- ⟨k⟩ = 20
- k_max = 50
- μ = 0.1
- c_min = 10, c_max = 50

**Configuration B (Medium):**
- N = 5,000
- τ1 = 2, τ2 = 1
- ⟨k⟩ = 20
- k_max = 50
- μ = 0.3
- c_min = 20, c_max = 100

**Configuration C (Hard — fuzzy communities):**
- N = 10,000
- τ1 = 3, τ2 = 2
- ⟨k⟩ = 25
- k_max = 50
- μ = 0.5
- c_min = 10, c_max = 50

**Configuration D (Large-scale, from Leiden paper):**
- N = 10³ to 10⁷
- Community size = 50 (fixed)
- ⟨k⟩ = 10
- μ varied 0.1–0.9

**Reference:** Configuration A parameters from Stack Overflow community recommendation and NetworkX documentation. Configuration D from Traag et al. (2019) Nature paper.

### 2.4 LFR Parameter Validation Rules

| Rule | Constraint | Rationale |
|------|-----------|-----------|
| τ1 > 1 | Strictly greater than 1 | Power law requirement |
| τ2 > 1 | Strictly greater than 1 | Power law requirement |
| 0 ≤ μ ≤ 1 | Fraction of inter-community edges | Probabilistic interpretation |
| ⟨k⟩ < N | Average degree less than nodes | Graph feasibility |
| k_max ≤ N−1 | Max degree bounded by complete graph | Graph feasibility |
| c_min > k_min | Min community > min degree | Ensures community can contain any node |
| c_max > k_max | Max community > max degree | Ensures community can contain any node |
| Σc_i = N | Sum of community sizes equals N | Complete partition |

---

## 3. Graph Corpus Definition for "All Test Graphs"

### 3.1 Problem Statement (CHK008)

The phrase "all test graphs" is not scoped to a specific, enumerable corpus. SC-002/SC-003 reference LFR N=10k but SC-001 does not specify which graphs are covered.

### 3.2 Recommended Explicit Graph Corpus

To make "all test graphs" testable and enumerable, define the corpus as the union of:

#### Tier 1: Deterministic Reference Graphs (always tested)

| Graph | Nodes | Edges | Properties | Source |
|-------|-------|-------|------------|--------|
| Empty graph | 0 | 0 | Edge case | Synthetic |
| Singleton | 1 | 0 | Edge case | Synthetic |
| Single edge | 2 | 1 | Minimal connected | Synthetic |
| Zachary Karate Club | 34 | 78 | Weighted, 2 communities | Zachary (1977) |
| Complete graph K_n | n | n(n-1)/2 | Dense, trivial partition | Synthetic |
| Complete bipartite K_{n,m} | n+m | nm | Bipartite structure | Synthetic |
| Path graph P_n | n | n-1 | Linear chain | Synthetic |
| Cycle graph C_n | n | n | Ring structure | Synthetic |
| Star graph S_n | n | n-1 | Hub-and-spoke | Synthetic |
| Grid graph (2D) | m×n | ~2mn-m-n | Lattice structure | Synthetic |

#### Tier 2: Standard Real-World Benchmarks (tested when available)

| Graph | Nodes | Edges | Communities | Source |
|-------|-------|-------|-------------|--------|
| Dolphins | 62 | 159 | 2 | Lusseau & Newman |
| American College Football | 115 | 613 | 12 | Girvan & Newman |
| Political Books (PolBooks) | 105 | 441 | 3 | Krebs |
| Political Blogs (PolBlogs) | 1,490 | 16,715 | 2 | Adamic & Glance |
| Les Misérables | 77 | 254 | Characters | Knuth |
| NetScience | 1,589 | 2,742 | Collaboration | Newman |

#### Tier 3: Synthetic Generators (parameterized, property-based)

| Generator | Parameter Space | Count per Test |
|-----------|----------------|----------------|
| Erdős-Rényi G(n,p) | n ∈ {10, 50, 100, 500}, p ∈ {1/n, 5/n, 0.1, 0.5} | 10–50 each |
| Barabási-Albert | n ∈ {100, 1000}, m ∈ {1, 3, 5} | 10–20 each |
| Watts-Strogatz | n ∈ {100, 1000}, k ∈ {4, 6}, β ∈ {0.1, 0.5} | 10–20 each |
| LFR | See Section 2 | 10–100 per config |
| Random Trees | n ∈ {10, 50, 100, 500} | 10–20 each |

### 3.3 leidenalg Reference Corpus (Actual Implementation)

From `tests/test_VertexPartition.py` (leidenalg):

```python
graphs = [
    # Zachary karate network
    ig.Graph.Famous('Zachary'),
    
    # ER Networks (8 variants)
    ig.Graph.Erdos_Renyi(100, p=1./100, directed=False, loops=False),
    ig.Graph.Erdos_Renyi(100, p=5./100, directed=False, loops=False),
    ig.Graph.Erdos_Renyi(100, p=1./100, directed=True, loops=False),
    ig.Graph.Erdos_Renyi(100, p=5./100, directed=True, loops=False),
    ig.Graph.Erdos_Renyi(100, p=1./100, directed=False, loops=True),
    ig.Graph.Erdos_Renyi(100, p=5./100, directed=False, loops=True),
    ig.Graph.Erdos_Renyi(100, p=1./100, directed=True, loops=True),
    ig.Graph.Erdos_Renyi(100, p=5./100, directed=True, loops=True),
    
    # Trees (3 variants)
    ig.Graph.Tree(100, 3, mode='undirected'),
    ig.Graph.Tree(100, 3, mode='out'),
    ig.Graph.Tree(100, 3, mode='in'),
    
    # Lattices (2 variants)
    ig.Graph.Lattice([100], nei=3, directed=False, mutual=True, circular=True),
    ig.Graph.Lattice([100], nei=3, directed=True, mutual=False, circular=True),
]

# Plus weighted versions of all above (random uniform [0,1] weights)
graphs += [make_weighted(H) for H in graphs]
```

**Total: 30 base graphs (15 unweighted + 15 weighted)**

From `tests/test_Optimiser.py`:
- Complete graph K_100
- Complete bipartite K_{50,50}
- Disjoint union of 10 trees (10 nodes each)
- ER(100, p=5/100)

---

## 4. Property-Based Test Invariants for Leiden Algorithm

### 4.1 Core Invariants (from Traag et al. 2019)

| Invariant | Description | When Guaranteed |
|-----------|-------------|-----------------|
| **Connected communities** | Every community is internally connected | After each iteration |
| **γ-separation** | No two communities can be merged | After each iteration |
| **γ-connectivity** | Communities are γ-connected variant | After each iteration |
| **Node optimality** | No single node can improve quality | After stable iteration |
| **Subpartition γ-density** | Communities have dense substructure | After stable iteration |
| **Uniform γ-density** | No subset can be separated | After convergence |
| **Subset optimality** | All subsets locally optimal | After convergence |

### 4.2 Testable Properties for Property-Based Testing

| Property | Input Strategy | Assertion |
|----------|---------------|-----------|
| Connectedness | Any graph | All communities connected |
| Quality non-decrease | Any graph | Quality after ≥ Quality before |
| Determinism | Fixed seed | Same partition every run |
| Resolution monotonicity | Vary γ | Higher γ → more communities |
| Weight scaling | Scale weights | Partition structure preserved |
| Subset optimality | Converged partition | No subset improves quality |

---

## 5. Source Citations

### Primary Sources

1. **LFR Benchmark Paper:**
   Lancichinetti, A., Fortunato, S., & Radicchi, F. (2008). "Benchmark graphs for testing community detection algorithms." *Physical Review E*, 78, 046110. arXiv:0805.4770.
   - DOI: 10.1103/PhysRevE.78.046110
   - Defines: N, μ, ⟨k⟩, τ1, τ2, s_min, s_max constraints

2. **Leiden Algorithm Paper:**
   Traag, V. A., Waltman, L., & van Eck, N. J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9, 5233.
   - DOI: 10.1038/s41598-019-41695-z
   - Defines: Connectedness guarantees, benchmark methodology

3. **leidenalg Reference Implementation:**
   - GitHub: https://github.com/vtraag/leidenalg
   - Tests: `tests/test_VertexPartition.py`, `tests/test_Optimiser.py`
   - Test corpus: 30 graphs (15 base + 15 weighted)

4. **libleidenalg (C++ core):**
   - GitHub: https://github.com/vtraag/libleidenalg
   - Implements: Modularity, RBConfiguration, RBER, CPM, Significance, Surprise

### Secondary Sources

5. **NetworkX LFR Implementation:**
   - https://networkx.org/documentation/stable/reference/generated/networkx.generators.community.LFR_benchmark_graph.html
   - Parameters: n, tau1, tau2, mu, average_degree, min_degree, max_degree, min_community, max_community

6. **CDlib LFR Implementation:**
   - https://cdlib.readthedocs.io/en/latest/reference/generated/cdlib.benchmark.LFR.html
   - Parameters: n, tau1, tau2, mu, average_degree, min_degree, max_degree, min_community, max_community

7. **Property-Based Testing Reference:**
   - Hypothesis documentation: https://hypothesis.readthedocs.io/
   - NetworkX property tests: https://github.com/pvikram-iisc/e0251o-networkx-property-tests

8. **LFR Benchmark Dataset (Zenodo):**
   - https://zenodo.org/records/4450167
   - Pre-generated LFR graphs with standard parameters

---

## 6. Recommendations for Specification

### For CHK009 (Property-based test parameters):
- Define node count ranges: tiny (1–10), small (10–100), medium (100–1,000), large (1,000–10,000)
- Define edge density: sparse (p=1/N), medium (p=5/N), dense (p=0.5)
- Define weight ranges: positive [0,1], negative, mixed, zero
- Define generators: ER, BA, WS, LFR, trees, lattices, complete, bipartite
- Define test counts: 50–200 examples per property for fast tests, 10–50 for slow tests

### For CHK008 ("All test graphs" scoping):
- Explicitly enumerate the graph corpus as Tier 1 (deterministic) + Tier 2 (real-world) + Tier 3 (synthetic)
- Reference the leidenalg corpus as a minimum viable set
- State that "all test graphs" means the union of all tiers

### For CHK039/CHK040 (LFR parameters):
- Specify full LFR configuration: N, μ, ⟨k⟩, k_max, τ1, τ2, c_min, c_max
- Include validation rules (τ1 > 1, τ2 > 1, c_min > k_min, etc.)
- Define 3–4 standard configurations (easy/medium/hard/large) with concrete values
- Reference the original LFR paper for parameter ranges

---

*End of research document.*
