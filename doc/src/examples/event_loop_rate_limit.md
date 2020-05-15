# Event Loop Rate Limit

Counts down from `1_000_000` as fast as possible, refreshing the screen at 60 FPS.

To compile this example locally, run:

```bash
(cd examples/event_loop_rate_limit && wasm-pack build --target web --out-dir ../../doc/src/pkg)
```

Terminal:

<div id="terminal" class="language-bash hljs" style="display: block; font-family: monospace;"></div>

<script type="module">
import init, * as exports from '../pkg/example_event_loop_rate_limit.js';
window.onload = async function() {
    await init();
    exports.run();
};
</script>
