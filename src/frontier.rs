//! Dominance frontier computation.

use crate::basic_block::BlockId;
use crate::cfg::Cfg;
use crate::dominance::DominanceTree;
use std::collections::{HashMap, HashSet};

/// Dominance frontier for each block.
#[derive(Debug)]
pub struct DominanceFrontier {
    frontiers: HashMap<BlockId, HashSet<BlockId>>,
}

impl DominanceFrontier {
    /// Compute dominance frontiers from a CFG and its dominance tree.
    pub fn compute(cfg: &Cfg, dom: &DominanceTree) -> Self {
        let mut frontiers: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();

        for block in cfg.blocks() {
            frontiers.insert(block.id, HashSet::new());
        }

        // For each join point (block with multiple predecessors)
        for block in cfg.blocks() {
            if block.predecessors.len() < 2 {
                continue;
            }
            for &pred in &block.predecessors {
                let mut runner = pred;
                loop {
                    if !dom.dominates(runner, block.id) {
                        if let Some(df) = frontiers.get_mut(&runner) {
                            df.insert(block.id);
                        }
                    }
                    match dom.idom(runner) {
                        Some(parent) if parent != runner => runner = parent,
                        _ => break,
                    }
                }
            }
        }

        Self { frontiers }
    }

    /// Get the dominance frontier set for a block.
    pub fn frontier_set(&self, b: BlockId) -> &HashSet<BlockId> {
        self.frontiers.get(&b).unwrap_or_else(|| {
            static EMPTY: std::sync::OnceLock<HashSet<BlockId>> = std::sync::OnceLock::new();
            EMPTY.get_or_init(HashSet::new)
        })
    }

    /// Check if block `b` is in the dominance frontier of `a`.
    pub fn in_frontier(&self, a: BlockId, b: BlockId) -> bool {
        self.frontier_set(a).contains(&b)
    }

    /// Total number of frontier entries.
    pub fn total_size(&self) -> usize {
        self.frontiers.values().map(|s| s.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::Cfg;

    #[test]
    fn test_simple_frontier() {
        let mut cfg = Cfg::new();
        let b1 = cfg.add_block();
        let b2 = cfg.add_block();
        let b3 = cfg.add_block();
        cfg.add_edge(BlockId::ENTRY, b1);
        cfg.add_edge(BlockId::ENTRY, b2);
        cfg.add_edge(b1, b3);
        cfg.add_edge(b2, b3);

        let dom = DominanceTree::compute(&cfg);
        let df = DominanceFrontier::compute(&cfg, &dom);

        // b3 is a join point, should be in the frontier of b1 and b2
        assert!(df.in_frontier(b1, b3));
        assert!(df.in_frontier(b2, b3));
    }

    #[test]
    fn test_linear_no_frontier() {
        let mut cfg = Cfg::new();
        let b1 = cfg.add_block();
        cfg.add_edge(BlockId::ENTRY, b1);

        let dom = DominanceTree::compute(&cfg);
        let df = DominanceFrontier::compute(&cfg, &dom);

        assert_eq!(df.total_size(), 0);
    }

    #[test]
    fn test_frontier_set() {
        let mut cfg = Cfg::new();
        let b1 = cfg.add_block();
        let b2 = cfg.add_block();
        let merge = cfg.add_block();
        cfg.add_edge(BlockId::ENTRY, b1);
        cfg.add_edge(BlockId::ENTRY, b2);
        cfg.add_edge(b1, merge);
        cfg.add_edge(b2, merge);

        let dom = DominanceTree::compute(&cfg);
        let df = DominanceFrontier::compute(&cfg, &dom);

        assert!(!df.frontier_set(b1).is_empty());
    }
}
