//! Control flow graph construction.

use crate::basic_block::{BasicBlock, BlockId};

/// A control flow graph.
pub struct Cfg {
    blocks: Vec<BasicBlock>,
    entry: BlockId,
}

impl Cfg {
    /// Create a new CFG with an entry block.
    pub fn new() -> Self {
        Self {
            blocks: vec![BasicBlock::new(BlockId::ENTRY)],
            entry: BlockId::ENTRY,
        }
    }

    /// Add a new block and return its ID.
    pub fn add_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len());
        self.blocks.push(BasicBlock::new(id));
        id
    }

    /// Add an edge from one block to another.
    pub fn add_edge(&mut self, from: BlockId, to: BlockId) {
        if let Some(block) = self.blocks.get_mut(from.0) {
            block.add_successor(to);
        }
        if let Some(block) = self.blocks.get_mut(to.0) {
            block.add_predecessor(from);
        }
    }

    /// Get a reference to a block.
    pub fn get(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(id.0)
    }

    /// Get a mutable reference to a block.
    pub fn get_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(id.0)
    }

    /// Number of blocks.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Returns true if empty (should never be, since entry always exists).
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Returns the entry block ID.
    pub fn entry(&self) -> BlockId {
        self.entry
    }

    /// Iterate over all blocks.
    pub fn blocks(&self) -> impl Iterator<Item = &BasicBlock> {
        self.blocks.iter()
    }

    /// Compute reverse postorder of blocks.
    pub fn reverse_postorder(&self) -> Vec<BlockId> {
        let mut visited = vec![false; self.blocks.len()];
        let mut order = Vec::new();
        self.rpo_dfs(self.entry, &mut visited, &mut order);
        order.reverse();
        order
    }

    fn rpo_dfs(&self, id: BlockId, visited: &mut [bool], order: &mut Vec<BlockId>) {
        if visited[id.0] {
            return;
        }
        visited[id.0] = true;
        if let Some(block) = self.get(id) {
            for &succ in &block.successors {
                self.rpo_dfs(succ, visited, order);
            }
        }
        order.push(id);
    }

    /// Compute all exit blocks (blocks with no successors or returning).
    pub fn exit_blocks(&self) -> Vec<BlockId> {
        self.blocks
            .iter()
            .filter(|b| b.successors.is_empty())
            .map(|b| b.id)
            .collect()
    }
}

impl Default for Cfg {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_block::Instruction;

    #[test]
    fn test_cfg_creation() {
        let cfg = Cfg::new();
        assert_eq!(cfg.len(), 1);
        assert_eq!(cfg.entry(), BlockId::ENTRY);
    }

    #[test]
    fn test_add_block() {
        let mut cfg = Cfg::new();
        let b1 = cfg.add_block();
        assert_eq!(b1, BlockId(1));
        assert_eq!(cfg.len(), 2);
    }

    #[test]
    fn test_add_edge() {
        let mut cfg = Cfg::new();
        let b1 = cfg.add_block();
        cfg.add_edge(BlockId::ENTRY, b1);
        assert_eq!(cfg.get(BlockId::ENTRY).unwrap().successors.len(), 1);
        assert_eq!(cfg.get(b1).unwrap().predecessors.len(), 1);
    }

    #[test]
    fn test_reverse_postorder() {
        let mut cfg = Cfg::new();
        let b1 = cfg.add_block();
        let b2 = cfg.add_block();
        cfg.add_edge(BlockId::ENTRY, b1);
        cfg.add_edge(b1, b2);
        let rpo = cfg.reverse_postorder();
        assert_eq!(rpo[0], BlockId::ENTRY);
    }

    #[test]
    fn test_exit_blocks() {
        let mut cfg = Cfg::new();
        cfg.get_mut(BlockId::ENTRY).unwrap().push(Instruction::Return { value: None });
        let exits = cfg.exit_blocks();
        assert!(exits.contains(&BlockId::ENTRY));
    }
}
