# elm-strip-comments

A program to do one thing only: remove comments from Elm code.


## Install

You can directly download the executable for your system
from the [latest release](https://github.com/mpizenberg/elm-strip-comments/releases),
and put it in a directory in your PATH env variable.


## Usage

```sh
# Transform one file, result is printed to stdout
elm-strip-comments src/Main.elm > src/MainNoComments.elm

# Transform multiple files in place
elm-strip-comments --replace src/*.elm
```


## Contributing

Contributions are very welcome.
This project uses [rust format][rustfmt] and [clippy][clippy] (with its default options) to enforce good code style.
To install these tools run

```bash
rustup update
rustup component add clippy rustfmt
```

And then before committing run

```bash
cargo fmt -- --check
touch Cargo.toml && cargo clippy
```

PS: clippy is a rapidly evolving tool so if there are lint errors on CI
don't forget to `rustup update`.

[rustfmt]: https://github.com/rust-lang/rustfmt
[clippy]: https://github.com/rust-lang/rust-clippy
