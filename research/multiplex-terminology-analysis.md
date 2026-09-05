# Multiplex vs. Multilayer Terminology Analysis

**Date**: 2026-03-08  
**Scope**: Community detection framework (Communal) — determine canonical terminology for trait naming and constitution language  
**Author**: Research analysis  

---

## Executive Summary

**The terms "multiplex" and "multilayer" are NOT interchangeable in the network science literature.** "Multilayer network" is the broad umbrella term; "multiplex network" is a specific constrained subtype. The framework should standardize on **"multilayer"** as the primary term, and the sealed marker trait should be renamed from `MultiplexView` to **`MultilayerView`**. This aligns with primary-source literature, is forward-compatible with the v1.1 roadmap (which will support more than just multiplex topologies), and follows the principle that trait names should describe the most general concept they represent.

---

## 1. The Literature Distinction

### 1.1 Kivelä et al. (2014) — The Definitive Taxonomy

The most-cited reference on this topic is:

> Kivelä, M., Arenas, A., Barthelemy, M., Gleeson, J. P., Moreno, Y., & Porter, M. A. (2014). Multilayer networks. *Journal of Complex Networks*, 2(3), 203–271. https://doi.org/10.1093/comnet/cnu016

This paper establishes the definitive hierarchy. Key quotations:

> *"In the rest of this paper, we will use the term multiplex network in a manner that is similar to Ref. [103]: we consider all diagonally coupled multilayer networks in which each layer shares at least one node with some other layer in the network to be multiplex networks. That is, we include multilayer networks that are not node-aligned in our definition of multiplex networks, but we leave out layer-disjoint networks."* (Section 2.5)

> *"Additionally, this definition of multiplex networks also includes multilayer networks with more than one aspect, because it is natural to represent certain multiplex structures using multilayer networks with more than one type (i.e. aspect) of layer."* (Section 2.5)

The paper's **Figure 1** explicitly visualizes the relationship: multiplex networks are a *subset* of multilayer networks, which are themselves a subset of all possible network structures. The mapping is injective but not surjective — every multiplex network is a multilayer network, but not every multilayer network is a multiplex network.

**Constraints that define a multiplex network** (from Table 1):
- **Node-aligned** (or at least partially shared nodes): layers share nodes
- **Diagonal couplings**: inter-layer edges only connect a node to its replicas in other layers
- **Not layer-disjoint**: each node exists in at least one layer; each layer shares at least one node with another

### 1.2 De Domenico et al. (2013) — Tensor Formalism

> De Domenico, M., Solé-Ribalta, A., Cozzo, E., Kivelä, M., Moreno, Y., Porter, M. A., Gómez, S., & Arenas, A. (2013). Mathematical formulation of multilayer networks. *Physical Review X*, 3(4), 041022. https://doi.org/10.1103/PhysRevX.3.041022

This paper introduces the tensor formalism using "multi-layer" as the general framework and explicitly defines multiplex as a special case:

> *"In this paper, we focus on multiplex networks. A multiplex network is a special type of multi-layer network in which the only possible types of inter-layer connections are ones in which a node is connected to its counterpart nodes in the other layers."* (Section III)

> *"Multiplex networks explicitly incorporate multiple channels of connectivity in a system, and they provide a natural description for systems in which entities have a different set of neighbors in each layer (which can represent, e.g., a task, an activity, or a category)."* (Introduction)

### 1.3 Baptista et al. (2022) — Contemporary Usage

> Baptista, A., Gonzalez, A., & Baudot, A. (2022). Universal multilayer network exploration by random walk with restart. *Communications Physics*, 5, 170. https://doi.org/10.1038/s42005-022-00937-9

This recent Nature Communications Physics paper reinforces the hierarchy:

> *"For instance, multiplex networks are multilayer networks composed of different layers containing the same nodes (called replica nodes) but different types of edges, and thereby different topologies."* (Introduction)

### 1.4 Boccaletti et al. (2014) — Physics Reports Review

> Boccaletti, S., Bianconi, G., Criado, R., del Amo, A. J., Gómez-Gardeñes, J., Romance, M., Sendiña-Nadal, I., Wang, Z., & Zanin, M. (2014). The structure and dynamics of multilayer networks. *Physics Reports*, 544(1), 1–122. https://doi.org/10.1016/j.physrep.2014.07.001

This 122-page review consistently uses "multilayer networks" as the overarching concept and treats "multiplex" as a specific case within that framework.

### 1.5 Wikipedia / Community Consensus

The Wikipedia article on "Multidimensional network" (which redirects from "Multilayer network") states:

