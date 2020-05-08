# Vision

Provide a game engine that is simple, ergonomic, and performant.

## Simple

<details>
<summary>For developers, "just enough" code is required to describe an intent.</summary>

What can be defaulted should be, with the option of overriding if desired.

```rust
fn main() -> Result<(), Error> {
    smol::run(async {
        EventLoop::builder()
            .with_event_handler(MainLogic::new()) // RateLimit::Unlimited
            .with_event_handler(Renderer::new().with_rate_limit(Fps(60)))
            .run()
    })
}
```

</details>

<details>
<summary>Building native applications and WASM should just work.</summary>

```bash
$ wasm-pack build
# ..
[INFO]: :-) Done in 26.45s
[INFO]: :-) Your wasm pkg is ready to publish at /work/my-package/pkg.
```

</details>

## Ergonomic

<details>
<summary>Good error messages are to be expected.</summary>

Error messages should be understandable for newcomers, and helpful for experienced users.

```rust
error[E0001]: file not found: `"img/file.png"`
 --> src/assets.rs:20:16
   |
20 |     assets.load(path).await?;
   |                 ^^^^
   |
   = note: `#[deny(missing_assets)]` on by default
   = hint: ensure that the file exists:

       * development: `/work/game/img/file.png`
       * native: `~/games/img/file.png`
       * wasm: "https://example.com/img/file.png"
```

</details>

<details>
<summary>Developers are aided to write better code.</summary>

When application behaviour produces degraded user experience, the software informs how to improve it.

```rust
warning: event handler took too long to complete: `MainLogic (duration: 272 ms)`
 --> src/lib.rs:32:36
   |
32 |         .with_event_handler(MainLogic::new())
   |                             ^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(slow_functions)]` on by default
   = hint: reduce usage of synchronous logic within event handlers
   = hint: benchmark the application to discover slow functions
```

Requires [RFC 2091] / [rust#47809].

</details>

<details>
<summary>Unnecessary features are gated for better compile and load times.</summary>

Most of the engine is not compiled by default, and may be turned on through features. This keeps compilation times lower during development, and faster load times at runtime.

```toml
[dependencies]
nginee_net_play = { version = "0.1.0", path = "crate/net_play", optional = true }

[features]
default = []
net_play = ["nginee_net_play"]
```

</details>

## Performant

<details>
<summary>Gain throughput through concurrency and parallelism both natively and in WASM.</summary>

Players should require a high-end computer to play a simple game. However, if a computer is capable of processing more in a short period of time, that capability may be used.

* `async` allows better utilization of CPU resources
* Parallelism allows more CPU resources to operate simultaneously on application logic.

</details>

[RFC 2091]: https://github.com/rust-lang/rfcs/blob/master/text/2091-inline-semantic.md
[rust#47809]: https://github.com/rust-lang/rust/issues/47809
