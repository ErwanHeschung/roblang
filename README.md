<div align="center">

<img src="docs/assets/logo.svg" alt="Rob logo" width="150" height="150" />

# Rob

**The readability of Java with the performance profile of Rust.**

[![License: MIT or Apache 2.0](https://img.shields.io/badge/license-MIT%20or%20Apache%202.0-blue.svg)](#license)

</div>

Rob (Rust + Object) is a statically typed, compiled programming language. It keeps the
familiar shape of Java, classes, interfaces, and clear type declarations, while
compiling to native code with no garbage collector and static memory safety. This
repository holds the language specification and the compiler, which is written in Rust.

> **Status: early design and bootstrap.** The specification is taking shape under
> `docs/`, the first reference programs live under `examples/`, and the compiler covers
> a small bootstrap subset of the grammar end to end. The language is not yet ready for
> general use.

## A taste of Rob

```rob
package examples;

public fun main(): Unit {
    name: String = "world";
    println("hello, ${name}!");
}
```

Immutable by default, with opt-in mutation, ranges, and built-in null safety through
`T?`:

```rob
public fun fibIter(n: Int): Long {
    mut a: Long = 0;
    mut b: Long = 1;
    for (i: Int in 0..n) {
        next: Long = a + b;
        a = b;
        b = next;
    }
    return a;
}
```

More programs, from `quicksort` to a small ray tracer, live under [`examples/`](examples).

## Why Rob

- **Familiar surface.** Classes, interfaces, enums, and data classes, with `fun`
  methods and `name: Type` declarations. A Java developer should read Rob and feel at
  home.
- **No garbage collector.** Value types live inline by default, heap sharing is
  explicit and reference counted, and lifetimes are inferred rather than annotated.
- **Safety without ceremony.** Null safety is built in through `T?`, exclusivity is
  inferred, and errors are typed and checked at the call site.

## Documentation

The language design lives in [`docs/`](docs):

- [`docs/grammar.md`](docs/grammar.md) defines the full MVP syntax in EBNF.
- [`docs/type-system.md`](docs/type-system.md) specifies primitives, generics, and null safety.
- [`docs/memory-model.md`](docs/memory-model.md) covers value types, sharing, moves, and lifetimes.
- [`docs/adr/`](docs/adr) records the significant design decisions.

## Repository layout

```
docs/        language specification: grammar, type system, memory model, and ADRs
examples/    reference programs written in Rob (.rob)
crates/      the compiler, as a Cargo workspace
  rob_ast     abstract syntax tree
  rob_lexer   tokenizer (logos)
  rob_parser  recursive-descent / Pratt parser
```

The compiler currently implements a **bootstrap subset** (arithmetic expressions)
across the whole pipeline: `rob_lexer` tokenizes, `rob_parser` builds a
`rob_ast::Expr`, and snapshot tests pin the output. The lexer and parser grow toward
the full grammar in later work.

## Building

The compiler builds with a stable Rust toolchain (see `rust-toolchain.toml`, which
`rustup` reads automatically).

```sh
git clone https://github.com/ErwanHeschung/roblang.git
cd roblang
cargo build --all                                          # build every crate
cargo test --all                                           # run all tests
cargo fmt --all -- --check                                 # check formatting
cargo clippy --all-targets --all-features -- -D warnings   # lints, warnings denied
```

Continuous integration runs formatting, clippy with warnings denied, and the test
suite on every push to `main` and every pull request.

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) for the
workflow, branch and commit conventions, and the checks your change is expected to
pass.

## License

Rob is dual licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. Unless you explicitly state otherwise, any contribution you submit for
inclusion in this project shall be dual licensed as above, without any additional terms
or conditions.
