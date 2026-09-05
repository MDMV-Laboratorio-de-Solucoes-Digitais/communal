# Forward Compatibility Patterns for Deferred Features in Rust Crates

## A Research Report on Graph Algorithm Libraries and Trait Design

---

## Table of Contents

1. [Introduction](#introduction)
2. [The Sealed Trait Pattern](#the-sealed-trait-pattern)
3. [Best Practices for Forward-Compatible Trait Design](#best-practices-for-forward-compatible-trait-design)
4. [Case Study: petgraph's Trait Architecture](#case-study-petgraphs-trait-architecture)
5. [Case Study: ndarray's Trait Architecture](#case-study-ndarrays-trait-architecture)
6. [The Frozen Pattern: Interior Mutability for Graphs](#the-frozen-pattern-interior-mutability-for-graphs)
7. [Summary of Patterns and Recommendations](#summary-of-patterns-and-recommendations)
8. [Citations](#citations)

---

## Introduction

Designing public traits in Rust that remain stable across crate versions—while still allowing the addition of features like node insertion, deletion, or new traversal modalities—is one of the hardest problems in API design. Rust's trait system is "open" by default: any public trait can be implemented by downstream code, which means adding a method with a default implementation can still be a breaking change if a downstream impl also defines a method with the same name.

This report examines the primary patterns Rust crates use to achieve forward compatibility, focusing on graph algorithm libraries (petgraph) and n-dimensional array libraries (ndarray) as case studies. These libraries were chosen because they represent mature, production-grade designs where trait stability and extensibility are critical.

---

## The Sealed Trait Pattern

### What Is a Sealed Trait?

A sealed trait is a public trait that cannot be implemented by downstream crates. This is accomplished by making the public trait extend a private supertrait that is inaccessible outside the defining crate. Since downstream crates cannot name or implement the private supertrait, they cannot implement the public trait either.

The Rust API Guidelines formalize this as rule **C-SEALED** ("Sealed traits protect against downstream implementations")[^1][^2].

### How It Works Mechanically

From the official API guidelines source[^3]:

```rust
/// This trait is sealed and cannot be implemented for types outside this crate.
pub trait TheTrait: private::Sealed {
    // Zero or more methods that the user is allowed to call.
    fn ...();
    // Zero or more private methods, not allowed for user to call.
    #[doc(hidden)]
    fn ...();
}

// Implement for some types.
impl TheTrait for usize {
    /* ... */
}

mod private {
    pub trait Sealed {}
    // Implement for those same types, but no others.
    impl Sealed for usize {}
}
```

The empty private `Sealed` supertrait cannot be named by downstream crates, so implementations of `Sealed` (and therefore `TheTrait`) only exist in the current crate. This allows the crate author to:

- **Add new methods** to `TheTrait` in a non-breaking release (even though that would ordinarily be a breaking change for unsealed traits, due to potential downstream name conflicts)
- **Change the signature** of `#[doc(hidden)]` methods freely

Note that removing a public method or changing the signature of a public, documented method is still a breaking change even for sealed traits.

### The `sealed` Crate

The `sealed` crate (v0.7, MSRV 1.7.0) provides a convenient `#[sealed]` procedural macro that automates this pattern[^4]:

```rust
#[sealed]
trait T {}

pub struct A;
#[sealed]
impl T for A {}

pub struct B;
#[sealed]
impl T for B {}

pub struct C;
impl T for C {} // compile error — trait is sealed
```

The attribute generates a private uniquely-named module when attached to a trait definition:

```rust
// What #[sealed] generates:
trait T: __seal_t::Sealed {}
mod __seal_t {
    pub trait Sealed {}
}
```

The crate also supports `pub(crate)` or `pub(in some::path)` visibility tuning for cases where the trait and its implementations are in different modules. Plain `pub` visibility is explicitly disallowed—it would break the sealing.

### Partial Sealing

The pattern can be extended to "partially sealed" traits, where some methods are sealed but the trait itself remains implementable. This is done by sealing specific method implementations or using sealed supertraits selectively[^5].

---

## Best Practices for Forward-Compatible Trait Design

### 1. Layered Trait Hierarchy

Rather than one monolithic trait, define a hierarchy of small, focused traits where each layer adds capability. Downstream code depends only on the layer it needs. New layers can be added without breaking existing code.

### 2. Sealed Foundation Traits

Seal the foundational traits that serve as bounds throughout your crate. This allows you to evolve those traits freely. Higher-level extension traits can remain unsealed if user implementability is desired.

### 3. Associated Types for Future Extension

Use associated types (e.g., `type NodeId`, `type Map`) rather than concrete types in trait method signatures. Associated types can be added as new associated types to traits without breaking existing method signatures.

### 4. Newtypes to Hide Implementation Details

Wrap complex types in newtypes so the exact representation is not part of the public API. This is the C-NEWTYPE-HIDE guideline[^1]. The client's view is simplified and the internal representation can change without breaking code.

### 5. `#[doc(hidden)]` Methods for Internal APIs

Methods that are part of the trait's contract but not intended for direct user call can be marked `#[doc(hidden)]`. For sealed traits, these method signatures can be changed freely in non-breaking releases.

### 6. Private Fields in Public Structs

Struct fields should be private (C-STRUCT-PRIVATE guideline)[^1]. This allows the internal representation to evolve, add validation, or maintain invariants—all without breaking downstream code that accesses data through getter/setter methods.

---

## Case Study: petgraph's Trait Architecture

### Overview

petgraph is a graph data structure library that provides multiple graph types (`Graph`, `StableGraph`, `GraphMap`, `MatrixGraph`, `Csr`, `List`) with different performance tradeoffs[^6][^7]. Its trait system is designed so that algorithms are generic over any graph type that implements the required traits.

### The Trait Hierarchy

petgraph uses a layered trait hierarchy in the `petgraph::visit` and `petgraph::data` modules. The foundational trait is[^8][^9]:

```rust
pub trait GraphBase {
    type EdgeId: Copy + PartialEq;
    type NodeId: Copy + PartialEq;
}
```

`GraphBase` defines only the associated identifier types—nothing more. This is the minimal requirement for any graph-like structure. Every other graph trait extends `GraphBase`:

- **`Data`** — Defines associated `NodeWeight` and `EdgeWeight` types[^10]:
  ```rust
  pub trait Data: GraphBase {
      type NodeWeight;
      type EdgeWeight;
  }
  ```

- **`DataMap`** — Read access to node/edge weights[^11]:
  ```rust
  pub trait DataMap: Data {
      fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight>;
      fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight>;
  }
  ```

- **`DataMapMut`** — Mutable access to weights (extends `DataMap`)[^12]:
  ```rust
  pub trait DataMapMut: DataMap {
      fn node_weight_mut(&mut self, id: Self::NodeId) -> Option<&mut Self::NodeWeight>;
      fn edge_weight_mut(&mut self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight>;
  }
  ```

- **`Visitable`** — Creates visit-maps for graph traversal[^13]:
  ```rust
  pub trait Visitable: GraphBase {
      type Map: VisitMap<Self::NodeId>;
      fn visit_map(&self) -> Self::Map;
      fn reset_map(&self, map: &mut Self::Map);
  }
  ```

- **`Build`** — Adding nodes and edges[^14]:
  ```rust
  pub trait Build: Data + NodeCount {
      fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId;
      fn update_edge(&mut self, a: Self::NodeId, b: Self::NodeId,
                     weight: Self::EdgeWeight) -> Self::EdgeId;
      // Provided method:
      fn add_edge(...) -> Option<Self::EdgeId> { ... }
  }
  ```

- **`Create`** — Constructing graphs with capacity[^15]:
  ```rust
  pub trait Create: Build + Default {
      fn with_capacity(nodes: usize, edges: usize) -> Self;
  }
  ```

- **`NodeCount`**, **`EdgeCount`**, **`NodeIndexable`**, **`EdgeIndexable`**, **`NodeCompactIndexable`** — Fine-grained capability traits for algorithms that need specific access patterns.

- **`IntoNeighbors`**, **`IntoNeighborsDirected`**, **`IntoEdges`**, **`IntoEdgesDirected`**, **`IntoNodeIdentifiers`**, **`IntoNodeReferences`**, **`IntoEdgeReferences`** — The `Into-*` iterator-producing traits[^16].

### Key Design Insights

1. **Loose coupling by design**: The petgraph documentation explicitly states: "The traits are rather loosely coupled at the moment (which is intentional, but will develop a bit), and there are traits missing that could be added."[^16] This is a deliberate forward-compatibility choice—new traits can be introduced without breaking existing ones.

2. **Minimum viable bounds**: Algorithms specify minimum trait bounds. For example, a DFS traversal requires only `GraphBase + IntoNeighbors + Visitable`. Algorithms do not require `Build` or `Create` unless they mutate structure.

3. **Node/edge insertion as a separate capability**: The `Build` trait (with `add_node` and `add_edge`/`update_edge`) is distinct from read-only traits. Algorithms that do not need to mutate graph structure are not burdened by `Build` bounds. If petgraph were to add `remove_node` or `remove_edge`, they would naturally go into a new trait (or extend `Build`), which would be a non-breaking change for algorithms that do not require it.

4. **Blanket implementations for references**: petgraph provides `impl<G: GraphBase> GraphBase for &G` and `impl<G: Data> Data for &G` etc., so borrowed graphs automatically implement the same traits.

5. **`Frozen` as an adapter**: The `Frozen<'a, G>` wrapper provides read-only access to graph structure while allowing mutable access to weights. It implements all the same traits as `G` (via `Deref`), making it a transparent adapter for algorithms[^17].

### How petgraph Handles "Deferred Features"

petgraph's trait design naturally accommodates deferred features:

- **Node removal**: Not part of any public trait (only available directly on `Graph` and `StableGraph` structs). If added to a trait, it would go into a new trait or extend `Build`.
- **Edge filtering and node filtering**: Implemented via `EdgeFiltered` and `NodeFiltered` adapter types that wrap any graph and implement the same trait hierarchy[^16].
- **Graph reversal**: The `Reversed<G>` adapter implements all graph traits[^16].
- **Acyclic graph wrapper**: `Acyclic<G>` implements `Build`, `Data`, etc., with cycle detection at insertion time.

---

## Case Study: ndarray's Trait Architecture

### Overview

ndarray provides an n-dimensional container for general elements and numerics[^18]. Its trait design centers on the `ArrayBase` type, which is used to implement both owned arrays and views[^19].

### The Data Trait Hierarchy

ndarray's representation traits form a layered hierarchy[^20][^21]:

```rust
pub unsafe trait RawData: Sized {
    type Elem;
}
```

`RawData` is the foundation—it defines only the element type. It does not imply any ownership or lifetime; pointers to elements may not be safe to dereference. As the docs state: *"Note: `RawData` is not an extension interface at this point. Traits in Rust can serve many different roles. This trait is public because it is used as a bound on public methods."*[^20]

```rust
pub unsafe trait Data: RawData { }
```

`Data` is a marker for representations where elements can be accessed with safe code. It is documented as *"Internal trait, see `RawData`."*[^21]

```rust
pub unsafe trait DataMut: Data { }       // Mutable element access
pub unsafe trait DataOwned: Data { }      // Owns its elements
pub unsafe trait DataShared: Data { }     // Shared ownership (copy-on-write)
```

### The Dimension Trait

The `Dimension` trait in ndarray is particularly relevant because it demonstrates the "sealed by documentation" pattern[^22]:

```rust
pub trait Dimension: Clone + Eq + Debug + Send + Sync + Default + ... {
    type Pattern: IntoDimension<Dim = Self> + Clone + Debug + PartialEq + Eq + Default;
    type Smaller: Dimension;
    type Larger: Dimension + RemoveAxis;
    const NDIM: Option<usize>;
    // ...
}
```

The docs explicitly state: *"Note: This trait can not be implemented outside the crate."* While not enforced via the sealed supertrait pattern, this is a convention-based seal. The trait's complexity (with associated types `Smaller`, `Larger`, and the `NDIM` constant) makes it effectively impossible to implement correctly outside the crate without knowledge of ndarray's internals.

### Key Design Insights

1. **`RawData` as a public-but-not-extensible foundation**: By making `RawData` public but documenting it as "not an extension interface," ndarray allows the trait to be used as a bound on public methods while signaling that downstream implementations are not supported. This is a softer form of sealing.

2. **Layered data traits for capability differentiation**: `Data → DataMut → DataOwned` allows algorithms to specify exactly what they need. A read-only algorithm requires only `Data`; a mutating algorithm requires `DataMut`.

3. **Internal traits with `#[doc(hidden)]`**: ndarray's documentation notes: *"functions/methods/traits/etc. hidden from the docs are not considered part of the public API, so changes to them are not considered breaking changes."*[^18] This gives the crate freedom to evolve internal abstractions.

4. **Sealed dimension trait**: The `Dimension` trait cannot be implemented outside the crate (enforced by convention and complexity). This allows ndarray to add new dimension types (e.g., `Ix7`, `Ix8`) or modify the trait without breaking downstream code.

---

## The Frozen Pattern: Interior Mutability for Graphs

petgraph's `Frozen<'a, G>` is a graph wrapper that provides read-only access to the graph's structure while allowing mutable access to its node and edge weights[^17][^23]:

```rust
pub struct Frozen<'a, G: 'a>(/* private fields */);
```

It works by holding a mutable reference to the graph (`&'a mut G`) and exposing all read-only graph operations through `Deref`, while also providing `IndexMut` for weight mutation. As the documentation explains:

> "The `Frozen` only allows shared access (read-only) to the underlying graph `G`, but it allows mutable access to its node and edge weights. This is used to ensure immutability of the graph's structure while permitting weights to be both read and written."[^23]

This pattern is directly relevant to forward compatibility for deferred features because:

1. **It separates structural mutation from weight mutation**: Algorithms that need to mutate weights but not structure can accept `&Frozen<G>` rather than `&mut G`. This is how petgraph's `Dfs` walker works—it uses `.next()` on a shared graph reference.

2. **It implements all the same traits as `G`**: Through `Deref` to `G` and explicit trait impls, `Frozen<G>` implements `GraphBase`, `Data`, `DataMap`, `DataMapMut`, `Visitable`, `IntoNeighbors`, etc. Any algorithm generic over these traits works with `Frozen<G>` without modification.

3. **Interior mutability for traversal state**: The `Frozen` wrapper enables patterns like `index_twice_mut` that allow simultaneous mutable access to two different weights (e.g., a node weight and an edge weight during DFS traversal) without violating Rust's borrow rules.

---

## Summary of Patterns and Recommendations

### For Designing Forward-Compatible Graph Algorithm Traits

| Pattern | What It Enables | Example |
|---------|----------------|---------|
| **Sealed traits** | Add methods without breaking downstream | `IndexType` in petgraph (unsafe trait, sealed by convention) |
| **Layered hierarchy** | Add new capability traits without breaking old ones | `Data` → `DataMap` → `DataMapMut` → `Build` → `Create` |
| **Minimum bounds** | Algorithms require only what they need | `Dfs` requires only `GraphBase + IntoNeighbors + Visitable` |
| **Adapter types** | Add functionality without changing core types | `Frozen<G>`, `Reversed<G>`, `NodeFiltered<G>`, `EdgeFiltered<G>` |
| **Associated types** | Extend type-level behavior without breaking signatures | `type NodeId`, `type Map`, `type NodeWeight` in petgraph |
| **Internal/doc-hidden traits** | Evolve internal abstractions freely | `RawData` in ndarray (documented as "not an extension interface") |
| **Newtype wrappers** | Hide implementation details | `Frozen<G>` wraps `&mut G` with controlled access |

### Concrete Recommendations for Graph Library Authors

1. **Start with a minimal `GraphBase` trait** that defines only identifier types. Seal it if you want freedom to evolve it. In petgraph, `GraphBase` is not formally sealed but is treated as a crate-internal extension point.

2. **Keep read-only and write capabilities separate.** `Data` (weights) should be separate from `Build` (structure mutation). This means a future `Remove` trait for node/edge deletion can be added without affecting algorithms that only read.

3. **Use the `Into-*` pattern for iteration.** petgraph's `IntoNeighbors`, `IntoEdges`, etc. follow the same pattern as `IntoIterator`—they take `&self` and produce iterators. This is extensible: new iteration modalities (e.g., `IntoWeightedEdges`, `IntoBfsOrder`) can be added as new traits.

4. **Consider a `Frozen`-style wrapper** for interior mutability. This enables shared references during traversal while allowing weight mutation—critical for many graph algorithms (e.g., marking visited nodes, accumulating path weights).

5. **Prefer trait bounds over concrete types** in generic algorithms. An algorithm written for `G: GraphBase + IntoNeighbors + Visitable` works with `Graph`, `StableGraph`, `GraphMap`, `Csr`, `Frozen<G>`, `Reversed<G>`, `NodeFiltered<G>`, and any future graph type.

6. **Document your sealing intentions.** Whether using the formal sealed supertrait pattern (C-SEALED) or the convention-based approach (as with ndarray's `Dimension`), document whether a trait is meant for downstream implementation or not.

---

## Citations

[^1]: Rust API Guidelines — Future Proofing (C-SEALED). https://rust-lang.github.io/api-guidelines/future-proofing.html
[^2]: DeepWiki — Sealed Traits and Encapsulation (rust-lang/api-guidelines). https://deepwiki.com/rust-lang/api-guidelines/7.1-sealed-traits-and-encapsulation
[^3]: rust-lang/api-guidelines — `src/future-proofing.md` on GitHub. https://github.com/rust-lang/api-guidelines/blob/master/src/future-proofing.md
[^4]: `sealed` crate (v0.7) — Docs.rs. https://docs.rs/sealed/latest/sealed/
[^5]: Predr — "A definitive guide to sealed traits in Rust." https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/
[^6]: petgraph — Docs.rs. https://docs.rs/petgraph/latest/petgraph/
[^7]: petgraph — GitHub repository. https://github.com/petgraph/petgraph
[^8]: `GraphBase` trait — petgraph::visit. https://docs.rs/petgraph/latest/petgraph/visit/trait.GraphBase.html
[^9]: `GraphBase` module source location. https://docs.rs/petgraph/latest/petgraph/visit/mod.rs.html#87-100
[^10]: `Data` trait — petgraph::visit. https://docs.rs/petgraph/latest/petgraph/visit/trait.Data.html
[^11]: `DataMap` trait — petgraph::data. https://docs.rs/petgraph/latest/petgraph/data/trait.DataMap.html
[^12]: `DataMapMut` trait — petgraph::data. https://docs.rs/petgraph/latest/petgraph/data/trait.DataMapMut.html
[^13]: `Visitable` trait — petgraph::visit. https://docs.rs/petgraph/latest/petgraph/visit/trait.Visitable.html
[^14]: `Build` trait — petgraph::data. https://docs.rs/petgraph/latest/petgraph/data/trait.Build.html
[^15]: `Create` trait — petgraph::data. https://docs.rs/petgraph/latest/petgraph/data/trait.Create.html
[^16]: `petgraph::visit` module documentation. https://docs.rs/petgraph/latest/petgraph/visit/index.html
[^17]: `Frozen` struct — petgraph::graph. https://docs.rs/petgraph/latest/petgraph/graph/struct.Frozen.html
[^18]: ndarray crate — Docs.rs. https://docs.rs/ndarray/latest/ndarray/
[^19]: `ArrayBase` documentation — ndarray. https://docs.rs/ndarray/latest/struct.ArrayBase.html
[^20]: `RawData` trait — ndarray. https://docs.rs/ndarray/latest/ndarray/trait.RawData.html
[^21]: `Data` trait — ndarray. https://docs.rs/ndarray/latest/ndarray/trait.Data.html
[^22]: `Dimension` trait — ndarray. https://docs.rs/ndarray/latest/ndarray/trait.Dimension.html
[^23]: `Frozen` documentation quote. https://docs.rs/petgraph/latest/petgraph/graph/struct.Frozen.html
