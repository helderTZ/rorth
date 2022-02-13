# RORTH

Rorth is a language like Forth but with an interpreter and compiler written in Rust.

Inspired by Porth: https://www.youtube.com/playlist?list=PLpM-Dvs8t0VbMZA7wW9aR3EtBqe2kinu4

Example:
```console
cargo run -- interpret tests/arithmetic.rorth
```

Compiling to native executable (Linux x86 only):
```console
cargo run -- compile tests/arithmetic.rorth
```

To run all tests:
```console
cargo test
```
