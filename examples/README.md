# Rob reference programs

Twenty small programs that define, by example, what Rob code looks like. They are the
corpus every later milestone is checked against: the grammar must parse them, the type
checker must accept them, the interpreter must run them, and the backend must compile them.

They are written against the decisions recorded in [`../docs/`](../docs):
[`grammar.md`](../docs/grammar.md), [`type-system.md`](../docs/type-system.md),
[`memory-model.md`](../docs/memory-model.md), and
[`adr/0001-error-handling.md`](../docs/adr/0001-error-handling.md).

## The programs

| # | File | Exercises |
|---|------|-----------|
| 1 | `01_hello_world.rob` | entry point, string interpolation |
| 2 | `02_fibonacci.rob` | recursion, iteration, `mut`, `Map` memoization, ranges |
| 3 | `03_fizzbuzz.rob` | `for`, `match` with tuple patterns and guards, modulo |
| 4 | `04_temperature.rob` | `data class`, `Copy`, destructuring, `Double` arithmetic |
| 5 | `05_bank_account.rob` | class, private `mut` field, visibility, typed `throws`, custom `Error` |
| 6 | `06_shapes.rob` | interface + default method, `enum` variants, `match`, `This` |
| 7 | `07_quicksort.rob` | generics with a bound, `This`, recursion, lambdas |
| 8 | `08_binary_search.rob` | `T?`/`Option` for not-found, existence check, `while` |
| 9 | `09_generic_stack.rob` | generic container, `throws` on underflow, `Option` peek |
| 10 | `10_linked_list.rob` | recursive structure via `shared`, function-typed param, move |
| 11 | `11_binary_tree.rob` | `shared` owning edges, `weak` parent to break cycles |
| 12 | `12_json.rob` | recursive `enum`, `match`, `List`/`Map`, tuples, serialization |
| 13 | `13_csv_parser.rob` | string splitting, typed `throws`, per-line recovery |
| 14 | `14_word_count.rob` | `Map`, elvis default, entry iteration, higher-order sort |
| 15 | `15_higher_order.rob` | function types, closures, `map`/`filter`/`reduce`, `compose` |
| 16 | `16_calculator.rob` | recursive AST `enum`, `match`, `throws` on divide-by-zero |
| 17 | `17_file_io.rob` | RAII cleanup, typed `throws` for I/O, line reading/writing |
| 18 | `18_option_demo.rob` | `Option` nesting (`Some(None)`), `?.`, `?:`, `!!` |
| 19 | `19_vectors_raytrace.rob` | `Copy` value math, zero allocation on a hot path |
| 20 | `20_async_fetch.rob` | `async`/`await`, `spawn`, moving into tasks, errors at `await` |

## Feature coverage

Every construct in the specs appears in at least one program:

- **Declarations:** classes (5, 9, 10, 11), data classes (4, 7, 12, 13, 19), interfaces
  (6, 7), enums (6, 12, 16), free functions (all), generics with bounds (7, 9, 15).
- **Statements:** `if`/`else`, `while` (8, 9), `for`/`in` ranges (2, 3, 15), `return`,
  `continue` (14), assignment and augmented assignment (all with `mut`).
- **Expressions:** `match` with variant/tuple patterns and guards (3, 6, 12, 16, 18),
  lambdas (6, 7, 10, 15), function types (10, 15), tuples (3, 12, 14, 18), string
  interpolation (all), the full operator set (19).
- **Types & memory:** `Copy` (4, 7, 19), move (10, 20), `shared` (10, 11, 16), `weak`
  (11), `mut` and immutable bindings (all), `T?`/`Option` and nesting (8, 18).
- **Errors:** typed `throws`, `try` marker, `try`/`catch`/`finally` (5, 9, 13, 16, 17, 20).
- **Concurrency:** `async`/`await`, `spawn`, `Task` (20).

## Pending syntax (drives the grammar follow-ups)

Writing these programs is what surfaces the syntax the grammar does not yet formalize. The
following are used here per the design decisions but are still tracked as additions to
`grammar.md`:

- typed `throws E`, the `try` call-site marker, and `try`/`catch`/`finally`
  (ADR 0001 follow-up EH-1);
- the `weak T` type qualifier and `weak(expr)` constructor (memory-model MM-Open-3);
- data-class destructuring binding in a statement (`Celsius(back) = ...`, program 4).

This is intentional: the reference programs lead, and the formal grammar is updated to
accept them.

## Standard-library assumptions

The programs assume a small, conventional standard library that is not yet specified:
`println`, `List` (`of`, `empty`, `map`, `filter`, `reduce`, `get`, `add`, ...), `Map`
(`empty`, `get: V?`, `put`, `entries`), `String` (`split`, `trim`, `toInt`, `length`, ...),
`File`, `Http`, `Task`/`spawn`, and the `Error`/`Comparable`/`Copy` interfaces. These APIs
are provisional and will be pinned down by the standard-library work; the programs only rely
on their obvious shapes.
