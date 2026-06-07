//! Basic block definitions.

/// Unique identifier for a basic block.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct BlockId(pub usize);

impl BlockId {
    /// Entry block ID convention.
    pub const ENTRY: BlockId = BlockId(0);

    /// Returns the raw index.
    pub fn index(self) -> usize {
        self.0
    }
}

/// A simple instruction placeholder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    /// No-op / placeholder.
    Nop,
    /// Assignment: target = value.
    Assign { target: String, value: String },
    /// Binary operation.
    BinOp { dest: String, op: String, lhs: String, rhs: String },
    /// Conditional branch.
    Branch { cond: String },
    /// Unconditional jump target.
    Jump { target: BlockId },
    /// Return.
    Return { value: Option<String> },
    /// Phi node.
    Phi { dest: String, sources: Vec<(BlockId, String)> },
}

/// A basic block containing a sequence of instructions.
#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub predecessors: Vec<BlockId>,
    pub successors: Vec<BlockId>,
}

impl BasicBlock {
    /// Create a new empty basic block.
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }

    /// Add an instruction.
    pub fn push(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    /// Number of instructions.
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Returns true if no instructions.
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Returns true if this block ends with a terminator.
    pub fn is_terminated(&self) -> bool {
        self.instructions.last().is_some_and(|i| {
            matches!(i, Instruction::Branch { .. } | Instruction::Jump { .. } | Instruction::Return { .. })
        })
    }

    /// Add a successor edge.
    pub fn add_successor(&mut self, succ: BlockId) {
        if !self.successors.contains(&succ) {
            self.successors.push(succ);
        }
    }

    /// Add a predecessor edge.
    pub fn add_predecessor(&mut self, pred: BlockId) {
        if !self.predecessors.contains(&pred) {
            self.predecessors.push(pred);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = BasicBlock::new(BlockId(0));
        assert!(block.is_empty());
        assert_eq!(block.id, BlockId::ENTRY);
    }

    #[test]
    fn test_push_instruction() {
        let mut block = BasicBlock::new(BlockId(0));
        block.push(Instruction::Assign { target: "x".into(), value: "1".into() });
        assert_eq!(block.len(), 1);
    }

    #[test]
    fn test_terminated() {
        let mut block = BasicBlock::new(BlockId(0));
        assert!(!block.is_terminated());
        block.push(Instruction::Return { value: None });
        assert!(block.is_terminated());
    }

    #[test]
    fn test_predecessor_successor() {
        let mut block = BasicBlock::new(BlockId(0));
        block.add_successor(BlockId(1));
        block.add_predecessor(BlockId(2));
        assert_eq!(block.successors.len(), 1);
        assert_eq!(block.predecessors.len(), 1);
    }

    #[test]
    fn test_no_duplicate_edges() {
        let mut block = BasicBlock::new(BlockId(0));
        block.add_successor(BlockId(1));
        block.add_successor(BlockId(1));
        assert_eq!(block.successors.len(), 1);
    }
}
