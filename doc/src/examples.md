# Examples

Each page on this section demonstrates the examples compiled to a live WASM application.

Each example needs to first be compiled before the page will successfully display the example.

You may compile all examples before visiting each page using the following command:

```bash
for example in $(ls examples)
do (cd "examples/${example}"; wasm-pack build --target web --out-dir "../../doc/src/pkg")
done
```
