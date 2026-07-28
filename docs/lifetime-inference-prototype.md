# GATE: lifetime-inference prototype on the reference programs

> **Status:** gate analysis, paper prototype.
> **Issue:** [#6 GATE: prototype lifetime inference on the 20 reference programs](https://github.com/ErwanHeschung/roblang/issues/6)
> **Corpus:** [`../examples/`](../examples) (the 20 programs from issue #5).
> **Companion:** [`memory-model.md`](memory-model.md) (§ 4 exclusivity, § 7 inferred lifetimes).
>
> **Acceptance:** ≥ 90% of borrows inferred without annotations. If below, pivot to a hybrid
> ownership + transparent ARC model before writing the compiler.

---

## 0. Verdict (read this first)

**PASS on the metric, with a qualified reading.**

- **34 / 35 borrow relationships (97%) are inferred with no annotation** by the elision
  rules below; the remaining one is inferred by the constraint-based region pass (still no
  annotation). By the letter of the acceptance criterion, the gate passes and no pivot is
  forced.
- **But the pass is soft.** The corpus contains **zero borrows stored in a data structure**,
  which is the category that actually forces lifetime annotations in Rust. Every long-lived
  reference in the corpus was expressed with `shared` (refcounting) instead. So the programs
  do not stress the hard case; they route around it by design.
- **The honest conclusion is that this validates the hybrid model, not pure inference.** The
  current design already *is* ownership + explicit ARC (`shared`). Inference handles the easy
  ~97%; `shared` handles the graphs and back-references. That split is exactly the safety net
  the pivot describes, except the ARC is **explicit** (`shared` in the type) rather than
  **transparent**.
- **Recommendation:** accept the gate, keep the **explicit-`shared`** hybrid, and do **not**
  adopt transparent ARC (it would violate the no-hidden-allocation guarantee of
  `memory-model.md` § 5). Before trusting this result enough to build the real checker, run
  the **adversarial corpus** in § 6, which deliberately targets the struct-stored-borrow case
  the current programs avoid.

The rest of this document is the model and the per-program evidence behind that verdict.

---

## 1. What counts as a "borrow"

A **borrow** is a reference into a value the current code does not own, whose validity the
compiler must prove. Three things are **not** borrows and need no lifetime reasoning:

- a **`Copy`** value (it is duplicated, § `type-system.md`),
- a **moved** value (ownership transfers; the source is gone),
- a **`shared`** value (reference counted; it lives as long as any owner, § 3 rules).

So a borrow is a read-only or mutable *view* into caller-owned, non-`Copy`, non-`shared`
data: a method receiver (`this`), a by-reference parameter, a reference returned from a
function, a field projection, or a loop over a collection.

We count borrows at **signature boundaries** (where an annotation would otherwise live).
Local, in-a-single-body borrows are always solved by the intra-procedural pass and are not
the annotation burden; as in Rust, annotations only ever appear on signatures.

---

## 2. The region-inference model (pseudo-code)

Region inference is constraint-based (Polonius-style), computed over the control-flow graph.

```
// A region is a set of program points where a reference must stay valid.
// Every owned value V has an origin region = the scope that created it.
// Every borrow B introduces a region variable r(B).

for each borrow B:
    // (C1) a borrow cannot outlive the value it points into
    r(B)  ⊆  region(owner(B))
    // (C2) a borrow must be live at every point where it is used
    for each use u of B:  point(u) ∈ r(B)

for each returned borrow R that flows from borrows {B1..Bk}:
    // (C3) the result's region is bounded by the inputs it derives from
    r(R)  ⊆  intersection(region(B1) .. region(Bk))

solve:
    least-fixpoint over the CFG for every r(B)   // liveness propagation
check:
    no owner is dropped at a point still inside its borrow's region  // == no dangling
    no two live borrows of the same value conflict (one mutable xor many shared)  // == § 4
```

If the check fails, inference reports an ownership error and offers `shared` or a
restructure (`memory-model.md` § 7.1). There is no annotation to emit.

---

## 3. Extended elision rules

Elision is the set of shortcuts that discharge the common borrow shapes **before** the full
constraint solve, so that ordinary signatures need no annotation. Rules, in priority order:

- **E1 (receiver).** A method's `this` borrow takes the receiver's region. A returned borrow
  with no other source borrows from `this`.
- **E2 (single input).** If a function has exactly one borrowed input, any returned borrow
  takes that input's region.
- **E3 (owned output).** If the return is an owned value (freshly constructed, moved out, or
  `Copy`), it carries no region and inference trivially succeeds.
- **E4 (field projection).** Borrowing a field yields a borrow with the container's region.
- **E5 (shared).** A `shared` value has an unbounded (refcounted) region; it imposes no
  constraint. `weak` upgrades are checked at use, not here.
- **E6 (escaping closure).** A closure that outlives the current scope captures its free
  non-`Copy` variables **by move**, not by borrow. (This is what lets returned/spawned
  closures type-check without lifetimes.)
- **E7 (provenance, the "extended" rule).** If a returned borrow provably derives, by
  intra-procedural dataflow, from a single input among several, it takes that input's region.
  This is the one rule beyond classic Rust elision; it covers the multi-input case that
  Rust would reject and require an annotation for.

E1 through E6 are local pattern matches. E7 needs the dataflow of § 2 but still emits no
annotation.

---

## 4. Per-program analysis

Columns: **B** = borrow relationships at signatures; then how each is discharged. `Copy` /
`move` / `shared` mean "not a borrow". **Elide** = discharged by E1 to E6. **Region** =
needed E7 / the constraint solve. **Annot** = inference failed (would need annotation or a
`shared` fallback).

| # | Program | B | Elide | Region | Annot | Notes |
|---|---------|---|-------|--------|-------|-------|
| 1 | hello_world | 0 | 0 | 0 | 0 | no borrows |
| 2 | fibonacci | 1 | 1 | 0 | 0 | `cache` mut ref, output `Long` owned (E3) |
| 3 | fizzbuzz | 0 | 0 | 0 | 0 | all `Int` (Copy) |
| 4 | temperature | 0 | 0 | 0 | 0 | `Celsius`/`Fahrenheit` are `Copy` |
| 5 | bank_account | 4 | 4 | 0 | 0 | 4 `this` borrows, owned returns (E1, E3) |
| 6 | shapes | 2 | 2 | 0 | 0 | `area`/`describe` on `this` (E1) |
| 7 | quicksort | 2 | 2 | 0 | 0 | `items` single input, owned list out (E2, E3); `compareTo` E1 |
| 8 | binary_search | 1 | 1 | 0 | 0 | `sorted` in, `Int?` out (E3) |
| 9 | generic_stack | 4 | 4 | 0 | 0 | `pop`/`peek`/`size`/`message` on `this`; `peek` return via E1 |
| 10 | linked_list | 2 | 2 | 0 | 0 | nodes are `shared` (E5); `size`/`sumWith` on `this` |
| 11 | binary_tree | 2 | 2 | 0 | 0 | `shared` edges + `weak` parent (E5); `attach*` on `this` |
| 12 | json | 1 | 1 | 0 | 0 | `render` on `this`, owned `String` out (E1, E3) |
| 13 | csv_parser | 2 | 2 | 0 | 0 | `line` in, owned `Person` out; `message` on `this` |
| 14 | word_count | 1 | 1 | 0 | 0 | `text` in, owned `Map` out (E2, E3) |
| 15 | higher_order | 1 | 1 | 0 | 0 | `compose` returns a closure capturing by move (E6) |
| 16 | calculator | 2 | 2 | 0 | 0 | `Expr` children are `shared` (E5); `eval`/`message` on `this` |
| 17 | file_io | 2 | 2 | 0 | 0 | `File` iterator borrows the file, local region (E1) |
| 18 | option_demo | 1 | 0 | 1 | 0 | `lookupNickname(directory, key)`: two ref inputs, result flows only from `directory` (E7) |
| 19 | vectors_raytrace | 4 | 4 | 0 | 0 | `Vec3` args are `Copy`; only `this` borrows remain (E1) |
| 20 | async_fetch | 3 | 3 | 0 | 0 | `spawn` closures capture `u` by move (E6); owned outputs |
| | **Total** | **35** | **34** | **1** | **0** | |

### The one interesting case (program 18)

```rob
fun lookupNickname(directory: Map<String, User>, key: String): String?? { ... }
```

Two borrowed inputs (`directory`, `key`), and a returned borrow. **Classic Rust elision
rejects this** ("two inputs, no `self`, cannot pick a lifetime") and demands an annotation.
Rob's **E7** inspects the body, sees the result flows only from `directory` (via
`directory.get(key)` then a field of the found `User`), never from `key`, and assigns the
result `directory`'s region. No annotation. This is the single case in the corpus that
exercises anything beyond textbook elision, and it is exactly the kind the "extended elision
rules" in the issue are meant to capture.

---

## 5. Result and interpretation

- Borrows total: **35**. Inferred without annotation: **35** (34 by elision, 1 by E7 /
  region). Annotation required: **0**. Inference rate **100%**, or **97%** if E7 is scored as
  "beyond elision" and counted conservatively. Either way **≥ 90%**: the gate passes.

But two facts qualify the number, and a GATE must state them:

1. **The corpus avoids the hard category.** Not one program stores a borrow inside a struct
   or returns a struct that holds a borrow. Every persistent link (linked list, tree,
   expression AST) uses `shared`. In real Rust, struct-held references are the dominant
   source of explicit lifetimes. Their absence here is why the rate is so high.
2. **Author bias.** The same author wrote the programs, the elision rules, and this analysis.
   Programs written to a mental model of the rules will fit the rules. The 100% is therefore
   an upper bound, not an unbiased estimate.

So the honest reading is not "pure inference solves 100% of real code". It is: **the design
already routes the hard cases to explicit `shared` (ARC), and inference cleanly covers what
remains.** That is a hybrid ownership + ARC model. The gate's pivot clause is effectively
already satisfied by the language's own design, with one improvement over the pivot's
wording: the ARC is **explicit** (`shared` appears in the type), which keeps the
no-hidden-allocation guarantee (`memory-model.md` § 5). **Transparent ARC would break that
guarantee and should not be adopted.**

---

## 6. Adversarial corpus (required before trusting this gate)

To make the gate meaningful rather than self-fulfilling, these programs must be written
**independently** and re-run through the model. They target the categories the current
corpus avoids:

1. **Struct holding a borrow.** A `Parser` that holds a reference into the source `String`
   it scans (the canonical lifetime-parameter case). Does inference need a struct-level
   region, and can it be elided?
2. **Return a borrow chosen from two inputs by a runtime condition.** `pick(a, b, cond)`
   returning a reference to `a` or `b`. E7's provenance is ambiguous here; expected to
   require `shared` or a restructure. This is the honest failure case.
3. **Iterator that outlives one mutation.** Build an iterator over a collection, then attempt
   to mutate the collection while it is live. Must be rejected by § 2's conflict check.
4. **Cache returning interior references.** A memo table returning references to stored
   values, then a later insertion. Tests borrow-vs-mutation across calls.
5. **Self-referential value.** A value that wants to point into itself. Expected: only
   expressible via `shared`; confirms the fallback boundary.

**Gate decision rule for the adversarial run:** if, on that corpus, the share of borrows that
need `shared` (rather than being inferred) is small and each `shared` is defensible, keep the
explicit-`shared` hybrid. If `shared` has to be sprinkled everywhere to make ordinary code
compile, that is the real trigger to revisit the model (and only then consider making the ARC
more automatic, weighed against § 5).

---

## 7. Follow-ups

- **LI-1:** Write and independently review the adversarial corpus (§ 6); re-run the model and
  record the rate.
- **LI-2:** Specify E7's provenance dataflow precisely (what "provably derives from a single
  input" means, and its limits).
- **LI-3:** Define the conflict check of § 2 against `memory-model.md` § 4 (exclusivity), with
  the iterator-invalidation case as the worked example.
- **LI-4:** Decide the diagnostic wording when inference falls back to `shared`, so the
  "no lifetime annotation, ever" promise is kept in the error text.
