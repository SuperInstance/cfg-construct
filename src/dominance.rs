//! Dominance analysis.

use crate::basic_block::BlockId;
use crate::cfg::Cfg;
use std::collections::{HashMap, HashSet};

/// Dominance tree computed from a CFG.
#[derive(Debug)]
pub struct DominanceTree {
    /// Immediate dominator for each block.
    idom: HashMap<BlockId, BlockId>,
    /// Dominance sets (dominates(a, b) means a dominates b).
    dom_sets: HashMap<BlockId, HashSet<BlockId>>,
}

impl DominanceTree {
    /// Compute dominance from a CFG using iterative algorithm.
    pub fn compute(cfg: &Cfg) -> Self {
        let entry = cfg.entry();
        let blocks: Vec<BlockId> = cfg.blocks().map(|b| b.id).collect();

        // Initialize: entry dominates only itself, all others dominated by everything
        let mut dom_sets: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
        let all: HashSet<BlockId> = blocks.iter().copied().collect();

        for &b in &blocks {
            if b == entry {
                let mut s = HashSet::new();
                s.insert(b);
                dom_sets.insert(b, s);
            } else {
                dom_sets.insert(b, all.clone());
            }
        }

        // Iterate until fixed point
        let mut changed = true;
        while changed {
            changed = false;
            for &b in &blocks {
                if b == entry {
                    continue;
                }
                // Intersect dominators of all predecessors
                let preds: Vec<BlockId> = cfg.get(b).map(|blk| blk.predecessors.clone()).unwrap_or_default();
                let mut new_dom = all.clone();
                for &p in &preds {
                    if let Some(pdom) = dom_sets.get(&p) {
                        new_dom = new_dom.intersection(pdom).copied().collect();
                    }
                }
                new_dom.insert(b);

                if dom_sets.get(&b) != Some(&new_dom) {
                    dom_sets.insert(b, new_dom);
                    changed = true;
                }
            }
        }

        // Compute immediate dominators
        let mut idom: HashMap<BlockId, BlockId> = HashMap::new();
        for &b in &blocks {
            if b == entry {
                continue;
            }
            if let Some(dom) = dom_sets.get(&b) {
                let mut doms: Vec<BlockId> = dom
                    .iter()
                    .filter(|&&d| d != b)
                    .copied()
                    .collect();
                doms.sort_by(|a, b| {
                    let sa = dom_sets.get(a).map(|s| s.len()).unwrap_or(0);
                    let sb = dom_sets.get(b).map(|s| s.len()).unwrap_or(0);
                    sb.cmp(&sa) // largest set first = closest dominator
                });
                if let Some(&id) = doms.first() {
                    idom.insert(b, id);
                }
            }
        }

        Self { idom, dom_sets }
    }

    /// Get the immediate dominator of a block.
    pub fn idom(&self, b: BlockId) -> Option<BlockId> {
        self.idom.get(&b).copied()
    }

    /// Check if block `a` dominates block `b`.
    pub fn dominates(&self, a: BlockId, b: BlockId) -> bool {
        self.dom_sets.get(&b).is_some_and(|s| s.contains(&a))
    }

    /// Get all blocks dominated by `a`.
    pub fn dominated_by(&self, a: BlockId) -> Vec<BlockId> {
        let mut result = Vec::new();
        for (&b, doms) in &self.dom_sets {
            if b != a && doms.contains(&a) {
                result.push(b);
            }
        }
        result.sort_by_key(|b| b.0);
        result
    }

    /// Get the dominance tree children of a block.
    pub fn children(&self, a: BlockId) -> Vec<BlockId> {
        let mut children: Vec<BlockId> = self
            .idom
            .iter()
            .filter(|(_, &dom)| dom == a)
            .map(|(&b, _)| b)
            .collect();
        children.sort_by_key(|b| b.0);
        children
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::Cfg;

    fn linear_cfg() -> Cfg {
        let mut cfg = Cfg::new();
        let b1 = cfg.add_block();
        let b2 = cfg.add_block();
        cfg.add_edge(BlockId::ENTRY, b1);
        cfg.add_edge(b1, b2);
        cfg
    }

    #[test]
    fn test_linear_dominance() {
        let cfg = linear_cfg();
        let dom = DominanceTree::compute(&cfg);
        assert!(dom.dominates(BlockId::ENTRY, BlockId(1)));
        assert!(dom.dominates(BlockId::ENTRY, BlockId(2)));
        assert!(dom.dominates(BlockId(1), BlockId(2)));
        assert!(!dom.dominates(BlockId(2), BlockId(1)));
    }

    #[test]
    fn test_idom() {
        let cfg = linear_cfg();
        let dom = DominanceTree::compute(&cfg);
        assert_eq!(dom.idom(BlockId(1)), Some(BlockId::ENTRY));
        assert_eq!(dom.idom(BlockId(2)), Some(BlockId(1)));
    }

    #[test]
    fn test_entry_has_no_idom() {
        let cfg = Cfg::new();
        let dom = DominanceTree::compute(&cfg);
        assert_eq!(dom.idom(BlockId::ENTRY), None);
    }

    #[test]
    fn test_dominated_by() {
        let cfg = linear_cfg();
        let dom = DominanceTree::compute(&cfg);
        let dominated = dom.dominated_by(BlockId::ENTRY);
        assert!(dominated.contains(&BlockId(1)));
        assert!(dominated.contains(&BlockId(2)));
    }

    #[test]
    fn test_diamond_cfg() {
        let mut cfg = Cfg::new();
        let left = cfg.add_block();
        let right = cfg.add_block();
        let merge = cfg.add_block();
        cfg.add_edge(BlockId::ENTRY, left);
        cfg.add_edge(BlockId::ENTRY, right);
        cfg.add_edge(left, merge);
        cfg.add_edge(right, merge);
        let dom = DominanceTree::compute(&cfg);
        assert!(dom.dominates(BlockId::ENTRY, merge));
    }
}
