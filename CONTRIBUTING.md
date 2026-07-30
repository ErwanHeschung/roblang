# Contributing to Rob

Thanks for your interest in Rob. This document explains how the project is organized
and what a good contribution looks like. Contributions of all sizes are welcome, from
fixing a typo in the specification to implementing a new compiler pass.

## Code of conduct

Be respectful and constructive. Assume good faith, keep discussion focused on the work,
and help newcomers get started. Harassment of any kind is not tolerated.

## How the project is organized

Work is tracked through GitHub issues. Each unit of work has an issue, and each issue is
developed on its own branch and lands through a pull request. Nothing is pushed straight
to `main`.

The repository has two halves:

- **Specification** (`docs/`, `examples/`): the language design and reference programs.
- **Compiler** (`crates/`): the Rust implementation, as a Cargo workspace.

## Getting started

You need a stable Rust toolchain. The pinned channel is declared in
`rust-toolchain.toml`, so `rustup` selects the right version automatically.

```sh
git clone https://github.com/ErwanHeschung/roblang.git
cd roblang
cargo build --all
cargo test --all
```

## Workflow

1. **Pick or open an issue.** Comment so others know you are working on it.
2. **Branch off `main`.** Name the branch after the issue, for example
   `issue-42-parser-error-recovery`.
3. **Make focused commits.** Keep each commit coherent and its message clear.
4. **Run the checks locally** (see below) before opening a pull request.
5. **Open a pull request** against `main` and link the issue it closes.

### Commit messages

Commit messages follow the Conventional Commits style: a type, an optional scope, and a
short imperative summary.

```
feat: add lexer support for string literals
fix: correct operator precedence for unary minus
docs: clarify the null-safety rules in the type system
```

Common types are `feat`, `fix`, `docs`, `test`, `refactor`, and `chore`.

## Checks your change must pass

Continuous integration runs the following on every pull request, and each step must be
green before a change can merge. Run them locally first:

```sh
cargo fmt --all -- --check                                 # formatting
cargo clippy --all-targets --all-features -- -D warnings   # lints, warnings denied
cargo test --all                                           # tests
```

The parser uses [`insta`](https://insta.rs) for snapshot tests. When a change alters
parser output, review and accept the new snapshots with `cargo insta review`, and commit
the updated `.snap` files alongside your change.

## Working on the specification

Specification and documentation live under `docs/` and `examples/`. A few conventions:

- Documentation is written in English.
- Significant design decisions are recorded as ADRs under `docs/adr/`.
- Keep `examples/` programs consistent with the current grammar in `docs/grammar.md`.

## License

By contributing to Rob, you agree that your contributions will be dual licensed under
the Apache License 2.0 and the MIT license, matching the license of the project. You can
find the full terms in [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).
