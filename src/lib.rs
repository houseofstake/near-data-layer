mod pb;
mod processors;
mod pushers;
pub mod config;

use substreams_database_change::pb::database::DatabaseChanges;
use crate::processors::process_block;
use crate::pb::sf::near::r#type::v1::Block;

#[substreams::handlers::map]
fn db_out(block: Block) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut database_changes: DatabaseChanges = Default::default();
    
    // Process the block and all its nested data
    process_block(&mut database_changes, &block);

    Ok(database_changes)
}
