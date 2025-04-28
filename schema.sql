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