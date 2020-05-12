# nginee

![CI](https://github.com/azriel91/nginee/workflows/CI/badge.svg) [![codecov](https://codecov.io/gh/azriel91/nginee/branch/master/graph/badge.svg)](https://codecov.io/gh/azriel91/nginee)

Experimental `async` game engine designed to be simple, ergonomic and performant.

For more information, please see the [vision] and [rationale] documents.

## Development

### Testing

Workspace tests can be run using:

```bash
cargo test --workspace --tests -- --nocapture --color always
```

### Code Coverage

1. To check code coverage locally, run:

    ```bash
    cargo tarpaulin --workspace --exclude-files "examples/*" --run-types Tests --out Html
    ```

2. Open `tarpaulin-report.html` in the repository root.

## Examples

### Native

To run an example, run the following commands from the repository root:

```bash
example=event_loop
cargo run --manifest-path "examples/${example}/Cargo.toml"
```

### WASM

To build an individual example, run the following commands from the repository root:

```bash
example=event_loop # example name
(cd "examples/${example}"; wasm-pack build --target web --out-dir "../../doc/src/pkg")
```

To build the all examples for the WASM target, run:

```bash
for example in $(ls examples)
do (cd "examples/${example}"; wasm-pack build --target web --out-dir "../../doc/src/pkg")
done
```

To view the example, run `mdbook serve` in the `doc` directory, and navigate to <http://localhost:3000/>.

[vision]: VISION.md
[rationale]: RATIONALE.md
