use std::hash::BuildHasherDefault;

use rustc_hash::FxHasher;

pub use db::{Db, Jar};
pub use semantic_model::{HasTy, SemanticModel};

pub mod ast_node_ref;
mod db;
pub mod name;
mod node_key;
pub mod semantic_index;
mod semantic_model;
pub mod types;

type FxIndexSet<V> = indexmap::set::IndexSet<V, BuildHasherDefault<FxHasher>>;
