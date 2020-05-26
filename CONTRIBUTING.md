# Contributing

## Development Environment Setup

Install the following:

1. Rust: <https://rustup.rs/>.
2. `wasm32-unknown-unknown` component: `rustup component add wasm32-unknown-unknown`.
3. [`wasm-pack`]: <https://rustwasm.github.io/wasm-pack/installer/>
4. [`mdbook`]: `cargo install mdbook`
5. [`tarpaulin`] (Linux only, for coverage): `cargo install cargo-tarpaulin`

## Project structure

The following shows the project file structure:

```bash
nginee                  # Main crate
│
├── crate               # Libraries
│  ├── event_loop
│  ├── # ..
│  └── workspace_tests  # Tests
│
├── doc                 # Book documentation
│  └── src/examples     # Live WASM examples
│     ├── event_loop.md
│     └── # ..
│
└── examples            # Example crates
   ├── event_loop
   └── # ..
```

## Testing

Workspace tests can be run using:

```bash
cargo test --workspace
```

## Code Coverage

1. To check code coverage locally, run:

    ```bash
    cargo tarpaulin
    ```

2. Open `target/tarpaulin/tarpaulin-report.html`.

## Conventions

### Workspace

* All crates in the workspace are versioned the same.

    This saves us from requiring crate-name-prefixed git tags to track which commit contains which version of which crate.

    Consumers are expected to depend on the top level `nginee` crate, and the lower level crates may be disabled via feature flags.

### Library Crates

Library crates are placed under `crate/<name>`.

* Crate names are prefixed with `nginee_` inside `Cargo.toml`, but not in their directory name.
* `doctest` and `test` are set to `false`.

    This saves us from running an executable per test and doc snippet, which can take 1 second per crate.

    ```toml
    [lib]
    doctest = false
    test = false
    ```

* Tests are placed inside the `workspace_tests` crate.

    Each crate should have a corresponding module in `workspace_tests` that matches the crate name, and tests are placed in submodules that match the structure of the crate.

    Check out the [Dev Time Optimization] blog post for the rationale.

### Example Crates

Example crates are placed under `examples/<name>`.

* Crate names are prefixed with `example_` inside `Cargo.toml`, but not in their directory name.

    This clarifies that this is an example crate.

* Crates have their own `Cargo.toml` to track their own dependencies.

    This is necessary to not interleave different examples' dependencies when compiling to a WASM binary.

* Crates' source are directly inside the example crate's directory, with a `src` symlink.

    This allows the example to be both a `lib` and `bin` crate, and allows users to run examples using `cargo run --example <non_prefixed_name>`

* The main `nginee` crate must specify the example under `[dev-dependencies]`.

    This allows the example to be both a `lib` and `bin` crate.

* A book page is written for each example to provide a live demo of the WASM binary.

[`mdbook`]: https://github.com/rust-lang/mdBook
[`tarpaulin`]: https://github.com/xd009642/tarpaulin
[`wasm-pack`]: https://rustwasm.github.io/wasm-pack
[Dev Time Optimization]: https://azriel.im/will/2019/10/08/dev-time-optimization-part-1-1.9x-speedup-65-less-disk-usage/
