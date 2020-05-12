# Event Loop

To compile this example locally, run:

```bash
wasm-pack build --target web --out-dir ../../doc/src/pkg examples/event_loop
```

Open the browser console to see the output of this application.

<script type="module">
import init, * as exports from '../pkg/example_event_loop.js';
window.onload = async function() {
    await init();
    exports.run();
};
</script>
