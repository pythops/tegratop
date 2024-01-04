<div align="center">
  <img height="100" src="assets/logo.png"/>
  <h2> TUI monitoring tool (top like) for Nvidia jetson boards </h2>
</div>

## 🚀 Installation

### 📥 Binary release

You can download the pre-built binaries from the release page [release page](https://github.com/pythops/tegratop/releases)

### 📦 crates.io

You can install `tegratop` from [crates.io](https://crates.io/crates/tegratop)

```shell
cargo install tegratop
```

### ⚒️ Build from source

To build from the source, you need [Rust](https://www.rust-lang.org/) compiler and
[Cargo package manager](https://doc.rust-lang.org/cargo/).

#### On a Jetson board

Run the following command:

```shell
cargo build --release
```

Then run `strip` to reduce the size of the binary

```shell
strip target/aarch64-unknown-linux-gnu/release/tegratop
```

This will produce an executable file at `target/release/tegratop` that you can copy to a directory in your `$PATH`.

#### Cross compilation

Make sure you have those dependencies installed:

- [cross](https://github.com/cross-rs/cross)
- [podman](https://github.com/containers/podman)
- [aarch64-linux-gnu-strip ](https://command-not-found.com/aarch64-linux-gnu-strip)

then run the following command to build:

```shell
CROSS_CONTAINER_ENGINE=podman cross build --target=aarch64-unknown-linux-gnu --release
```

Finally, run `strip` to reduce the size of the binary

```
aarch64-linux-gnu-strip target/aarch64-unknown-linux-gnu/release/tegratop
```

## 🪄 Usage

run `tegratop` with sudo to get full the stats, otherwise some information might not show

```
$ sudo tegratop
```

ℹ️ If certain information is not displayed, you can check the file `/tmp/tegratop.log`

## ⚖️ License

GPLv3