> *"The rapid exploration of complex networks in recent years has been dogged by a lack of standardized naming conventions, as various groups use overlapping and contradictory terminology to describe specific network configurations (e.g., multiplex, multilayer, multilevel, multidimensional, multirelational, interconnected)."*

Despite historical inconsistency, the **current consensus** in the literature (post-2014) is clear: **multilayer = umbrella term; multiplex = specific subtype**.

---

## 2. The Formal Relationship

```
All network structures
  └── Multilayer networks (general: multiple layers, arbitrary constraints)
        ├── Multiplex networks (node-aligned, diagonal couplings)
        │     ├── Categorical multiplex (all layer pairs coupled)
        │     ├── Ordinal multiplex (only adjacent layers coupled, e.g. temporal)
        │     └── General multiplex (arbitrary coupling pattern, still diagonal)
        ├── Interdependent networks (layer-disjoint, dependency edges)
        ├── Networks of networks (layer-disjoint, arbitrary inter-layer edges)
        ├── Temporal networks (ordered layers, often ordinal multiplex)
        └── Heterogeneous networks (different node types across layers)
```

**Key insight**: The common thread across the literature is that "multiplex" implies **the same entities exist across all layers** (with replicas/counterparts). This is the property that makes multiplex community detection algorithms work — a community can span layers because the same node exists in multiple layers. This is also why multiplex algorithms are the natural first step for v1.1: they extend single-layer algorithms in the most straightforward way.

---

## 3. The Communal Framework Context

### 3.1 Current Usage

| Artifact | Current Term | Location |
|----------|-------------|----------|
| Constitution | "Multiplex/multilayer networks" (interchangeable) | Principle I, item 5 |
| Spec FR-035 | "multiplex/multilayer network support" | Line 307 |
| Spec FR-043 | `MultiplexView: GraphView` | Line 315 |
| Spec FR-035 | "sealed `MultiplexView` marker trait" | Line 307 |
| Assumption | "multiplex/multilayer network implementation" | Line 555 |
| Clarification | "MultiplexView: GraphView — multiplex graphs are always graphs" | Session 2026-09-04 |

### 3.2 The Inconsistency

The constitution uses "multiplex/multilayer" as if they were synonyms. This is the core problem. While many papers in the literature DO use them interchangeably (especially pre-2014), the **semantically correct** and **forward-compatible** choice is to treat "multilayer" as the umbrella term.

---

## 4. Rust Ecosystem / petgraph Community

### 4.1 petgraph

The petgraph crate (v0.8) does not natively implement multiplex or multilayer networks. Its trait hierarchy (`GraphBase`, `Data`, `Visitable`, `IntoNeighbors`, etc.) is designed for single-layer graphs. However, the trait architecture is **extensible** — a `MultilayerView` could be built as a new trait that wraps multiple `GraphView` instances with inter-layer coupling information.

The BRAPH (Brain Analysis Graph) project's documentation uses the precise distinction:

> *"A multiplex graph is a type of multilayer graph where only interlayer edges are allowed between homologous nodes."*

### 4.2 Python Ecosystem

- **pymnet** (Python library for multilayer networks): Uses "multilayer" as the primary term, with "multiplex" as a specific network type. Based on Kivelä et al. (2014).
- **multinet** (C++ library with R bindings): Uses "multilayer" throughout.

### 4.3 Sealed Trait Pattern in Rust

The Rust API Guidelines (C-SEALED) describe the sealed trait pattern but **do not mandate any particular naming** for the sealed trait itself. The convention is:

```rust
pub trait TheTrait: private::Sealed {
    // methods
}

mod private {
    pub trait Sealed {}
    impl Sealed for SomeType {}
}
```

The name of the sealed trait reflects its **semantic purpose**, not the fact that it is sealed. The `Sealed` supertrait is the mechanism; the public trait name should describe what it represents.

**Real-world examples from the Rust ecosystem:**
- `serde_json::value::Index` — sealed trait for indexing JSON values
- `byteorder::ByteOrder` — sealed trait for byte order operations
- `std::alloc::Allocator` (nightly) — uses sealing mechanism

None of these include "Sealed" in their public name. The sealing is an implementation detail.

---

## 5. Recommendation

### 5.1 Standardize on "Multilayer"

**The framework should standardize on "multilayer" as the primary term**, with "multiplex" used only when specifically referring to the node-aligned subtype.

