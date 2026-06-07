//! # CFG Construct
//!
//! Control flow graph construction from basic blocks with dominance analysis.

pub mod basic_block;
pub mod cfg;
pub mod dominance;
pub mod frontier;
pub mod loop_detect;

pub use basic_block::{BasicBlock, BlockId, Instruction};
pub use cfg::Cfg;
pub use dominance::DominanceTree;
pub use frontier::DominanceFrontier;
pub use loop_detect::{LoopInfo, NaturalLoop};
