# Rationale

## Short Version

* Many people are trying out Rust to build games.
* Early Rust game libraries are not structured for running within browsers.
* Newer libraries are in discovery mode, and prioritize getting things to work over learnability.
* *nginee* aims to prioritize usability and user experience over features.

## Long Version

### Existing Libraries

Most existing Rust game engine libraries have one or more of the following properties:

* Built for synchronous execution.
* Tied to native rendering models.
* Assume local configuration / assets.
* Have many dependencies.

When targeting the web, the runtime environment (browser) is largely different:

* Control flow model is asynchronous and event based.
* Application is loaded on demand, as opposed to downloaded beforehand.
* Configuration is loaded remotely, instead of from the file system.

Bridging these two models is difficult to do, and difficult to do well.

### Community

The Rust game development ecosystem is still evolving, but is also crossing into the "growth" phase &ndash; more people are trying to use Rust to build games. Often they are new to Rust, and may also be new to game development.

Many existing libraries provide the means to build good games; what tends to be difficult is still debugging / troubleshooting, alleviated by support through instant messaging platforms, issue trackers, and forums.

### Direction

Starting afresh gives us an opportunity to build a game engine that fits the asynchronous event execution model, as well as leverage technologies such as Rust's `async` crates and [`wgpu-rs`], which were not available or in a ready enough state compared to when the earlier game engine libraries began.

Notably the [`miniquad`] family of crates by [Fedor] is something to watch, as it targets PC, mobile, and web targets, with [`macroquad`] using an `async` event loop.

[`macroquad`]: https://github.com/not-fl3/macroquad
[`miniquad`]: https://github.com/not-fl3/miniquad
[`wgpu-rs`]: https://github.com/gfx-rs/wgpu-rs
[Fedor]: https://github.com/not-fl3
