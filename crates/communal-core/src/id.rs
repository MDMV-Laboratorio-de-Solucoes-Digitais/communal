use std::num::NonZeroU32;

/// Opaque node identifier using dense contiguous indexing.
///
/// Wraps a `NonZeroU32` so that `Option<NodeId>` is the same size as `u32`,
/// enabling memory-efficient graph representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NodeId(NonZeroU32);

impl NodeId {
    /// Creates a new `NodeId` from a raw index.
    ///
    /// Returns `None` if the index is zero (reserved as the "no node" niche).
    pub fn new(index: u32) -> Option<Self> {
        NonZeroU32::new(index).map(Self)
    }

    /// Returns the internal index as `usize`.
    pub fn index(self) -> usize {
        self.0.get() as usize
    }

    /// Returns the raw `NonZeroU32` value.
    pub fn raw(self) -> NonZeroU32 {
        self.0
    }
}

/// Opaque community identifier.
///
/// Uses the same `NonZeroU32` niche optimization as [`NodeId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct CommunityId(NonZeroU32);

impl CommunityId {
    /// Creates a new `CommunityId` from a raw index.
    ///
    /// Returns `None` if the index is zero.
    pub fn new(index: u32) -> Option<Self> {
        NonZeroU32::new(index).map(Self)
    }

    /// Returns the internal index as `usize`.
    pub fn index(self) -> usize {
        self.0.get() as usize
    }
}
