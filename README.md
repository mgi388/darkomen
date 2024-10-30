# Warhammer: Dark Omen library and CLI in Rust

[![Crates.io](https://img.shields.io/crates/v/darkomen.svg)](https://crates.io/crates/darkomen)
[![Docs.rs](https://docs.rs/darkomen/badge.svg)](https://docs.rs/darkomen)
[![CI](https://github.com/mgi388/darkomen/workflows/CI/badge.svg)](https://github.com/mgi388/darkomen/actions)

A Rust library designed to work with the classic game **Warhammer: Dark Omen**. It provides developers with tools to read, manipulate, and write game data, enabling the creation of mods, custom levels, and analysis tools.

## Features

The following table shows the game file support in this library:

| Kind                                       | File extension(s) | Read | Write | Known issues?                           |
| ------------------------------------------ | ----------------- | ---- | ----- | --------------------------------------- |
| [3D models](src/m3d)                       | .M3D              | ✅   | ❌    |                                         |
| [Army and save games](src/army)            | .ARM              | ✅   | ✅    | ⚠️ Save games not completely understood |
| [Battle blueprints](src/battle)            | .BTB              | ✅   | ❌    |                                         |
| CTL                                        | .CTL              | ❌   | ❌    |                                         |
| Cursors                                    | .ANI              | ❌   | ❌    |                                         |
| Fonts                                      | .FNT              | ❌   | ❌    |                                         |
| Game flows                                 | .DOT              | ❌   | ❌    |                                         |
| [Lights](src/light)                        | .LIT              | ✅   | ✅    |                                         |
| Movies                                     | .TGQ              | ❌   | ❌    |                                         |
| Particle effects                           | .PLB, .H          | ❌   | ❌    |                                         |
| Portrait scripts                           | .KEY, .SEQ        | ❌   | ❌    |                                         |
| [Projects](src/project)                    | .PRJ              | ✅   | ✅    |                                         |
| [Shadows](src/shadow)                      | .SHD              | ✅   | ✅    |                                         |
| [Sound effects](src/sound/sfx)             | .H                | ✅   | ❌    |                                         |
| [Sound mono audio](src/sound/mad)          | .MAD              | ✅   | ✅    |                                         |
| [Sound scripts](src/sound/script)          | .FSM              | ✅   | ✅    |                                         |
| [Sound stereo audio](src/sound/sad)        | .SAD              | ✅   | ✅    |                                         |
| [Sprite sheets](src/graphics/sprite_sheet) | .SPR              | ✅   | ❌    |                                         |

## Installation

### Cargo

- Install the Rust toolchain, which also installs `cargo`, by following the [Install Rust guide](https://www.rust-lang.org/tools/install)
- Run `cargo add darkomen`

#### Cargo features

`darkomen` supports [Bevy Reflection](https://docs.rs/bevy_reflect/latest/bevy_reflect)
through the `bevy_reflect` feature. To enable it, add the following line to
your `Cargo.toml`:

```toml
darkomen = { version = "0.3.6", features = ["bevy_reflect"] }
```

## Important notes

> [!NOTE]
> This library does not ship with any game assets. You must have a copy of the game to get the most from this library.

> [!NOTE]
> This library is not developed by or endorsed by Games Workshop or Electronic Arts.

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
