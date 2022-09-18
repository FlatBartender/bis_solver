# SGE BiS solving tool

This tool is an experimental FFXIV SGE best-in-slot gearset solver.

# Building and running

`cargo run --release`

# Building the user-friendly documentation

First of all, you will need [`mdbook`](https://github.com/rust-lang/mdBook/releases).

You will need to remove this from `book.toml`:
```toml
[preprocessor.include-checks]
command = "mdbook-include-check"
before = ["links"]
```

Then, you can run `mdbook build`, and the documentation will be available at `book/index.html`.