**Rationale:**
1. **Literature alignment**: Kivelä et al. (2014), De Domenico et al. (2013), Boccaletti et al. (2014), and Baptista et al. (2022) all establish "multilayer" as the umbrella term.
2. **Forward compatibility**: The v1.1 roadmap defers "full multiplex/multilayer" support. If the framework later supports interdependent networks, networks of networks, or temporal networks with different node sets, "multilayer" is the correct umbrella term. "Multiplex" would be inaccurate for those cases.
3. **Trait semantics**: The trait `MultiplexView: GraphView` currently exists as a marker for "this graph has multiple layers." Since it will eventually cover all multilayer types (not just multiplex), the name should be `MultilayerView`.
4. **Community detection context**: Most multiplex community detection algorithms (Mucha et al., 2010; De Domenico et al., 2014) are actually algorithms for a specific type of multilayer network. Referring to them as "multilayer community detection" is more accurate and future-proof.

### 5.2 Rename `MultiplexView` → `MultilayerView`

The sealed marker trait should be renamed:

```rust
// BEFORE (imprecise — implies only multiplex topology)
pub trait MultiplexView: GraphView + private::Sealed {
    // placeholder for v1.1
}

// AFTER (correct — covers all multilayer topologies)
pub trait MultilayerView: GraphView + private::Sealed {
    // placeholder for v1.1
}
```

**Impact:**
- The trait is currently a **sealed placeholder** with no methods (per FR-035). Renaming has zero implementation cost.
- Downstream crates cannot implement it (sealed), so the rename is non-breaking for implementors.
- Only internal references in `communal-core` and the spec need updating.

### 5.3 Update the Constitution

The constitution should be updated to use "multilayer" consistently, with an optional parenthetical note clarifying the multiplex subtype:

**Current (Principle I, item 5):**
> *Complex Topologies*: Multiplex/multilayer networks (deferred to v1.1; core traits designed for forward compatibility via sealed `MultiplexView` marker trait)

**Proposed:**
> *Complex Topologies*: Multilayer networks (including multiplex topologies as a special case; deferred to v1.1; core traits designed for forward compatibility via sealed `MultilayerView` marker trait)

### 5.4 When to Use "Multiplex"

