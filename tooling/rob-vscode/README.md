# Rob Language Support

Minimal VS Code extension that adds syntax highlighting for [Rob](https://github.com/ErwanHeschung/roblang) (`.rob`) files.

Closes #45 — TextMate grammar + minimal VS Code extension.

## What's included

- **TextMate grammar** (`syntaxes/rob.tmLanguage.json`) covering the full lexical grammar in
  [`docs/grammar.md`](https://github.com/ErwanHeschung/roblang/blob/main/docs/grammar.md):
  - Line (`//`) and nested block (`/* */`) comments
  - String literals with `${expr}` / `$ident` interpolation and escapes
  - Char literals
  - Int literals (decimal, hex, octal, binary, with `_` separators) and float literals (incl. exponents)
  - Boolean/null literals, `this`/`super`
  - Declaration keywords (`class`, `interface`, `enum`, `data`, `fun`, `init`, `package`, `import`, `as`)
  - Modifiers (`public`, `private`, `open`, `override`, `abstract`, `static`, `shared`, `mut`, `async`)
  - Control flow (`if`, `else`, `while`, `for`, `in`, `return`, `break`, `continue`, `match`, `await`)
  - The typed error-handling surface from [ADR 0001](https://github.com/ErwanHeschung/roblang/blob/main/docs/adr/0001-error-handling.md): `throws`, `try`, `catch`, `finally`, `throw`
  - Class/interface/enum names and function names as distinct entities
  - PascalCase type references (`List<Order>`, `Int`, `Money`, ...)
  - Function call highlighting
  - Full operator set: `?:`, `?.`, `!!`, `->`, `..`, comparisons, logical, arithmetic, compound assignment
- **Language configuration** (`language-configuration.json`): comment toggling, bracket matching,
  auto-closing/surrounding pairs for `{}[]()"'`, indent-on-brace rules.

## Try it locally

1. Open this folder in VS Code.
2. Press `F5` (or **Run > Start Debugging**) to launch an Extension Development Host.
3. Open any `.rob` file (e.g. one of the files under `roblang/examples/`) in the new window.

## Packaging

```bash
npm install -g @vscode/vsce
vsce package
```

This produces a `.vsix` you can install via **Extensions: Install from VSIX...**.

## Notes / follow-ups

- `throws`/`try`/`catch`/`finally`/`throw` are highlighted per ADR 0001's "intended surface
  syntax," since that ADR notes these tokens are not yet formalized in `grammar.md` (follow-up
  EH-1). If the grammar changes when EH-1 lands, the grammar file should be updated to match.
- Type highlighting uses a PascalCase heuristic (matches the codebase's own naming convention)
  rather than semantic resolution, since TextMate grammars are regex-based and have no type
  information. A semantic highlighter (via a language server) could refine this later.
- No icon/theme included — this is intentionally the minimal grammar + extension scaffold the
  issue asks for.
