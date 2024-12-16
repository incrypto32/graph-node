use std::{collections::BTreeMap, ops::Range, sync::Arc};

use graph::{
    blockchain::{
        block_stream::{
            EntitySubgraphOperation, EntityWithType, SubgraphTriggerScanRange,
            TriggersAdapterWrapper,
        },
        mock::MockTriggersAdapter,
        SubgraphFilter,
    },
    components::store::SourceableStore,
    data_source::CausalityRegion,
    prelude::{BlockHash, BlockNumber, BlockPtr, DeploymentHash, StoreError, Value},
    schema::{EntityType, InputSchema},
};
use slog::Logger;
use tonic::async_trait;

pub struct MockSourcableStore {
    entities: BTreeMap<BlockNumber, Vec<EntityWithType>>,
    schema: InputSchema,
    block_ptr: Option<BlockPtr>,
}

impl MockSourcableStore {
    pub fn new(
        entities: BTreeMap<BlockNumber, Vec<EntityWithType>>,
        schema: InputSchema,
        block_ptr: Option<BlockPtr>,
    ) -> Self {
        Self {
            entities,
            schema,
            block_ptr,
        }
    }

    pub fn set_block_ptr(&mut self, ptr: BlockPtr) {
        self.block_ptr = Some(ptr);
    }

    pub fn clear_block_ptr(&mut self) {
        self.block_ptr = None;
    }

    pub fn increment_block(&mut self) -> Result<(), &'static str> {
        if let Some(ptr) = &self.block_ptr {
            let new_number = ptr.number + 1;
            self.block_ptr = Some(BlockPtr::new(ptr.hash.clone(), new_number));
            Ok(())
        } else {
            Err("No block pointer set")
        }
    }

    pub fn decrement_block(&mut self) -> Result<(), &'static str> {
        if let Some(ptr) = &self.block_ptr {
            if ptr.number == 0 {
                return Err("Block number already at 0");
            }
            let new_number = ptr.number - 1;
            self.block_ptr = Some(BlockPtr::new(ptr.hash.clone(), new_number));
            Ok(())
        } else {
            Err("No block pointer set")
        }
    }
}

#[async_trait]
impl SourceableStore for MockSourcableStore {
    fn get_range(
        &self,
        _entity_types: Vec<EntityType>,
        _causality_region: CausalityRegion,
        block_range: Range<BlockNumber>,
    ) -> Result<BTreeMap<BlockNumber, Vec<EntityWithType>>, StoreError> {
        Ok(self
            .entities
            .range(block_range)
            .map(|(k, v)| (*k, v.clone()))
            .collect())
    }

    fn input_schema(&self) -> InputSchema {
        self.schema.clone()
    }

    async fn block_ptr(&self) -> Result<Option<BlockPtr>, StoreError> {
        Ok(self.block_ptr.clone())
    }
}

#[tokio::test]
async fn test_triggers_adapter_with_entities() {
    // Create a schema for a User entity
    let id = DeploymentHash::new("test_deployment").unwrap();
    let schema = InputSchema::parse_latest(
        "type User @entity { id: String!, name: String!, age: Int }",
        id.clone(),
    )
    .unwrap();

    // Create some test entities
    let user1 = schema
        .make_entity(vec![
            ("id".into(), Value::String("user1".to_owned())),
            ("name".into(), Value::String("Alice".to_owned())),
            ("age".into(), Value::Int(30)),
        ])
        .unwrap();

    let user2 = schema
        .make_entity(vec![
            ("id".into(), Value::String("user2".to_owned())),
            ("name".into(), Value::String("Bob".to_owned())),
            ("age".into(), Value::Int(25)),
        ])
        .unwrap();

    // Create EntityWithType instances
    let user_type = schema.entity_type("User").unwrap();
    let entity1 = EntityWithType {
        entity_type: user_type.clone(),
        entity: user1,
        entity_op: EntitySubgraphOperation::Create,
        vid: 1,
    };

    let entity2 = EntityWithType {
        entity_type: user_type,
        entity: user2,
        entity_op: EntitySubgraphOperation::Create,
        vid: 2,
    };

    // Create a BTreeMap with entities at different blocks
    let mut entities = BTreeMap::new();
    entities.insert(1, vec![entity1]);
    entities.insert(2, vec![entity2]);

    // Create block hash properly
    let hash_bytes: [u8; 32] = [0u8; 32];
    let block_hash = BlockHash(hash_bytes.to_vec().into_boxed_slice());

    // Create the mock store with proper BlockPtr
    let initial_block = BlockPtr::new(block_hash, 0);
    let store = Arc::new(MockSourcableStore::new(
        entities,
        schema.clone(),
        Some(initial_block),
    ));

    // Create the adapter wrapper
    let adapter = Arc::new(MockTriggersAdapter {});
    let wrapper = TriggersAdapterWrapper::new(adapter, vec![store]);

    // Create a test filter with correct structure
    let filter = SubgraphFilter {
        subgraph: id,
        start_block: 0,                     // Start from block 0
        entities: vec!["User".to_string()], // Monitor the User entity
    };

    // Test scanning a range of blocks
    let logger = Logger::root(slog::Discard, slog::o!());
    let result = wrapper
        .blocks_with_subgraph_triggers(&logger, &[filter], SubgraphTriggerScanRange::Range(1, 2))
        .await;

    assert!(result.is_ok(), "Failed to get triggers: {:?}", result.err());

    let blocks = result.unwrap();
    assert!(!blocks.is_empty(), "Should have found blocks with triggers");

    // Additional assertions could be made here about the specific blocks and triggers found
}
