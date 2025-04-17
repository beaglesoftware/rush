![Rush poster](assets/Rush_poster.png)

# Rush

A "beagleful" shell written in Rust. It is both blazingly fast and lightweight, and easy to learn.

Our goal is to have a shell that it's fast, lightweight and easy to learn and cross-platform. Also have a Bourne shell-compatible shell not only for macOS or Linux but for Windows, too.

"We won't publish our Bourne shell-compatible shell for Windows because Windows is terrible, is not free and open-source, it's bloated, Just for Linux" no more.

> [!CAUTION]
>
> The project is under development. It may crash and it doesn't advanced features

> [!NOTE]
>
> Rush is not Bourne shell-compatible shell, yet.

![A preview for Rush](assets/Rush_preview.gif)

## Installation

### Using Cargo

```shell
cargo install rush
```

### Package managers

Not avaliable yet.

## Thanks to

- [reedline](https://github.com/nushell/reedline): A feature-rich line editor - powering Nushell
- [serde](https://serde.rs): Serialization framework for Rust
- [serde_json](https://github.com/serde-rs/json): Strongly typed JSON library for Rast
- [miette](https://github.com/zkat/miette): Fancy extension for std::error::Error with pretty, detailed diagnostic printing.
- [anyhow](https://github.com/dtolnay/anyhow): Flexible concrete Error type built on std::error::Error
- []