"Multiplex" should be used specifically when referring to:
- Node-aligned multilayer networks (same nodes in all layers)
- Diagonal inter-layer couplings (a node connected to its replicas)
- The specific algorithms designed for this case (e.g., Mucha et al.'s multislice modularity)

Example: *"The v1.1 release will first implement multiplex community detection (node-aligned layers with diagonal couplings), with general multilayer support following in a later release."*

---

## 6. Required Edits

### Constitution (`.specify/memory/constitution.md`)

| Line | Current | Proposed |
|------|---------|----------|
| 19 | `Multiplex/multilayer networks (deferred to v1.1; core traits designed for forward compatibility via sealed ``MultiplexView`` marker trait)` | `Multilayer networks (including multiplex topologies; deferred to v1.1; core traits designed for forward compatibility via sealed ``MultilayerView`` marker trait)` |

### Spec (`specs/001-community-detection/spec.md`)

| Line | Current | Proposed |
|------|---------|----------|
| 34 (Clarification) | ``MultiplexView: GraphView`` | ``MultilayerView: GraphView`` |
| 307 (FR-035) | `multiplex/multilayer network support` / ``sealed ``MultiplexView`` marker trait`` | `multilayer network support` / ``sealed ``MultilayerView`` marker trait`` |
| 315 (FR-043) | ``MultiplexView: GraphView`` | ``MultilayerView: GraphView`` |
| 555 (Assumption) | `Full multiplex/multilayer network implementation is deferred to v1.1` | `Full multilayer network implementation is deferred to v1.1` |

### Contracts (`specs/001-community-detection/contracts/core-traits.md`)

The `MultiplexView` trait definition should be renamed to `MultilayerView` (currently implied by FR-043 but not yet explicitly defined in the contract — the contract defines `GraphView`, `CommunityDetector`, `PartitionResult`, `DynamicGraph`, `AlgorithmConfig`).

---

## 7. Sources

### Primary Literature

1. **Kivelä, M., Arenas, A., Barthelemy, M., Gleeson, J. P., Moreno, Y., & Porter, M. A.** (2014). Multilayer networks. *Journal of Complex Networks*, 2(3), 203–271. https://doi.org/10.1093/comnet/cnu016 — **Definitive taxonomy paper.** Establishes "multilayer" as the umbrella term and "multiplex" as a constrained subtype. Contains the definitive Figure 1 and Table 1 mapping all network types.

2. **De Domenico, M., Solé-Ribalta, A., Cozzo, E., Kivelä, M., Moreno, Y., Porter, M. A., Gómez, S., & Arenas, A.** (2013). Mathematical formulation of multilayer networks. *Physical Review X*, 3(4), 041022. https://doi.org/10.1103/PhysRevX.3.041022 — Introduces tensor formalism. Uses "multi-layer" as the general framework and explicitly defines multiplex as a special case: *"A multiplex network is a special type of multi-layer network in which the only possible types of inter-layer connections are ones in which a node is connected to its counterpart nodes in the other layers."*

3. **Baptista, A., Gonzalez, A., & Baudot, A.** (2022). Universal multilayer network exploration by random walk with restart. *Communications Physics*, 5, 170. https://doi.org/10.1038/s42005-022-00937-9 — Recent paper reinforcing the hierarchy: *"multiplex networks are multilayer networks composed of different layers containing the same nodes (called replica nodes) but different types of edges."*

4. **Boccaletti, S., Bianconi, G., Criado, R., del Amo, A. J., Gómez-Gardeñes, J., Romance, M., Sendiña-Nadal, I., Wang, Z., & Zanin, M.** (2014). The structure and dynamics of multilayer networks. *Physics Reports*, 544(1), 1–122. https://doi.org/10.1016/j.physrep.2014.07.001 — 122-page comprehensive review using "multilayer" as the overarching concept.

5. **Mucha, P. J., Richardson, T., Macon, K., Porter, M. A., & Onnela, J.-P.** (2010). Community structure in time-dependent, multiscale, and multiplex networks. *Science*, 328(5980), 876–878. https://doi.org/10.1126/science.1184819 — Seminal multiplex community detection paper. Uses the term "multislice" (equivalent to node-aligned multiplex), establishing the algorithmic foundation.

6. **De Domenico, M., Solé-Ribalta, A., Gómez, S., & Arenas, A.** (2014). Navigability of interconnected networks under random failures. *Proceedings of the National Academy of Sciences*, 111(23), 8351–8356. https://doi.org/10.1073/pnas.1318469111

### Rust Ecosystem

7. **Rust API Guidelines — Future Proofing (C-SEALED).** https://rust-lang.github.io/api-guidelines/future-proofing.html — Official documentation of the sealed trait pattern. Confirms that sealed traits should describe their semantic purpose, not include "Sealed" in their public name.

8. **petgraph** — Graph data structure library. https://docs.rs/petgraph/latest/ — The `petgraph::visit` and `petgraph::data` modules demonstrate layered trait design that could be extended with a `MultilayerView` trait.

### Ecosystem Software

9. **pymnet** — Python library for multilayer networks. https://mnets.github.io/pymnet/ — Based on Kivelä et al. (2014). Uses "multilayer" as the primary term, with "multiplex" as a specific network type.

10. **BRAPH** — Brain Analysis Graph. http://braph.org/connectivity-analysis/multilayer-graphs/ — *"A multiplex graph is a type of multilayer graph where only interlayer edges are allowed between homologous nodes."*

---

## 8. Decision Matrix

| Criterion | `MultiplexView` (current) | `MultilayerView` (proposed) |
|-----------|--------------------------|---------------------------|
| **Literature alignment** | ⚠️ Too narrow — "multiplex" is a subset | ✅ Matches umbrella term in Kivelä 2014, De Domenico 2013, Boccaletti 2014 |
| **Forward compatibility** | ❌ Would need rename when adding interdependent/networks-of-networks support | ✅ Covers all future multilayer types without rename |
| **Accuracy for v1.1** | ✅ Accurate for node-aligned multiplex | ✅ Still accurate — multiplex IS a multilayer type |
| **Trait naming convention** | ⚠️ Implies specific constraints in the name | ✅ General concept, constraints defined by supertraits/where-clauses |
| **Constitution alignment** | ⚠️ Perpetuates imprecise interchangeability | ✅ Enables precise language: "multilayer (including multiplex)" |
| **Implementation cost** | N/A | ✅ Zero — trait is a sealed placeholder with no methods |
| **Breaking change** | N/A | ✅ Non-breaking — trait is sealed (no downstream impls) |

---

## 9. Conclusion

The literature is unambiguous: **"multilayer" is the general term; "multiplex" is a specific constrained subtype.** The Communal framework should adopt this terminology precisely. The sealed marker trait should be named `MultilayerView`, not `MultiplexView`, because:

1. It correctly describes the most general concept (any network with multiple layers).
2. It is forward-compatible with v1.1 and beyond — the trait name won't need to change when non-multiplex multilayer types are added.
3. It aligns with the primary-source literature and the broader network science community.
4. It costs nothing to rename now (the trait is a sealed placeholder with no methods).

The constitution and spec should be updated to use "multilayer" consistently, reserving "multiplex" for the specific case of node-aligned layers with diagonal couplings.
