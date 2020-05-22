# Event Loop Rate Limit

Counts down from `1_000_000` as fast as possible, refreshing the screen at 10 FPS.

To run this example locally, run:

```bash
cargo run --package event_loop_rate_limit
```

For WASM:

```bash
wasm-pack build --target web --out-dir ../../doc/src/pkg examples/event_loop_rate_limit
```

Terminal:

<div id="terminal" class="language-bash hljs" style="display: block; font-family: monospace;"></div>

<script type="module">
import init, * as exports from '../pkg/event_loop_rate_limit.js';
window.onload = async function() {
    await init();
    exports.run();
};
</script>
