CREATE TABLE IF NOT EXISTS cursors
(
    id         text not null constraint cursor_pk primary key,
    cursor     text,
    block_num  bigint,
    block_id   text
);

CREATE TABLE IF NOT EXISTS blocks (
    height BIGINT PRIMARY KEY,
    hash TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    author TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    gas_price TEXT NOT NULL,
    total_supply TEXT NOT NULL
); 

CREATE TABLE IF NOT EXISTS chunks (
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

CREATE TABLE IF NOT EXISTS receipts (
    height BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    predecessor_id TEXT NOT NULL,
    receiver_id TEXT NOT NULL,
    receipt_kind TEXT NOT NULL,
    author TEXT NOT NULL,
    PRIMARY KEY (receipt_id)
);

CREATE TABLE IF NOT EXISTS receipt_actions (
    id TEXT PRIMARY KEY,
    block_height BIGINT NOT NULL,
    receipt_id TEXT NOT NULL,
    signer_account_id TEXT NOT NULL,
    signer_public_key TEXT NOT NULL,
    gas_price TEXT NOT NULL,
    action_kind TEXT NOT NULL,
    predecessor_id TEXT NOT NULL,
    receiver_id TEXT NOT NULL,
    block_hash TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    author TEXT NOT NULL,
    method_name TEXT NOT NULL,
    gas BIGINT NOT NULL,
    deposit TEXT NOT NULL,
    args_base64 TEXT NOT NULL,
    action_index INTEGER NOT NULL,
    block_timestamp TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS execution_outcomes (
    receipt_id TEXT PRIMARY KEY,
    block_height BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    shard_id TEXT NOT NULL,
    gas_burnt BIGINT NOT NULL,
    gas_used FLOAT NOT NULL,
    tokens_burnt FLOAT NOT NULL,
    executor_account_id TEXT NOT NULL,
    status TEXT NOT NULL,
    outcome_receipt_ids TEXT[] NOT NULL,
    executed_in_block_hash TEXT NOT NULL
);
