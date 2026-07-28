# Rob Memory Model

> **Status:** living document. This file specifies how Rob manages memory: ownership,
> value vs `shared`, move semantics, mutation and aliasing, inferred lifetimes, and the
> "no hidden allocation" guarantee.
>
> **Issue:** [#3 Specify the memory model](https://github.com/ErwanHeschung/roblang/issues/3)
> **Companion docs:** syntax in [`grammar.md`](grammar.md), typing in
> [`type-system.md`](type-system.md). This file owns the *runtime meaning* of ownership;
> the type checker enforces the rules stated here.
>
> **Scope:** value types by default, `shared` heap types, move semantics, inferred
> lifetimes, no hidden allocation.

---

## 0. Design principles

1. **Values live inline by default.** A value is stored on the stack or embedded in its
   container. The heap is entered only on explicit request (`shared`) or through a
   collection type. There is no implicit boxing.
2. **Single ownership by default.** Every value has exactly one owner at a time. When the
   owner goes out of scope, the value is destroyed (deterministic, no GC).
3. **Sharing is explicit.** Multiple owners require the `shared` qualifier, which is
   reference counted. The cost (a refcount, a heap allocation) is visible in the type.
4. **Aliasing is inferred, never annotated.** The programmer never writes `&`, `&mut`, or a
   lifetime. The compiler performs the exclusivity and region analysis that a Rust
   programmer would write by hand.
5. **Safety is static.** No use-after-free and no data race can occur in safe code; both are
   compile errors. This is a consequence of principles 2 through 4.

---

## 1. Value types by default

A `class`, `data class`, `enum`, or primitive is a **value type**. Its storage is inline:

```rob
data class Point(x: Int, y: Int) : Copy;

p: Point = Point(1, 2);      // 16 bytes on the stack, no allocation
line: Segment = Segment(Point(0, 0), Point(1, 1));  // points embedded in Segment
```

Value types have **no identity**: two equal values are indistinguishable (§ 7 of
`type-system.md`). They are never null (only `T?` is, which is `Option<T>`). Nothing about a
value type touches the heap.

`shared` (§ 3) is the only built-in way to put a user value on the heap; collections
(`List`, `Map`) allocate internally but that allocation is a documented property of the
collection type, not a hidden effect of ordinary code.

---

## 2. Ownership: move and copy

### 2.1 Move is the default

Binding, assigning, passing as an argument, or returning a value **transfers ownership**.
For a non-`Copy` value this is a **move**: the source name becomes invalid, and using it
afterward is a compile error.

```rob
a: Buffer = Buffer.ofSize(1024);
b: Buffer = a;          // MOVE: ownership transferred to b
use(a);                 // ERROR: 'a' was moved to 'b'

fn(b);                  // MOVE: ownership transferred into fn
use(b);                 // ERROR: 'b' was moved into fn
```

A move is a shallow, O(1) transfer (it copies the value's bits and invalidates the source);
it never deep-copies owned heap data and never allocates. This is what makes ownership
transfer free.

### 2.2 `Copy` for trivial values (explicit opt-in)

A type may declare conformance to the built-in **`Copy`** interface. A `Copy` value is
**duplicated** instead of moved: the source stays valid.

```rob
data class Point(x: Int, y: Int) : Copy;

p: Point = Point(1, 2);
q: Point = p;           // COPY: both p and q are valid
use(p);                 // OK
```

Rules for `Copy`:

- All primitives (`Byte`, `Short`, `Int`, `Long`, `Float`, `Double`, `Bool`, `Char`) are
  `Copy` intrinsically.
- A user type may declare `: Copy` **only if every field is `Copy`**; otherwise it is a
  compile error. This keeps `Copy` meaning "trivially, cheaply duplicable with no owned
  resource".
- `Copy` is **explicit** on purpose (it is a visible part of a type's contract, and it
  changes the semantics of every assignment of that type). Adding a non-`Copy` field to a
  `Copy` type is a clear error at the declaration, never a silent switch to move.
- `Copy` implies a bitwise duplicate. It cannot be combined with a custom destructor
  (§ 6), because a type that needs cleanup is not trivially duplicable.

### 2.3 Explicit deep copy: `clone`

For a non-`Copy` type that you genuinely want to duplicate (including its owned heap data),
conform to **`Clonable`** and call `.clone()`. This is always explicit, so the cost of a
deep copy is always visible at the call site.

```rob
a: Buffer = Buffer.ofSize(1024);
b: Buffer = a.clone();  // explicit deep copy (allocates); 'a' still valid
```

| Kind | Assignment does | Source after | Cost |
|------|-----------------|--------------|------|
| non-`Copy` value | move | invalid | O(1), no alloc |
| `Copy` value | duplicate | valid | O(size), no alloc |
| `.clone()` | deep copy | valid | may allocate |
| `shared` value (§ 3) | share (refcount + 1) | valid | atomic-or-not increment |

---

## 3. `shared`: heap, reference-counted ownership

Single ownership cannot express graphs, back-references, or "many readers keep this alive".
For those, `shared T` provides **shared ownership**: the value lives on the heap and is
reference counted.

```rob
node: shared Node = shared Node(value = 1);   // heap-allocated, refcount = 1
alias: shared Node = node;                     // SHARE: refcount = 2, both valid
```

- Assigning a `shared` value does **not** move it; it produces another owner and increments
  the refcount. The value is destroyed when the last owner is dropped (refcount reaches 0).
- `shared` is the one place a user value is heap allocated, and it is spelled in the type,
  satisfying "no hidden allocation" (§ 5).

### 3.1 Inferred atomicity

The refcount is **non-atomic** (fast) when the compiler proves the value never leaves its
originating thread, and **atomic** (thread-safe) when it can cross a thread boundary (for
example, captured by a spawned task, § 8). The choice is made by the compiler and is
invisible in the source: there is no `Rc` vs `Arc` distinction to write.

If the analysis cannot prove single-thread confinement, it conservatively uses the atomic
form. Correctness is never at stake; only the fast path is.

### 3.2 Mutating shared state

A `shared` value may have several live owners, so unrestricted mutation through one owner
would break exclusivity (§ 4) or race across threads. Therefore a `shared` value is
**read-only through its shared handles** by default. To mutate shared state, wrap the
interior in a standard-library cell whose cost matches the sharing:

- single-thread shared: a runtime-checked cell (borrow checked at run time),
- cross-thread shared: a `Mutex`/atomic type.

The cell type is chosen by the programmer and is visible in the field type, so the
synchronization cost is never hidden. (Exact cell APIs are a standard-library concern; this
document only fixes the rule that shared mutation is explicit.)

### 3.3 Cycles

Reference counting does not collect cycles. A cycle of `shared` values leaks. The standard
library provides a **`weak`** handle (a non-owning reference that does not keep the value
alive) to break cycles; back-references in graph structures should use it. The linter warns
on `shared` structures that can form cycles without a `weak` edge.

---

## 4. Mutation and aliasing

### 4.1 Immutable by default, `mut` to opt in

Bindings and fields are **immutable by default**. Reassignment and in-place mutation require
`mut` (this settles the open `mut` question in `grammar.md`):

```rob
mut total: Int = 0;
total += 1;             // OK

name: String = "Ada";
name = "Bob";           // ERROR: 'name' is not 'mut'
```

`mut` expresses *intent to change a binding or field*. It is not a borrow annotation; it
says nothing about aliasing, which is inferred separately (§ 4.2).

### 4.2 Inferred exclusivity (the replacement for `&`/`&mut`)

Instead of explicit borrows, the compiler enforces one invariant by analysis:

> At any point in the program, a value is reached either through **exactly one mutating
> access**, or through **any number of read-only accesses**, never both at once.

This is the same guarantee Rust encodes with `&mut` (unique) versus `&` (shared), but Rob
infers which one applies from how the value is used, and the programmer writes no symbol.

```rob
mut list: List<Int> = List.empty();
list.add(1);            // unique mutable access to 'list' here: OK

first: Int = list.get(0);   // read-only access
list.add(2);                // OK: 'first' is an Int (Copy), not a live view into 'list'
```

When a piece of code would require simultaneous mutable and readable access to the same
value (aliased mutation), the compiler rejects it and points at the conflict, suggesting
either a restructure or `shared` + a cell (§ 3.2). The programmer never sees a lifetime or a
borrow token; they see a plain-language conflict.

---

## 5. No hidden allocation

Every heap allocation in a Rob program is visible in the source:

- `shared T` — allocates a reference-counted box (spelled in the type).
- collection types (`List`, `Map`, `String`, ...) — allocate internally; this is part of
  the type's documented contract, not an effect of ordinary syntax.
- an explicit `unsafe`/FFI boundary.

Ordinary value code (locals, fields, calls, returns, moves, `Copy`) allocates **nothing**.
A **strict lint mode** flags every allocation site (including collection growth) for code
that must be allocation-free, so a hot path can be audited to zero heap traffic.

There is no autoboxing, no implicit `String` concatenation buffer beyond what the operation
documents, and no hidden closure allocation for a lambda that does not escape.

---

## 6. Destruction (deterministic, RAII)

A value is destroyed **when its owner goes out of scope**, deterministically, in reverse
order of declaration within a scope. There is no finalizer thread and no GC pause.

- Destroying a value destroys the values it owns (its fields), recursively.
- A `shared` value is destroyed when its last owner is dropped.
- A type may define a **destructor** (a `Droppable`-style hook) to release a non-memory
  resource (file, socket). A `Copy` type may not have one (§ 2.2).
- Moving a value transfers the destruction obligation to the new owner; a moved-from name is
  never destroyed.

```rob
public fun writeReport(path: String): Unit {
    file: File = File.open(path);   // acquires the handle
    file.writeLine("done");
    // 'file' goes out of scope here -> handle closed automatically
}
```

---

## 7. Inferred lifetimes

Rob performs the region/lifetime analysis that guarantees no reference outlives the value it
points into. **This analysis is entirely internal.** There is no lifetime syntax in the
language: no named lifetimes, no `'a`, nothing generated for the user to accept.

- Function signatures, struct fields, and returns are analyzed by region inference with
  elision-style heuristics extended to cover the common cases.
- The programmer writes ordinary signatures (`fun first(items: List<T>): T`) and the
  compiler proves the result does not dangle.

### 7.1 When inference cannot prove safety

If the compiler cannot prove a value lives long enough, it does **not** ask the programmer
to write a lifetime. It reports a plain-language ownership error and offers two fixes:

1. **Make the value `shared`.** Reference counting keeps it alive for all its owners; the
   compiler notes the added cost.
2. **Restructure ownership** so the value is clearly owned for the needed span (return the
   value by move instead of a view into it, hoist the owner to an outer scope, and so on).

```
error: cannot prove 'node' lives long enough for the returned reference.
  fix 1: make it 'shared Node' (adds a refcount), or
  fix 2: return the node by value so the caller owns it.
note: no lifetime annotation is required or possible; ownership must be made explicit.
```

This keeps the promise absolute: **a Rob programmer never reads or writes a lifetime.** The
escape hatch is always `shared` or a clearer ownership shape, never annotation.

> This inference is the project's headline technical risk. If real programs push too much
> code into the `shared` fallback, the model degrades toward reference counting everywhere.
> Measuring the inference success rate on real programs is a gating concern tracked outside
> this document.

---

## 8. Concurrency and data-race freedom

Data-race freedom falls out of ownership, at compile time:

- **Moving into a task** transfers ownership: a value sent to another thread is no longer
  accessible on the sender, so two threads never hold a mutating path to the same value.
- **Sharing across threads** requires a `shared` value; crossing the boundary switches its
  refcount to atomic (§ 3.1), and mutation still goes through a `Mutex`/atomic cell (§ 3.2).
- A value that is safe to send to another thread is **inferred** (a `Sendable`-style
  property); sending a non-sendable value is a compile error, not a runtime hazard.

```rob
public async fun parallelSum(chunks: List<Buffer>): Long {
    // each chunk is MOVED into its task; no chunk is aliased across threads
    tasks: List<Task<Long>> = chunks.map((c: Buffer) -> spawn(() -> sumOf(c)));
    return await tasks.map((t: Task<Long>) -> await t).sum();
}
```

---

## 9. Worked examples

### 9.1 Move, Copy, clone side by side

```rob
data class Id(value: Long) : Copy;         // trivially copyable
data class Doc(id: Id, body: String);      // owns a String -> not Copy

public fun demo(): Unit {
    a: Id = Id(1);
    b: Id = a;            // COPY: a still valid
    print(a.value);       // OK

    d1: Doc = Doc(Id(2), "hello");
    d2: Doc = d1;         // MOVE: d1 invalid
    // print(d1.body);    // ERROR: d1 was moved
    d3: Doc = d2.clone(); // explicit deep copy; d2 still valid
}
```

### 9.2 Shared graph with a weak back-edge

```rob
public class Tree {
    mut children: List<shared Tree> = List.empty();
    parent: weak Tree?;                    // non-owning back-reference, breaks the cycle

    public fun addChild(child: shared Tree): Unit {
        children.add(child);
    }
}
```

### 9.3 Ownership error and its fix

```rob
// Does NOT compile: returns a view into a value owned by this function.
public fun firstNameBroken(users: List<User>): String {
    u: User = users.get(0);
    return u.name;          // if 'name' were a view into 'u', 'u' dies here
}

// Fix by moving the value out (the String is owned by the caller after return).
public fun firstName(users: List<User>): String {
    u: User = users.get(0);
    return u.name.clone();  // caller owns the returned String
}
```

---

## 10. Open questions

- **MM-Open-1, Inference reach.** Region inference is the headline risk. The elision
  heuristics must be prototyped on real programs; the target is a high share of code needing
  no `shared` fallback. Below target, the fallback becomes "reference counting everywhere".
- **MM-Open-2, Shared mutation cells.** The exact standard-library cell types (single-thread
  runtime-checked cell, cross-thread `Mutex`) and their APIs need specifying. This document
  fixes only that shared mutation is explicit and visible.
- **MM-Open-3, `weak` semantics.** Upgrade of a `weak` handle to a live `shared` (returning
  `shared Tree?` when the target is alive) and its exact API.
- **MM-Open-4, Sendable inference.** The precise rule for which types may cross a thread, and
  whether any type ever needs an explicit opt-out/opt-in.
- **MM-Open-5, `Copy` size limit.** Whether the linter warns when a large struct is declared
  `Copy` (silent expensive copies), and if so at what size threshold.
- **MM-Open-6, Self-referential values.** Whether any safe construct allows a value to hold a
  reference into itself, or whether these always require `shared`.

---

*Last updated: see git history. Any change to the memory model semantics **must** update
this file.*
