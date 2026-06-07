//! Natural loop detection.

use crate::basic_block::BlockId;
use crate::cfg::Cfg;
use std::collections::{HashMap, HashSet};

/// A detected natural loop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NaturalLoop {
    /// The loop header.
    pub header: BlockId,
    /// The back edge source.
    pub back_edge: BlockId,
    /// All blocks in the loop body (including header).
    pub body: HashSet<BlockId>,
}

impl NaturalLoop {
    /// Number of blocks in the loop.
    pub fn len(&self) -> usize {
        self.body.len()
    }

    /// Returns true if the loop has no body blocks.
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }

    /// Check if a block is in this loop.
    pub fn contains(&self, b: BlockId) -> bool {
        self.body.contains(&b)
    }

    /// Nesting depth (1 = outermost).
    pub fn depth(&self, nesting: &LoopNesting) -> usize {
        nesting.depth(self.header).unwrap_or(1)
    }
}

/// Loop nesting information.
#[derive(Debug)]
pub struct LoopNesting {
    depths: HashMap<BlockId, usize>,
}

impl LoopNesting {
    /// Get the nesting depth of a block.
    pub fn depth(&self, b: BlockId) -> Option<usize> {
        self.depths.get(&b).copied()
    }
}

/// Result of loop detection.
#[derive(Debug)]
pub struct LoopInfo {
    pub loops: Vec<NaturalLoop>,
    pub nesting: LoopNesting,
}

impl LoopInfo {
    /// Detect natural loops in a CFG.
    pub fn detect(cfg: &Cfg) -> Self {
        let mut loops = Vec::new();

        // Find back edges: edges where the target dominates the source
        // For simplicity, we use a simple dominance check
        let dom = crate::dominance::DominanceTree::compute(cfg);

        for block in cfg.blocks() {
            for &succ in &block.successors {
                if dom.dominates(succ, block.id) {
                    // Back edge found: block -> succ where succ dominates block
                    let body = Self::compute_loop_body(cfg, succ, block.id);
                    loops.push(NaturalLoop {
                        header: succ,
                        back_edge: block.id,
                        body,
                    });
                }
            }
        }

        // Compute nesting depths
        let mut depths = HashMap::new();
        for header in loops.iter().map(|l| l.header) {
            let depth = loops.iter().filter(|l| l.body.contains(&header)).count();
            depths.insert(header, depth.max(1));
        }
        // Also set depths for body blocks
        for l in &loops {
            for &b in &l.body {
                depths.entry(b).or_insert(1);
            }
        }

        let nesting = LoopNesting { depths };
        LoopInfo { loops, nesting }
    }

    /// Compute the loop body for a natural loop with given header and back edge source.
    fn compute_loop_body(cfg: &Cfg, header: BlockId, back_edge: BlockId) -> HashSet<BlockId> {
        let mut body = HashSet::new();
        body.insert(header);

        // Walk backwards from back_edge to header
        let mut stack = vec![back_edge];
        while let Some(node) = stack.pop() {
            if node != header && body.insert(node) {
                if let Some(block) = cfg.get(node) {
                    for &pred in &block.predecessors {
                        if !body.contains(&pred) {
                            stack.push(pred);
                        }
                    }
                }
            }
        }

        body
    }

    /// Number of detected loops.
    pub fn len(&self) -> usize {
        self.loops.len()
    }

    /// Returns true if no loops.
    pub fn is_empty(&self) -> bool {
        self.loops.is_empty()
    }

    /// Find the innermost loop containing a block.
    pub fn innermost_loop(&self, b: BlockId) -> Option<&NaturalLoop> {
        self.loops
            .iter()
            .filter(|l| l.contains(b))
            .min_by_key(|l| l.body.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::Cfg;

    #[test]
    fn test_no_loops() {
        let mut cfg = Cfg::new();
        let b1 = cfg.add_block();
        cfg.add_edge(BlockId::ENTRY, b1);
        let info = LoopInfo::detect(&cfg);
        assert!(info.is_empty());
    }

    #[test]
    fn test_simple_loop() {
        let mut cfg = Cfg::new();
        let header = cfg.add_block(); // BlockId(1)
        let body = cfg.add_block(); // BlockId(2)
        cfg.add_edge(BlockId::ENTRY, header);
        cfg.add_edge(header, body);
        cfg.add_edge(body, header); // back edge

        let info = LoopInfo::detect(&cfg);
        assert_eq!(info.len(), 1);
        assert_eq!(info.loops[0].header, header);
    }

    #[test]
    fn test_loop_body() {
        let mut cfg = Cfg::new();
        let header = cfg.add_block(); // 1
        let body = cfg.add_block(); // 2
        cfg.add_edge(BlockId::ENTRY, header);
        cfg.add_edge(header, body);
        cfg.add_edge(body, header);

        let info = LoopInfo::detect(&cfg);
        let l = &info.loops[0];
        assert!(l.contains(header));
        assert!(l.contains(body));
    }

    #[test]
    fn test_innermost_loop() {
        let mut cfg = Cfg::new();
        let h1 = cfg.add_block();
        let b1 = cfg.add_block();
        cfg.add_edge(BlockId::ENTRY, h1);
        cfg.add_edge(h1, b1);
        cfg.add_edge(b1, h1);

        let info = LoopInfo::detect(&cfg);
        let inner = info.innermost_loop(b1);
        assert!(inner.is_some());
    }

    #[test]
    fn test_loop_nesting_depth() {
        let mut cfg = Cfg::new();
        let h1 = cfg.add_block();
        let body = cfg.add_block();
        cfg.add_edge(BlockId::ENTRY, h1);
        cfg.add_edge(h1, body);
        cfg.add_edge(body, h1);

        let info = LoopInfo::detect(&cfg);
        assert!(info.nesting.depth(h1).unwrap_or(0) >= 1);
    }
}
