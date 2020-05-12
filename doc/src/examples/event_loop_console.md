# Event Loop Console

To compile this example locally, run:

```bash
wasm-pack build --target web --out-dir ../../doc/src/pkg examples/event_loop_console
```

Terminal:

<div id="terminal" class="language-bash hljs" style="display: block; font-family: monospace;"></div>

<script type="module">
import init, * as exports from '../pkg/example_event_loop_console.js';
window.onload = async function() {
    await init();
    exports.run();
};
</script>
