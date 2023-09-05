# Alphabetic MerkleTree API

Service helper for generating MerkleTree inclusion and exclusion proofs for CroccsGovernancePlugin proposals

# Getting started

Prerequisites:
 - Rust
 - Postgres

```
$ git clone https://github.com/GoldmanDAO/alphabetic-merkle
$ cd alphabetic-merkle
$ cargo build
$ export DATABASE_URL=postgres://localhost/alphabetic_merkle   # This DB should already exists
$ cargo run
```
