# Cargo gtest tool

Along with various tools and libraries to facilitate stability of your Gear programs and any Gear network as a whole. We plan to use this to test for regressions and backward compatibility.

## How to use

#### Install cargo-gtest

For contracts development (Gear programs) to run on Vara or different Gear protocol powered network, install `cargo-gtest`:
```
cargo install --git https://github.com/NikVolf/onchain-tests
```

This will add cargo extension as a drop-in replacement for `cargo test`:
```
cargo gtest
```

(it accepts any parameters as regular `cargo test` does)

#### In your contracts/programs, tests can be declared with a simple decorator:

```rust
#[gear-test-codegen::test]
fn some_test() {
    assert_eq(1, 1)
}

```

#### Examples

See `./examplle` in this repository or standalone minimal example at https://github.com/NikVolf/gtest-min.
