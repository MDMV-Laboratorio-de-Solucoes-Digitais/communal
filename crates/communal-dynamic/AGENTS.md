# communal-dynamic — Incremental Graph Updates

**Generated:** 2026-09-05

## OVERVIEW

Efficient incremental updates to community partitions when the underlying graph changes — without recomputing from scratch.

## STRUCTURE

```
src/
├── error.rs      # Dynamic operation errors
├── hierarchy.rs  # Hierarchical community structure across levels
├── lib.rs        # Crate root
├── mutation.rs   # Edge mutation types for dynamic changes
├── streaming.rs  # Streaming detector trait for incremental updates
└── update.rs     # Incremental update handler (local neighborhood)
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add new mutation type | `mutation.rs` | Edge change classification |
| Implement streaming | `streaming.rs` | Trait for incremental detection |
| Track hierarchy | `hierarchy.rs` | Multi-level community structure |
