# ⚙️ [nginee]

[![CI](https://github.com/azriel91/nginee/workflows/CI/badge.svg)](https://github.com/azriel91/nginee/actions?query=workflow%3ACI) [![codecov](https://codecov.io/gh/azriel91/nginee/branch/master/graph/badge.svg)](https://codecov.io/gh/azriel91/nginee)

Experimental `async` game engine designed to be simple, ergonomic and performant.

For more information, please see the [vision] and [rationale] documents.

To contribute, please see the [contribution guide].

## Examples

### Native

To run an example, run the following commands from the repository root:

```bash
cargo run --example event_loop
```

### WASM

To build an individual example, run the following commands from the repository root:

```bash
wasm-pack build --target web --out-dir "../../doc/src/pkg" examples/event_loop
```

To build the all examples for the WASM target, run:

```bash
for example in $(ls examples)
do wasm-pack build --target web --out-dir "../../doc/src/pkg" "examples/${example}"
done
```

To view the example, run `mdbook serve doc`, and navigate to <http://localhost:3000/>.

[contribution guide]: CONTRIBUTING.md
[nginee]: https://nginee.rs
[rationale]: RATIONALE.md
[vision]: VISION.md
