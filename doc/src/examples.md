# Examples

Each page on this section demonstrates the examples compiled as WASM applications. Each example needs to first be compiled before the page will successfully display the example.

You may compile all examples for WASM before visiting the pages using the following command:

```bash
for example in $(ls examples)
do wasm-pack build --target web --out-dir "../../doc/src/pkg" "examples/${example}"
done
```

> **Note:**
>
> To run examples natively, run:
>
> ```bash
> cargo run --package <example_name>`
> ```
>
> This differs from the usual `--example` parameter as individual `Cargo.toml` files are used to specify the dependencies relevant to that example.
