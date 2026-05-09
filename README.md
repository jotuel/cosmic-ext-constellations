# Constellations

Constellations is a native Matrix client for the [COSMIC desktop environment](https://github.com/pop-os/cosmic-epoch), built with [libcosmic](https://github.com/pop-os/libcosmic) and [matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk).

## Project Status: Alpha

Constellations is currently **alpha quality** software. While it is usable for basic daily tasks, you should expect bugs, missing features, and potential breaking changes.

The goal is to reach a stable 1.0 release once the underlying [matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk) reaches its own stable release.

### Recommendations

If you are looking for a more mature Matrix client today, we highly recommend:

- **[Fractal](https://gitlab.gnome.org/GNOME/fractal)**: A great GTK-based client for the GNOME desktop.
- **[iamb](https://github.com/ulysses-ao/iamb)**: A powerful terminal-based Matrix client for Vim lovers.

## Features

- **Authentication**: Support for both traditional password login and modern OIDC.
- **Room Management**: Sidebar with room list, spaces support, and room settings.
- **Messaging**: Timeline view with markdown support via a built-in composer.
- **Security**: End-to-end encryption (E2EE) support via `matrix-sdk-sqlite`.
- **COSMIC Integration**: Built using native COSMIC widgets and following the system design language.
- **Internationalization**: Support for multiple languages via Fluent.

## Building

To build Constellations, you will need the Rust toolchain and the dependencies required by `libcosmic`.

```bash
cargo build --release
```

To run the application:

```bash
cargo run --release
```

## License

This project is licensed under the MIT License or Apache License 2.0 (at your option).
