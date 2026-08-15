//! Akkadian DSL v3.4 - AST Node Definitions
//! Complete port from AkkadianNodes.cs to Rust
//! 
//! Author: Bahaa Fadam
//! BahyWay Sovereign Ecosystem

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// PROGRAM ROOT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkkadianProgram {
    pub contexts: Vec<ContextNode>,
}

impl AkkadianProgram {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
        }
    }
}

// ============================================================================
// CONTEXT NODE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextNode {
    pub name: String,
    pub identities: Vec<IdentityNode>,
    pub storage: Option<StorageNode>,
    pub vectorization: Option<VectorizationNode>,
    pub commands: Vec<CommandNode>,
    pub rag_queries: Vec<RagQueryNode>,
    
    // NEW V3.1 BLOCKS
    pub rule_set: Option<RuleSetNode>,
    pub presentation: Option<PresentationNode>,
    pub meta_algorithmics: Option<MetaAlgorithmicsNode>,
}

impl ContextNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            identities: Vec::new(),
            storage: None,
            vectorization: None,
            commands: Vec::new(),
            rag_queries: Vec::new(),
            rule_set: None,
            presentation: None,
            meta_algorithmics: None,
        }
    }
}

// ============================================================================
// IDENTITY (Enhanced)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityNode {
    pub name: String,
    pub business_keys: Vec<String>,
    pub fuzzy_rules: Vec<FuzzyRuleConfig>,
    pub spatial_id: Option<SpatialIdConfig>,
}

impl IdentityNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            business_keys: Vec::new(),
            fuzzy_rules: Vec::new(),
            spatial_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyRuleConfig {
    pub fields: Vec<String>,
    pub algorithm: String,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialIdConfig {
    pub algorithm: String,
    pub dimensions: Vec<String>,
    pub precision: u32,
}

// ============================================================================
// STORAGE (Enhanced)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNode {
    pub hubs: Vec<TableNode>,
    pub satellites: Vec<TableNode>,
    pub links: Vec<TableNode>,
}

impl StorageNode {
    pub fn new() -> Self {
        Self {
            hubs: Vec::new(),
            satellites: Vec::new(),
            links: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableNode {
    pub name: String,
    pub table_type: TableType,
    pub has_temporal_tracking: bool,
    
    // Physical Storage Options
    pub partition_strategy: Option<String>,
    pub clustered_by: Option<String>,
    
    pub columns: Vec<ColumnNode>,
    pub indexes: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TableType {
    Hub,
    Satellite,
    Link,
}

impl TableNode {
    pub fn new(name: String, table_type: TableType) -> Self {
        Self {
            name,
            table_type,
            has_temporal_tracking: false,
            partition_strategy: None,
            clustered_by: None,
            columns: Vec::new(),
            indexes: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnNode {
    pub name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub is_unique: bool,
}

// ============================================================================
// PRESENTATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationNode {
    pub styles: Vec<StyleNode>,
}

impl PresentationNode {
    pub fn new() -> Self {
        Self {
            styles: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleNode {
    pub target_entity: String,
    pub color: String,
    pub icon: String,
    pub shape: String,
    pub size_by_field: String,
}

// ============================================================================
// RULE SET
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSetNode {
    pub name: String,
    pub algorithm: String, // Default: "MAMDANI"
    pub logic_rules: Vec<String>,
}

impl RuleSetNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            algorithm: "MAMDANI".to_string(),
            logic_rules: Vec::new(),
        }
    }
}

// ============================================================================
// META ALGORITHMICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAlgorithmicsNode {
    pub execution_model: String, // Default: "actor_based"
    pub actors: Vec<ActorConfig>,
}

impl MetaAlgorithmicsNode {
    pub fn new() -> Self {
        Self {
            execution_model: "actor_based".to_string(),
            actors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorConfig {
    pub name: String,
    pub properties: HashMap<String, String>,
}

// ============================================================================
// VECTORIZATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizationNode {
    pub model_name: String,
    pub embeddings: HashMap<String, Vec<String>>,
}

impl VectorizationNode {
    pub fn new() -> Self {
        Self {
            model_name: String::new(),
            embeddings: HashMap::new(),
        }
    }
}

// ============================================================================
// COMMANDS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandNode {
    pub name: String,
    pub validations: Vec<String>,
    pub execution_steps: Vec<StatementNode>,
}

impl CommandNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            validations: Vec::new(),
            execution_steps: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementNode {
    pub action: String,
    pub target: String,
    pub json_payload: String,
}

// ============================================================================
// RAG QUERY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQueryNode {
    pub name: String,
    pub description: String,
    pub retrieval: Option<RetrievalNode>,
    pub generation: Option<GenerationNode>,
}

impl RagQueryNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: String::new(),
            retrieval: None,
            generation: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalNode {
    pub vector_target: String,
    pub top_k: u32,
    pub graph_hops: u32,
    pub graph_edges: Vec<String>,
    pub temporal_window: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationNode {
    pub model: String,
    pub prompt_template: String,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_program() {
        let mut program = AkkadianProgram::new();
        let context = ContextNode::new("TestContext".to_string());
        program.contexts.push(context);
        
        assert_eq!(program.contexts.len(), 1);
        assert_eq!(program.contexts[0].name, "TestContext");
    }

    #[test]
    fn test_create_identity() {
        let mut identity = IdentityNode::new("PersonIdentity".to_string());
        identity.business_keys.push("name".to_string());
        identity.business_keys.push("birth_date".to_string());
        
        assert_eq!(identity.business_keys.len(), 2);
    }

    #[test]
    fn test_create_table() {
        let table = TableNode::new("PersonHub".to_string(), TableType::Hub);
        assert_eq!(table.name, "PersonHub");
        assert_eq!(table.table_type, TableType::Hub);
        assert!(!table.has_temporal_tracking);
    }
}
