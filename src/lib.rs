mod pb;
mod processors;
mod pushers;
mod stores;
mod transforms;
pub mod config;

use substreams_database_change::pb::database::DatabaseChanges;
use substreams::store::{self, DeltaProto};

// Store handlers and transform functions
use crate::transforms::*;

#[substreams::handlers::map]
fn db_out(
    blocks_deltas: store::Deltas<DeltaProto<pb::near::entities::v1::Block>>,
    receipts_deltas: store::Deltas<DeltaProto<pb::near::entities::v1::Receipt>>,
    receipt_actions_deltas: store::Deltas<DeltaProto<pb::near::entities::v1::ReceiptAction>>,
    execution_outcomes_deltas: store::Deltas<DeltaProto<pb::near::entities::v1::ExecutionOutcome>>,
) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut database_changes: DatabaseChanges = Default::default();
    
    // Transform all store deltas into database changes
    transform_blocks_deltas(&mut database_changes, blocks_deltas);
    transform_receipts_deltas(&mut database_changes, receipts_deltas);
    transform_receipt_actions_deltas(&mut database_changes, receipt_actions_deltas);
    transform_execution_outcomes_deltas(&mut database_changes, execution_outcomes_deltas);

    Ok(database_changes)
}
