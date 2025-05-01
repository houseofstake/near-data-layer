CREATE TABLE IF NOT EXISTS cursors
(
    id         text not null constraint cursor_pk primary key,
    cursor     text,
    block_num  bigint,
    block_id   text
);

CREATE TABLE IF NOT EXISTS block_meta (
    height BIGINT PRIMARY KEY,
    hash TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    author TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    gas_price TEXT NOT NULL,
    total_supply TEXT NOT NULL
); 

CREATE TABLE IF NOT EXISTS chunk_meta (
    height BIGINT NOT NULL,
    chunk_hash TEXT NOT NULL,
    prev_block_hash TEXT NOT NULL,
    outcome_root TEXT NOT NULL,
    prev_state_root TEXT NOT NULL,
    encoded_merkle_root TEXT NOT NULL,
    encoded_length BIGINT NOT NULL,
    height_created BIGINT NOT NULL,
    height_included BIGINT NOT NULL,
    shard_id BIGINT NOT NULL,
    gas_used BIGINT NOT NULL,
    gas_limit BIGINT NOT NULL,
    validator_reward TEXT NOT NULL,
    balance_burnt TEXT NOT NULL,
    outgoing_receipts_root TEXT NOT NULL,
    tx_root TEXT NOT NULL,
    author TEXT NOT NULL,
    PRIMARY KEY (chunk_hash)
); 
