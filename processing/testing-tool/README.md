# sql-testing-tool

A small utility for verifying that test data is loaded into the database correctly.

The tool reads SQL files from the local directory, runs them inside a transaction, but does **not** commit.
If an error occurs, the process failed.

Run in dry-run mode (no commit):

```
cargo run --bin sql-testing-tool
```

Run with an actual commit to the database:

```
cargo run --bin sql-testing-tool -- --commit
```
