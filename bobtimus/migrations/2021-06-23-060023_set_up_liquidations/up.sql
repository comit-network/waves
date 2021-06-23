CREATE TABLE liquidations
(
       id               TEXT NOT NULL PRIMARY KEY,
       tx_hex           TEXT NOT NULL,
       locktime         BIGINT NOT NULL
);
