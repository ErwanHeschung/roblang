# ADR 0001: Error handling model

- **Status:** Accepted
- **Date:** 2026-07-28
- **Issue:** [#4 ADR: error-handling model (Result vs exceptions)](https://github.com/ErwanHeschung/roblang/issues/4)
- **Companion docs:** [`type-system.md`](../type-system.md), [`memory-model.md`](../memory-model.md), [`grammar.md`](../grammar.md)

---

## Context

Rob needs one model for **recoverable** failures (a file is missing, input is malformed, a
network call fails). Three families were on the table:

- **`Result<T, E>`**: a fallible function returns a value that is either success or error,
  propagated explicitly (Rust). Maximum explicitness and zero cost, unfamiliar surface.
- **Exceptions**: a fallible function `throw`s and the error travels up the stack implicitly
  to a `catch` (Java/C#). Familiar and terse, but invisible control flow and stack unwinding.
- **Typed `throws`**: `throws`/`try` keywords with the error type in the signature and the
  failure point marked at the call site, compiled down to `Result` with no unwinding (Swift).

The decision is pulled by two goals that point in opposite directions:

- **Readability for Java developers** (a primary, measured goal) favors familiar `throws`
  clauses and `try`/`catch` blocks.
- **Performance in the Rust range with no GC and no hidden cost** favors value-based errors:
  explicit, allocation-free, no unwinding, no invisible control flow.

Two prior decisions constrain this one:

- Rob already models **absence** with `T?` = `Option<T>` (`type-system.md` § 4), a standard
  enum with syntactic sugar. Whatever we pick for errors should compose with that, not
  duplicate it.
- The whole language is built on **making cost and control flow visible**: explicit types
  everywhere, no inference, `shared` puts allocation in the type, "a reader never runs
  inference in their head". An error model with *invisible* control flow would be the one
  feature that contradicts Rob's own character.

## Decision

**Rob uses typed `throws`: a checked, value-based error model with a Java-shaped surface.**

It reads like Java's `throws`/`try`/`catch` (which the target audience already knows) and
runs like `Result<T, E>` (which the performance and no-GC goals require). Concretely:

1. **The error type is part of the signature.** A fallible function declares
   `: ReturnType throws E`, where `E` is the error type it may produce. This is Java's
   `throws` clause, made precise and enforced.
2. **Failure points are marked at the call site** with the prefix keyword `try`. A call to a
   throwing function must be written `try f(...)`. This is the one new thing relative to
   Java, and it is deliberate: it restores the *visible control flow* that the rest of the
   language guarantees. There is no invisible throw.
3. **`throws` is checked.** A `try`-marked call must be inside either (a) a function that
   itself declares a compatible `throws`, in which case the error **propagates**, or (b) a
   `try { ... } catch { ... }` block that **handles** it. Otherwise it is a compile error.
4. **Under the hood it is `Result<T, E>`.** The compiler lowers `throws`/`try` to a value
   returned in registers (a tagged union), with **no stack unwinding**, **no heap
   allocation**, and **no stack-trace capture**. There is nothing for the ownership and
   region analysis (`memory-model.md`) to unwind through.
5. **Error types compose by conformance.** Every error type conforms to the standard
   `Error` interface. Propagating a `try f()` whose error is `F` out of a function declaring
   `throws E` is allowed when `F <: E` (for example both conform to a shared error enum or
   `E` is `Error`); otherwise the caller must `catch` `F` and map it. `throws Error` is the
   deliberately-imprecise escape ("may fail somehow"), analogous to catching broadly.
6. **Errors are for the recoverable.** They are *not* the mechanism for:
   - **Absence of a value** — use `T?` / `Option<T>`; do not throw for "not found".
   - **Programmer bugs and broken invariants** (index out of bounds, debug overflow, failed
     internal assertion) — these `panic` and abort; they are outside the `throws`/`catch`
     flow. This is the same two-tier split implied by `memory-model.md` and `type-system.md`.

### Intended surface syntax

```rob
public class ParseError(message: String) : Error;

public fun parse(text: String): Int throws ParseError {
    if (!text.isNumeric()) { throw ParseError("not a number: ${text}"); }
    return text.toInt();
}

// Propagation: 'load' declares 'throws ParseError', so 'try parse(...)' flows out.
public fun load(raw: String): Config throws ParseError {
    n: Int = try parse(raw);       // visible failure point
    return Config(n);
}

// Handling: Java-shaped try/catch/finally. Inner calls keep their 'try' marker.
public fun main(): Unit {
    try {
        c: Config = try load(input);
        use(c);
    } catch (e: ParseError) {
        report(e.message);
    } finally {
        cleanup();                 // always runs
    }
}
```

For a Java reader: the `throws ParseError` clause and the `try`/`catch`/`finally` block are
already familiar; the only new token is the `try` in front of a fallible call, which marks
exactly where failure can occur.

> **Grammar note.** `throws E` on `functionDecl`/`methodDecl`, the `try` call-site prefix
> operator, the `try`/`catch`/`finally` statement, `throw`, and the `Error` interface are
> **not yet in [`grammar.md`](../grammar.md)**. Adding them is a required follow-up (EH-1).
> The `try` keyword is used both as the call marker (`try expr`) and to open a handling block
> (`try { ... }`); the two are disambiguated by whether `{` follows, mirroring Swift's
> `try` / `do` split but keeping the Java block keyword.

## Consequences

### Positive

- **Consistent with Rob's identity.** Failure points are visible (`try` markers) and error
  types are in the signature (`throws E`), matching the explicit-cost, explicit-control
  principle the rest of the language enforces. This is the decisive reason it was chosen over
  plain exceptions.
- **Familiar to the target audience.** `throws` clauses and `try`/`catch`/`finally` are Java
  vocabulary; only the call-site `try` is new, and it teaches the reader where errors happen.
- **Meets the performance goals.** Lowering to `Result` gives a zero-cost happy path *and* a
  cheap error path: no unwinding tables walked, no allocation, no stack traces. This is the
  strongest error model for `≤ 1.3× Rust`.
- **Does not burden the riskiest analysis.** With no unwinding, the ownership/region analysis
  (`memory-model.md`, the project's headline risk) gains no exceptional control-flow edges to
  reason about. Destruction stays the ordinary scope-exit RAII of a normal `return`.
- **Composes with `Option<T>`.** Both are standard enums with sugar (`Option` for absence,
  the `Result` behind `throws` for failure); one consistent story, two distinct concerns.

### Negative and costs we accept

- **More language machinery than raw exceptions.** There is surface sugar (`throws`/`try`)
  over an underlying `Result` lowering. This is real complexity in the front end, justified
  by keeping the runtime model simple (no unwinder).
- **Per-call verbosity.** Every fallible call carries a `try`. This is strictly more typing
  than invisible exceptions, and it is the point: visibility costs a keyword. The linter
  should not fight it.
- **Error-type plumbing across layers.** Propagating differing error types requires
  conformance to a common `Error`/enum or an explicit map, similar to Rust's conversion on
  `?`. Mitigated by `throws Error` and by shared error enums; still more thought than "throw
  anything".
- **Dual use of `try`** (call marker vs block opener) is a small learning bump, accepted for
  Java block familiarity.

### Cross-cutting rules

- **Concurrency.** A `throws` error does not implicitly cross a task boundary; it surfaces at
  the corresponding `await` of the task's result (consistent with `memory-model.md` § 8).
- **`Option` vs `throws` boundary.** "Not found" returns `T?`; "could not perform the
  operation" throws. A standard-library API commits to one; mixing them for the same
  condition is discouraged and lintable.

## Alternatives considered

### A. Raw `Result<T, E>` with a `?` propagation operator (rejected as the surface)

The Rust surface: `Result` return types, `?` to propagate, `match` to handle.

- **Pros:** maximally explicit and zero-cost; smallest language. **It is, in fact, the model
  Rob lowers `throws` to.**
- **Why rejected as the surface:** `Result<T, E>` return types, `?`, and `match` on every
  fallible call put an unfamiliar shape in front of the primary audience. Typed `throws`
  keeps this exact semantics while presenting Java vocabulary, so we adopt the semantics and
  reject only the surface.

### B. Java-style exceptions, unchecked, with stack unwinding (rejected)

Classic `throw`/`try`/`catch` with implicit propagation and runtime unwinding.

- **Pros:** most familiar; least code for propagation (no `try` markers, no `throws`
  plumbing).
- **Why rejected:** invisible control flow is the one feature that contradicts Rob's
  explicitness principle; unwinding adds exceptional edges to the ownership/region analysis
  that is already the project's top risk; and the throw path costs unwinding and boxing.
  Every modern no-GC/perf language (Rust, Swift, Go, Zig) avoided unchecked exceptions, and
  large C++ codebases disable them (`-fno-exceptions`, `std::expected`). The familiarity win
  did not justify abandoning the language's character and complicating its riskiest analysis.

### C. Checked exceptions (Java `throws`, enforced, with unwinding) (rejected)

- **Why rejected:** carries the unwinding costs of B, and the enforcement is the part of Java
  that Kotlin and C# deliberately dropped. Typed `throws` keeps the useful half (the error
  type in the signature, enforced) without the unwinding half.

## Follow-ups

- **EH-1:** Add the syntax to `grammar.md`: `throws E` in `functionDecl`/`methodDecl`, the
  `try` call-site prefix, the `try`/`catch`/`finally` statement, `throw`, and the `Error`
  interface. Blocking for implementation.
- **EH-2:** Specify the standard `Error` interface, the error-type conversion rule used on
  propagation, and the `panic` vs `throw` boundary in the type-system and stdlib docs.
- **EH-3:** Decide whether an opt-in debug stack-trace exists on `throws` errors and its cost
  (off by default in release; § Decision item 4).
- **EH-4:** Lint rules: encourage precise `throws E` over `throws Error`, and flag `throws` on
  documented hot paths.
- **EH-5:** Confirm the `try` dual-use parse (marker vs block) is unambiguous in the finalized
  grammar; if not, pick a distinct block keyword.
