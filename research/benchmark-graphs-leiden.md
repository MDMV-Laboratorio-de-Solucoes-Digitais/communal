# Benchmark Graphs with Known Leiden Solutions: Reference Corpus for Communal

**Research Date:** 2026-03-06
**Purpose:** Identify the largest, hardest, and most well-characterized graphs with known/proven Leiden algorithm solutions for testing the Communal Rust-based community detection framework.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [LFR Benchmark Graphs](#2-lfr-benchmark-graphs)
3. [Real-World Benchmark Graphs](#3-real-world-benchmark-graphs)
4. [Reference Solutions](#4-reference-solutions)
5. [Specific Graph Recommendations](#5-specific-graph-recommendations)
6. [Downloadable Test Corpora](#6-downloadable-test-corpora)
7. [What Cannot Be Verified](#7-what-cannot-be-verified)
8. [Source Citations](#8-source-citations)

---

## 1. Executive Summary

**Key finding:** There is no single "reference partition" for the Leiden algorithm on real-world graphs. Unlike LFR benchmarks (where the planted partition is ground truth by construction), real-world graphs have no single "correct" Leiden output — the partition depends on random seed, iteration count, and quality function. The testing strategy must therefore be multi-pronged:

- **LFR graphs:** Compare detected partition against planted ground truth (NMI/ARI)
- **Property-based:** Verify Leiden's formal guarantees (connectedness, γ-separation, γ-connectivity)
- **Modularity targets:** Compare achieved modularity against published reference values from leidenalg/igraph
- **Largest proven LFR:** N=19,200 (Zenodo dataset); largest in Traag et al. paper: N=10⁷ (simplified benchmark variant)
- **Largest real-world with published Leiden modularity:** Web UK (39.25M nodes, Q=0.9801)

---

## 2. LFR Benchmark Graphs

### 2.1 Original LFR Benchmark (Lancichinetti, Fortunato, Radicchi 2008)

The LFR benchmark is the gold standard for community detection evaluation. It generates graphs with power-law degree and community size distributions, with a mixing parameter μ controlling the fraction of inter-community edges.

**Source:** [Lancichinetti, A., Fortunato, S., & Radicchi, F. (2008). "Benchmark graphs for testing community detection algorithms." *Physical Review E*, 78, 046110.](https://doi.org/10.1103/PhysRevE.78.046110) — arXiv:0805.4770

**Key property:** Every LFR graph comes with a **known planted partition** — this is the ground truth by construction. The Leiden algorithm's output can be compared against this using Normalized Mutual Information (NMI) or Adjusted Rand Index (ARI).

**Standard parameters (from original paper):**

| Parameter | Symbol | Standard Values |
|-----------|--------|-----------------|
| Number of nodes | N | 1,000; 5,000; 10,000 |
| Mixing parameter | μ | 0.0 → 0.6 |
| Average degree | ⟨k⟩ | 15; 20; 25 |
| Degree distribution exponent | τ1 | 2 ≤ τ1 ≤ 3 |
| Community size distribution exponent | τ2 | 1 ≤ τ2 ≤ 2 |

### 2.2 Pre-Generated LFR Dataset (Zenodo — Toth et al. 2021)

The most directly usable LFR reference corpus with ground truth:

**Source:** [Toth, C. et al. (2021). "A collection of LFR benchmark graphs." Zenodo.](https://zenodo.org/records/4450167)

| Set | Node Sizes | Avg Degrees | Mixing Params | Graphs per Config | Total |
|-----|-----------|-------------|---------------|-------------------|-------|
| **Set A** | 300–19,200 | 15, 25, 50 | 20 values in [0.2, 0.8] | 100 | ~72,000 |
| **Set B** | 300–2,400 | 20 | 20 values in [0.2, 0.8] | 100 | ~12,000 |

**Format:** Edge list + ground truth membership list (JSON) + random seeds + network statistics

**Largest graph:** N=19,200 (Set A, avg degree 50, max community size 0.2N)

**License:** Open access (Zenodo public dataset)

**Generating code:** [Fortunato's original LFR code](https://www.santofortunato.net/resources) embedded in [synwalk-analysis](https://github.com/synwalk/synwalk-analysis)

### 2.3 CDlib LFR Datasets

**Source:** [CDlib datasets repository](https://github.com/GiulioRossetti/cdlib_datasets)

Pre-generated LFR benchmarks with planted ground truth:

| Parameter | Values |
|-----------|--------|
| Number of nodes | 1,000; 5,000; 10,000; 50,000; 100,000 |
| Average degree | 5 |
| Min community size | 50 |
| Mixing coefficient | 0.1–0.9 (9 values) |
| Degree distribution exponent | 3 (fixed) |
| Community size distribution exponent | 1.5 (fixed) |

**Naming pattern:** `LFR_N{nodes}_ad{degree}_mc{min_community}_mu{mixing}`

**Largest:** N=100,000 — significantly larger than Zenodo set

**Access:** Via `cdlib.datasets.fetch_network_data()` and `cdlib.datasets.fetch_ground_truth_data()`

### 2.4 Traag et al. 2019 Benchmark Variant

The Leiden paper uses a **simplified LFR variant** (not the full LFR model):

**Source:** [Traag, V.A., Waltman, L. & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9, 5233.](https://doi.org/10.1038/s41598-019-41695-z) — arXiv:1810.08473

| Parameter | Value |
|-----------|-------|
| Number of nodes | 10³ to 10⁷ |
| Community size | 50 (fixed, equal) |
| Average degree | 10 |
| Mixing parameter | 0.1–0.9 |
| Quality function | CPM (Constant Potts Model) |

**Critical note:** This is NOT the standard LFR model — communities are equal-sized (not power-law distributed). The ground truth is still known by construction. The paper reports quality as H/(2m) where H is the CPM quality.

**Largest:** N=10⁷ (10 million nodes) — this is the largest graph with known community structure used in the Leiden paper.

---

## 3. Real-World Benchmark Graphs

### 3.1 SNAP Ground-Truth Communities

**Source:** [SNAP AGM Datasets](https://snap.stanford.edu/agm/) — Stanford Network Analysis Project

These 6 datasets have explicitly identified ground-truth communities (user-defined groups, circles, etc.):

| Dataset | Nodes | Edges | Communities | Type |
|---------|-------|-------|-------------|------|
| **LiveJournal** | 3,997,962 | 34,681,189 | 664,414 | Social network |
| **Friendster** | 65,608,366 | 1,806,067,135 | 1,620,991 | Social network |
| **Orkut** | 3,072,441 | 117,185,083 | 15,301,901 | Social network |
| **Youtube** | 1,134,890 | 2,987,624 | 16,386 | Social network |
| **DBLP** | 317,080 | 1,049,866 | 13,477 | Collaboration |
| **Amazon** | 334,863 | 925,872 | 271,570 | Co-purchase |

**Ground truth definition:** User-created groups/circles (social), co-purchase clusters (Amazon), publication venues (DBLP)

**License:** Free for research use (SNAP terms)

**Download:** [SNAP data page](https://snap.stanford.edu/data/)

### 3.2 Traag et al. 2019 Empirical Networks

The Leiden paper tested on these 6 real-world networks (modularity with γ=1):

| Network | Nodes | Avg Degree | Louvain Q | Leiden Q | Source |
|---------|-------|-----------|-----------|----------|--------|
| **DBLP** | 317,080 | 6.6 | 0.8262 | 0.8387 | SNAP |
| **Amazon** | 334,863 | 5.6 | 0.9301 | 0.9341 | SNAP |
| **IMDB** | 374,511 | 80.2 | 0.7062 | 0.7069 | Sparse Matrix (Tamu) |
| **LiveJournal** | 3,997,962 | 17.4 | 0.7653 | 0.7739 | SNAP |
| **Web of Science** | 9,811,130 | 21.2 | 0.7911 | 0.7951 | Restricted license |
| **Web UK** | 39,252,879 | 39.8 | 0.9796 | 0.9801 | LAW (Univ. Milano) |

**Important:** These modularity values are **reference targets**, not "ground truth partitions." The Leiden algorithm's exact output varies with random seed. The paper reports the **maximum modularity over 10 replications of 10 iterations each**.

**Largest:** Web UK (39.25M nodes, ~780M edges) — but data has license restrictions.

### 3.3 Standard Real-World Benchmarks (Small, Well-Characterized)

These are the canonical test graphs used across the community detection literature:

| Graph | Nodes | Edges | Known Communities | Ground Truth Type | Source |
|-------|-------|-------|-------------------|-------------------|--------|
| **Zachary Karate Club** | 34 | 78 | 2 | Faction split (observed) | [Zachary (1977)](http://www-personal.umich.edu/~mejn/netdata/) |
| **Dolphins** | 62 | 159 | 2 | Observed groups | [Lusseau & Newman (2003)](http://www-personal.umich.edu/~mejn/netdata/) |
| **American College Football** | 115 | 613 | 12 | Conference membership | [Girvan & Newman (2004)](http://www-personal.umich.edu/~mejn/netdata/) |
| **Political Books (PolBooks)** | 105 | 441 | 3 | Liberal/Conservative/Neutral | [Krebs](http://www-personal.umich.edu/~mejn/netdata/) |
| **Political Blogs (PolBlogs)** | 1,490 | 16,715 | 2 | Left/Right | [Adamic & Glance (2005)](http://www-personal.umich.edu/~mejn/netdata/) |
| **Les Misérables** | 77 | 254 | Characters | Co-appearance | [Knuth](http://www-personal.umich.edu/~mejn/netdata/) |
| **NetScience** | 1,589 | 2,742 | Collaboration | Co-authorship | [Newman](http://www-personal.umich.edu/~mejn/netdata/) |
| **email-Eu-core** | 1,005 | 16,706 | 42 | Department membership | [SNAP](https://snap.stanford.edu/data/email-Eu-core.html) |

**Note on ground truth:** For these small graphs, the "ground truth" is externally defined (conference membership, political leaning, etc.). The Leiden algorithm optimizes modularity/CPM, which may not perfectly align with these external labels. This is expected — it's a limitation of the evaluation methodology, not the algorithm.

### 3.4 Community-Graphs Repository (vlivashkin)

**Source:** [GitHub: vlivashkin/community-graphs](https://github.com/vlivashkin/community-graphs)

A curated collection of 41 graphs with non-overlapping ground truth partitions in GML format:

| Family | Graphs | Sizes | Source |
|--------|--------|-------|--------|
| Cora | cora (2,708), cora_full (23,166), 11 subsets | 457–23,166 nodes | Citation networks |
| Newsgroup | 18 graphs (2–5 classes) | 398–999 nodes | Text classification |
| Standard benchmarks | karate, dolphins, football, polbooks, polblogs, eu-core, sp-school | 34–1,490 nodes | Various |
| AS Internet | as (23,752) | 23,752 nodes | Topology |

**Format:** GML with `gt` attribute for ground truth labels

**License:** MIT

**Largest:** AS Internet (23,752 nodes, 58,416 edges, 176 communities) and cora_full (23,166 nodes, 89,157 edges, 70 communities)

### 3.5 KONECT (Koblenz Network Collection)

**Source:** [KONECT — Koblenz Network Collection](http://konect.cc/)

- 160+ network datasets (directed, undirected, bipartite, weighted, signed, temporal)
- Includes network statistics and visualizations
- **Caveat:** KONECT provides network data but does NOT provide ground truth community labels — it's a network collection, not a community detection benchmark

### 3.6 Network Repository

**Source:** [Network Repository](https://networkrepository.com/)

- Thousands of real-world networks with interactive visualization
- Largest scientific network data repository
- **Caveat:** Like KONECT, provides network data but not ground truth community labels

---

## 4. Reference Solutions

### 4.1 What "Reference Solution" Means for Leiden

**Critical distinction:** For the Leiden algorithm, there are three types of "reference solutions":

1. **LFR planted partition** — The known ground truth by construction. The algorithm's output is compared against this using NMI/ARI. This is the only truly "proven" solution.

2. **Published modularity values** — For real-world graphs, papers report the maximum modularity achieved. These are reference targets for the quality function value, not for the partition itself.

3. **Formal guarantees** — Leiden guarantees connected communities, γ-separation, γ-connectivity, and (asymptotically) subset optimality. These are testable properties, not reference partitions.

### 4.2 leidenalg Test Suite

**Source:** [GitHub: vtraag/leidenalg/tests](https://github.com/vtraag/leidenalg/tree/main/tests)

The reference Python implementation's test suite uses:

**From `test_VertexPartition.py`:**

```python
graphs = [
    # Zachary karate network (34 nodes, 78 edges)
    ig.Graph.Famous('Zachary'),
    
    # ER Networks (8 variants): N=100, p=1/100 and 5/100, directed/undirected, loops/noloops
    ig.Graph.Erdos_Renyi(100, p=1./100, directed=False, loops=False),
    ig.Graph.Erdos_Renyi(100, p=5./100, directed=False, loops=False),
    # ... (8 total ER variants)
    
    # Trees (3 variants): N=100, 3 children, undirected/out/in
    ig.Graph.Tree(100, 3, mode='undirected'),
    
    # Lattices (2 variants): [100] nodes, nei=3
    ig.Graph.Lattice([100], nei=3, directed=False, mutual=True, circular=True),
]

# Plus weighted versions of all above (random uniform [0,1] weights)
graphs += [make_weighted(H) for H in graphs]
```

**Total: 30 base graphs (15 unweighted + 15 weighted)**

**From `test_Optimiser.py`:**
- Complete graph K_100
- Complete bipartite K_{50,50}
- Disjoint union of 10 trees (10 nodes each)
- ER(100, p=5/100)

**What the tests verify:** The leidenalg test suite does NOT check for specific reference partitions. Instead, it verifies:
- Quality non-decrease (quality after ≥ quality before)
- Determinism (same seed → same partition)
- Connectedness of communities
- Resolution monotonicity (higher γ → more communities)

### 4.3 igraph Test Suite

**Source:** [GitHub: igraph/igraph/tests](https://github.com/igraph/igraph/tree/main/tests)

The igraph C library has regression and unit tests for community detection, but does not publish specific reference partitions for Leiden. The tests verify:
- Algorithm runs without crashing
- Modularity values are in valid range
- Community structure object is well-formed

### 4.4 Published Modularity Reference Values

From Traag et al. (2019), Table II — maximum modularity over 10 replications × 10 iterations, γ=1:

| Network | Nodes | Leiden Q (reference) | Louvain Q |
|---------|-------|---------------------|-----------|
| DBLP | 317,080 | **0.8387** | 0.8262 |
| Amazon | 334,863 | **0.9341** | 0.9301 |
| IMDB | 374,511 | **0.7069** | 0.7062 |
| LiveJournal | 3,997,962 | **0.7739** | 0.7653 |
| Web of Science | 9,811,130 | **0.7951** | 0.7911 |
| Web UK | 39,252,879 | **0.9801** | 0.9796 |

**Use for Communal:** These values serve as regression targets. A correct Rust implementation should achieve modularity within ~0.001 of these values (the exact value depends on random seed and tie-breaking).

---

## 5. Specific Graph Recommendations

### 5.1 Largest Graph with Known Leiden "Ground Truth"

**LFR Zenodo Set A, N=19,200** — This is the largest standard LFR graph with published ground truth communities. At this size, the graph has:
- ~19,200 nodes
- Average degree 15, 25, or 50
- Mixing parameters from 0.2 to 0.8
- Ground truth partition provided as membership list

**For larger tests:** The CDlib LFR datasets go up to N=100,000 with planted ground truth, but these are generated via NetworkX's LFR implementation (which differs slightly from the original).

### 5.2 Hardest Graph (Most Challenging Community Structure)

**LFR with μ → 0.8** — When the mixing parameter approaches 0.8, nodes have 80% of their edges outside their own community. At this level:
- Community structure is extremely fuzzy
- Even the planted partition has low modularity
- Most algorithms fail to recover the ground truth (NMI → 0)
- Leiden's advantage over Louvain is most pronounced at high μ

**From Traag et al. (2019), Figure 7:** At μ=0.9, N=10⁷, Louvain requires ~2.5 days for first iteration while Leiden needs <10 minutes. This is the hardest case in the paper.

**Recommendation for Communal:** Use LFR with μ=0.7–0.8 as the "hard" test case. At μ=0.9, the signal is so weak that even a correct implementation will produce near-random partitions.

### 5.3 Graphs Where Leiden Outperforms Louvain

From Traag et al. (2019), Leiden shows the clearest advantages over Louvain on:

1. **Amazon (N=334,863):** 23% of Louvain communities are badly connected vs. 0% for Leiden. Q improvement: 0.9301 → 0.9341
2. **DBLP (N=317,080):** 16% badly connected in Louvain. Q improvement: 0.8262 → 0.8387
3. **Web UK (N=39.25M):** 14% badly connected in Louvain. Q improvement: 0.9796 → 0.9801
4. **High-μ LFR benchmarks:** Leiden is 2–100× faster than Louvain at μ > 0.5

**Key insight:** The advantage is largest on graphs with heterogeneous community structure and high mixing parameters — exactly where Louvain produces disconnected communities.

### 5.4 Recommended Test Corpus for Communal

#### Tier 1: Deterministic Reference Graphs (always tested)

| Graph | Nodes | Edges | Known Solution | Test Type |
|-------|-------|-------|----------------|-----------|
| Zachary Karate | 34 | 78 | 2 factions (observed) | Partition comparison |
| Dolphins | 62 | 159 | 2 groups (observed) | Partition comparison |
| Football | 115 | 613 | 12 conferences | Partition comparison |
| PolBooks | 105 | 441 | 3 categories | Partition comparison |
| Les Misérables | 77 | 254 | Character groups | Partition comparison |
| Complete graph K_n | n | n(n-1)/2 | Trivial (1 community) | Property test |
| Complete bipartite K_{n,m} | n+m | nm | 2 communities | Partition test |
| Empty graph | n | 0 | n singletons | Edge case |
| Path graph P_n | n | n-1 | n singletons (Q=0) | Edge case |

#### Tier 2: LFR Benchmarks (ground truth known)

| Configuration | Nodes | μ | Purpose |
|--------------|-------|---|---------|
| LFR N=1000, μ=0.1 | 1,000 | 0.1 | Easy, well-defined communities |
| LFR N=1000, μ=0.5 | 1,000 | 0.5 | Medium difficulty |
| LFR N=5000, μ=0.3 | 5,000 | 0.3 | Scaling test |
| LFR N=10000, μ=0.7 | 10,000 | 0.7 | Hard, fuzzy communities |
| LFR N=19200, μ=0.5 | 19,200 | 0.5 | Largest Zenodo LFR |

#### Tier 3: Real-World Networks (modularity targets)

| Network | Nodes | Reference Q (Leiden) | Source |
|---------|-------|---------------------|--------|
| PolBlogs | 1,490 | ~0.426 | Newman |
| NetScience | 1,589 | ~0.955 | Newman |
| email-Eu-core | 1,005 | ~0.574 | SNAP |
| DBLP | 317,080 | 0.8387 | Traag 2019 |
| Amazon | 334,863 | 0.9341 | Traag 2019 |
| LiveJournal | 3,997,962 | 0.7739 | Traag 2019 |

---

## 6. Downloadable Test Corpora

### 6.1 Direct Download URLs

| Resource | URL | Format | License |
|----------|-----|--------|---------|
| **Zenodo LFR Dataset** | [zenodo.org/records/4450167](https://zenodo.org/records/4450167) | Edge list + JSON ground truth | Open access |
| **SNAP Ground-Truth** | [snap.stanford.edu/data/](https://snap.stanford.edu/data/) | Various (edge list, adjacency) | Free for research |
| **SNAP AGM Datasets** | [snap.stanford.edu/agm/](https://snap.stanford.edu/agm/) | Edge list | Free for research |
| **Community-Graphs** | [github.com/vlivashkin/community-graphs](https://github.com/vlivashkin/community-graphs) | GML | MIT |
| **Newman's Network Data** | [www-personal.umich.edu/~mejn/netdata/](http://www-personal.umich.edu/~mejn/netdata/) | Pajek .net | Public domain |
| **CDlib LFR Datasets** | Via `cdlib.datasets.fetch_network_data()` | NetworkX graph | BSD-2 |
| **CDlib Real-World** | Via `cdlib.datasets.fetch_network_ground_truth()` | NetworkX graph | Various |
| **KONECT** | [konect.cc/](http://konect.cc/) | Various | Free for research |
| **Network Repository** | [networkrepository.com/](https://networkrepository.com/) | Various | Free for research |

### 6.2 File Formats

| Format | Extension | Description | Parsing Complexity |
|--------|-----------|-------------|-------------------|
| **Edge list** | `.edges`, `.txt` | One edge per line: `node1 node2 [weight]` | Trivial |
| **GML** | `.gml` | Graph Modularity Language (key-value) | Moderate |
| **GraphML** | `.graphml` | XML-based graph format | Moderate |
| **Pajek** | `.net` | Pajek network format | Moderate |
| **GEXF** | `.gexf` | Graph Exchange XML Format | Moderate |
| **JSON** | `.json` | Structured data (Zenodo ground truth) | Trivial |
| **Matrix Market** | `.mtx` | Sparse matrix format | Moderate |

### 6.3 Licensing Restrictions

| Dataset | License | Notes |
|---------|---------|-------|
| Zenodo LFR | Open access (CC-BY likely) | Check Zenodo record |
| SNAP | Free for research | Cite the SNAP paper |
| Web of Science | **Restricted** | Cannot be shared; license required |
| Web UK | Free for research | Cite LAW (Univ. Milano) |
| Community-Graphs | MIT | Commercial use allowed |
| KONECT | Free for research | Cite KONECT paper |
| Newman data | Public domain | Cite original sources |

---

## 7. What Cannot Be Verified

### 7.1 No Single "Correct" Partition for Real-World Graphs

For real-world graphs (DBLP, Amazon, LiveJournal, etc.), there is no single "proven Leiden solution." The Leiden algorithm is stochastic — different random seeds produce different partitions (all with similar modularity). The published modularity values are **upper bounds** from multiple replications, not exact targets.

### 7.2 LFR Ground Truth ≠ Optimal Partition

The LFR planted partition is the "ground truth" by construction, but it may not be the modularity-optimal partition. A correct Leiden algorithm may find a partition with higher modularity than the planted one — this is not a bug, it's a feature of modularity optimization.

### 7.3 Implementation Differences

Different implementations of Leiden (igraph C, leidenalg C++/Python, Rust crates) may produce slightly different partitions due to:
- Tie-breaking in node movement order
- Floating-point arithmetic differences
- Random number generator differences
- Subtle differences in the refinement phase

**Recommendation:** Test properties (connectedness, quality non-decrease) rather than exact partition matching.

### 7.4 Scalability Beyond Published Results

The largest published Leiden result is N=10⁷ (Traag et al. 2019, simplified benchmark). For standard LFR, the largest readily available dataset is N=100,000 (CDlib) or N=19,200 (Zenodo). Testing Communal beyond N=10⁷ would be pioneering work with no reference solution.

---

## 8. Source Citations

### Primary Sources

1. **LFR Benchmark Paper:**
   Lancichinetti, A., Fortunato, S., & Radicchi, F. (2008). "Benchmark graphs for testing community detection algorithms." *Physical Review E*, 78, 046110. [DOI: 10.1103/PhysRevE.78.046110](https://doi.org/10.1103/PhysRevE.78.046110) — arXiv:0805.4770

2. **Leiden Algorithm Paper:**
   Traag, V.A., Waltman, L. & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9, 5233. [DOI: 10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z) — arXiv:1810.08473

3. **leidenalg Reference Implementation:**
   [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg) — C++ core with Python bindings. Test suite: `tests/test_VertexPartition.py`, `tests/test_Optimiser.py`

4. **igraph Community Detection:**
   [igraph.org](https://igraph.org/) — C library with Leiden implementation (`igraph_community_leiden()`). [C Manual](https://igraph.org/c/html/1.0.1/igraph-Community.html)

### Datasets & Repositories

5. **Zenodo LFR Dataset:**
   Toth, C. et al. (2021). "A collection of LFR benchmark graphs." [Zenodo](https://zenodo.org/records/4450167) — Pre-generated LFR graphs with ground truth

6. **SNAP Ground-Truth Communities:**
   [snap.stanford.edu/agm/](https://snap.stanford.edu/agm/) — 6 datasets with ground-truth communities (LiveJournal, Friendster, Orkut, Youtube, DBLP, Amazon)

7. **SNAP Data Collection:**
   [snap.stanford.edu/data/](https://snap.stanford.edu/data/) — Stanford Large Network Dataset Collection

8. **CDlib Datasets:**
   [github.com/GiulioRossetti/cdlib_datasets](https://github.com/GiulioRossetti/cdlib_datasets) — Remote repository of network datasets with ground truth

9. **Community-Graphs Repository:**
   [github.com/vlivashkin/community-graphs](https://github.com/vlivashkin/community-graphs) — 41 graphs with ground truth in GML format (MIT license)

10. **Newman's Network Data:**
    [www-personal.umich.edu/~mejn/netdata/](http://www-personal.umich.edu/~mejn/netdata/) — Karate, Dolphins, Football, PolBooks, PolBlogs, Les Misérables, NetScience

11. **KONECT:**
    [konect.cc/](http://konect.cc/) — Koblenz Network Collection, 160+ network datasets

12. **Network Repository:**
    [networkrepository.com/](https://networkrepository.com/) — Interactive network data repository

### Implementation References

13. **libleidenalg (C++ core):**
    [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg) — C++ implementation of Leiden algorithm

14. **leiden-rs (Rust):**
    [github.com/lfgranja/leiden-rs](https://github.com/lfgranja/leiden-rs) — Rust implementation with LFR generator and Criterion benchmarks

15. **fa-leiden-cd (Rust):**
    [github.com/fixed-ai/fa-leiden-cd](https://github.com/fixed-ai/fa-leiden-cd) — Minimal-dependency Rust Leiden implementation

### Secondary Sources

16. **CDlib Documentation:**
    [cdlib.readthedocs.io](https://cdlib.readthedocs.io/en/latest/reference/datasets.html) — Network datasets with annotated communities

17. **Fortunato's LFR Code:**
    [santofortunato.net/resources](https://www.santofortunato.net/resources) — Original LFR benchmark generator code

18. **LFR I/O-Efficient Generation:**
    Boldrin, L. et al. (2016). "I/O-Efficient Generation of Massive Graphs Following the LFR Benchmark." arXiv:1604.08738

---

*End of research document.*
