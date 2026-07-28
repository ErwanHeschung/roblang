# Rob Type System

> **Status:** living document. This file specifies the semantics of Rob's type system.
> The **syntax** it refers to is defined in [`grammar.md`](grammar.md); this file defines
> what the well-formed programs *mean* to the type checker.
>
> **Issue:** [#2 Specify the type system](https://github.com/ErwanHeschung/roblang/issues/2)
> **Scope:** primitives, classes, interfaces, monomorphized generics, explicit types
> everywhere (no inference).
>
> The **memory model** (value vs `shared`, move semantics, lifetimes) is specified
> separately in `memory-model.md`; this file only touches memory semantics where they
> affect typing (identity, equality).

---

## 0. Design principles

1. **Static.** Every expression has a type known at compile time. No runtime type
   information is required for dispatch in the default (non-virtual) case.
2. **Nominal.** Types are equal by name, not by shape. Two classes with identical fields
   are distinct types. Interface conformance is declared, never structural.
3. **Explicit.** Every declaration (local, field, parameter, return, loop element) carries
   a written type. There is no `var` and no declaration-site type inference (see § 6).
4. **Monomorphized.** Generics are compiled by specialization, not erasure (see § 5). A
   type parameter behaves, after compilation, exactly like the concrete type it was
   instantiated with.
5. **Null-safe by construction.** A type `T` never contains the absence of a value; absence
   is a separate type `T?`, which is sugar for the ordinary enum `Option<T>` (see § 4).

---

## 1. Primitive types

Primitives are value types with a fixed size and no identity. They are always copied,
never aliased, and never null (only `T?` can be absent).

### 1.1 Integers

Signed, two's complement, fixed width. No unsigned types in v1.

| Type | Width | Range |
|------|-------|-------|
| `Byte` | 8-bit | -128 .. 127 |
| `Short` | 16-bit | -32 768 .. 32 767 |
| `Int` | 32-bit | -2 147 483 648 .. 2 147 483 647 |
| `Long` | 64-bit | -2^63 .. 2^63 - 1 |

Overflow of a signed integer operation is a **checked error in debug builds** (panic) and
**wraps two's-complement in release builds**, mirroring the safety/performance split of a
systems language. Explicit wrapping/saturating/checked operations are provided as methods
in the standard library (`addWrapping`, `addChecked`, ...), not as operators.

### 1.2 Floating point

IEEE 754 binary floating point.

| Type | Width | Standard |
|------|-------|----------|
| `Float` | 32-bit | IEEE 754 binary32 |
| `Double` | 64-bit | IEEE 754 binary64 |

`Float`/`Double` follow IEEE semantics for `NaN`, infinities, and signed zero. Equality on
floats is IEEE equality (`NaN != NaN`); this is the one primitive whose `==` is not
reflexive, and the linter warns on direct float equality.

### 1.3 Boolean, character, unit

| Type | Meaning |
|------|---------|
| `Bool` | `true` or `false`. Not convertible to/from integers. |
| `Char` | A **Unicode scalar value** (`U+0000 .. U+10FFFF`, excluding surrogates), 32-bit. A single `Char` can hold any code point, including emoji. |
| `Unit` | The type with exactly one value, written `()`. The result type of a function that returns nothing meaningful (the equivalent of `void`). A function with no declared return type returns `Unit`. |

`String` is **not** a primitive: it is a standard-library value type holding UTF-8 bytes.
Because `Char` is a Unicode scalar and `String` is UTF-8, indexing a `String` by `Char` is
not O(1) and is therefore not provided by `[]`; iteration yields `Char` values decoded from
the UTF-8 sequence.

### 1.4 `Nothing` (bottom type)

`Nothing` is the type with **no values**. It is the result type of expressions that never
produce a value: `return`, `break`, `continue`, an infinite loop, or a call that always
diverges. `Nothing` is a subtype of every type (§ 4.3), which is what makes

```rob
name: String = maybe() ?: return "";
```

type-check: the right operand of `?:` has type `Nothing`, assignable to `String`.

---

## 2. Literal typing

Literals are the one place where a value's type is not written explicitly, so their typing
rules must be pinned down precisely. **This is not declaration-site inference** (§ 6): the
declaration's type is still mandatory. It only decides which primitive type a bare literal
denotes.

### 2.1 Default types

An unsuffixed literal used with **no expected type** takes its default:

| Literal | Default type |
|---------|--------------|
| `42`, `0xFF`, `0b1010` | `Long` (64-bit) |
| `3.14`, `6.022e23` | `Double` (64-bit) |
| `'a'`, `'\u{1F600}'` | `Char` |
| `"text"` | `String` |
| `true`, `false` | `Bool` |
| `null` | `Nothing?`, i.e. `None` assignable to any `T?` (§ 4) |

### 2.2 Context coercion

When a **numeric literal** appears where a specific primitive numeric type is expected (a
declaration with a written type, an argument to a typed parameter, an assignment target),
the literal takes that expected type **if the value is representable in it**; otherwise it
is a compile error.

```rob
count: Int = 0;         // literal 0 typed as Int, not Long
ratio: Float = 1.5;     // literal 1.5 typed as Float
tiny: Byte = 200;       // ERROR: 200 not representable in Byte (-128..127)
big: Byte = 100;        // OK
```

This keeps every declaration explicitly typed while avoiding a spurious "Long is not Int"
error on `Int count = 0`. Coercion applies **only to literals**, never to typed values
(§ 3). A `Long` *variable* is never silently usable where an `Int` is expected.

---

## 3. Conversions between primitives

There are **no implicit conversions between distinct primitive types**, not even widening.
This is a deliberate safety choice (it removes a large class of silent bugs and matches the
zero-surprise goal). Conversions are explicit methods:

```rob
n: Int = 300;
w: Long = n.toLong();       // explicit widening
b: Byte = n.toByte();       // explicit narrowing (may truncate; documented)
f: Double = n.toDouble();
c: Char = 65.toChar();      // Int -> Char (checked: must be a valid scalar)
```

`Bool` has no numeric conversion. The only exception to "no implicit conversion" is literal
context coercion (§ 2.2), which is a property of *literals*, not of values.

---

## 4. Optionality and the type lattice

### 4.1 `T?` is `Option<T>`

Absence is not a null sentinel baked into every type; it is an ordinary value of a standard
generic enum:

```rob
public enum Option<T> { Some(value: T), None }
```

`T?` is **syntactic sugar for `Option<T>`**, and the literal `null` is sugar for `None`.
`T` and `T?` are distinct types: a bare value `v: T` is usable where `T?` is expected
because it widens to `Some(v)` (§ 4.3), but not the reverse.

```rob
name:  String  = "Ada";    // a String
found: String? = "Ada";    // Some("Ada")
maybe: String? = null;     // None
name = maybe;              // ERROR: String? (an Option) is not a String
maybe = name;             // OK: "Ada" widens to Some("Ada")
```

Because `T?` is a genuine `Option<T>`, it **nests**, and the two kinds of absence stay
distinct (a nullable-only model such as Kotlin's or Swift's collapses both into one `null`
and cannot tell them apart):

```rob
b1: String?? = Some(null);  // the outer value EXISTS; its inner is absent
b2: String?? = null;        // the outer value does NOT exist
b1 == b2;                    // false
```

This is exactly the "check if it **exists**" versus "check if it **is null**" distinction:
`b1` exists (it is `Some(...)`) even though what it holds is empty. `T??` is
`Option<Option<T>>`, so `T?` never collapses.

`Option<T>` is a value type. For a `shared`/reference payload the compiler uses the
null-pointer niche (a `None` is a null pointer), so `T?` over a reference costs nothing
beyond `T`; for a value payload it is a tag plus the value. Optionality is therefore
zero-overhead in the common reference case and never introduces a hidden allocation.

### 4.2 Checking existence and unwrapping

You test **existence** (is it `Some`?), never a null sentinel. All of the following are
supported; none requires writing `!= null`, though that spelling remains as sugar:

| Form | Result | Meaning |
|------|--------|---------|
| `match (x) { Some(v) -> ..., None -> ... }` | per arm | destructure on existence; binds `v : T` in the `Some` arm |
| `if (v: T = x) { ... }` | binds `v : T` | **optional binding**: the branch runs only if `x` exists (proposed grammar sugar, see note) |
| `x?.m()` | `R?` | call `m` only if `x` exists, else `None` |
| `x ?: y` | common type of `T` and typeof `y` | the value if it exists, else `y` |
| `x!!` | `T` | assert existence; panics at runtime if `None` |
| `if (x != null) { ... }` | `x : T` in branch | sugar for "is `Some`"; flow-narrows to `T` |

Flow narrowing (inside a `Some` arm, an optional binding, or an `if (x != null)` guard,
`x`/`v` has the unwrapped type `T`) refines *usage*, not the *declaration*, so it respects
the explicit-types rule (§ 6).

> **Grammar note.** `match`, `?.`, `?:`, `!!`, and `!= null` already exist in
> [`grammar.md`](grammar.md). The optional-binding form `if (v: T = expr)` is a **proposed
> addition** to the grammar's `ifStmt`/`ifExpr`, tracked as a grammar follow-up; it is not
> yet part of the formal grammar. Existence checking is fully expressible today with
> `match`.

### 4.3 Subtyping

Rob's subtyping relation `<:` is:

1. `T <: T` (reflexive).
2. `Nothing <: T` for every `T` (bottom).
3. `T <: T?` for every `T` (a bare `v : T` widens to `Some(v) : T?`). Note `T?` does not
   collapse: `T?` and `T??` are distinct types (§ 4.1).
4. If `C` declares `: I` (implements interface `I`) then `C <: I`.
5. If `C` declares `: B` (extends class `B`, at most one level, § 5.4) then `C <: B`.
6. Subtyping is **not** propagated through generics: `List<Cat>` is **not** a subtype of
   `List<Animal>` (generics are invariant in v1, § 5.3).

There is no universal top type in v1 other than as a generic bound (`Any` is a standard
interface, not a magic supertype). Primitive types participate in subtyping only via rules
1 through 3.

---

## 5. Classes, interfaces, generics

### 5.1 Classes as nominal types

A `class C` introduces a type named `C`. Its type identity is its name plus its type
arguments (for a generic class). Fields and methods are accessed nominally. A class is a
**value type** by default (stack/inline); this affects identity and equality (§ 7) but not
the typing rules here.

### 5.2 Interfaces (traits)

An `interface I` introduces a type. A class conforms to `I` only by **declaring** `: I` and
providing an implementation for every abstract member; conformance is never inferred from
shape. Interfaces are the primary polymorphism mechanism. An interface may provide default
method bodies; a conforming class may override them.

A variable of interface type (`shape: Drawable`) is a form of polymorphism that requires
**dynamic dispatch**; the compiler represents it as a fat pointer (data + method table).
Calls through a concrete class type are statically dispatched. (How this interacts with
value layout is a memory-model concern.)

#### The `This` self-type

Inside an interface (or `open` class), `This` denotes "the concrete type that conforms".
When a class `C` implements the interface, every `This` in the inherited signatures is
bound to `C`.

```rob
public interface Comparable {
    fun compareTo(other: This): Int;      // 'This' = the implementing type
}

public data class Version(major: Int, minor: Int) : Comparable {
    public fun compareTo(other: Version): Int {   // 'This' resolved to Version
        return if (major != other.major) { major - other.major }
               else { minor - other.minor };
    }
}
```

`This` is **invariant in parameter position** (you can only `compareTo` your own type) and
**covariant in return position** (a builder can return `This` and callers see the concrete
type). It is resolved during monomorphization, so it carries no runtime cost.

### 5.3 Generics and monomorphization

A generic declaration is a template over type parameters:

```rob
public class Box<T>(value: T) { public fun get(): T { return value; } }
```

- **Monomorphization:** for each distinct set of type arguments actually used, the compiler
  emits a specialized copy (`Box<Int>`, `Box<String>` are separate compiled types). There
  is **no type erasure**: `T` is a real, fully-known type in the generated code, so type
  arguments are available, fields of type `T` are laid out inline, and there is no boxing.
- **Bounds:** a type parameter may require interface conformance: `<T: Comparable>`, or
  several: `<T: Comparable + Hashable>`. Inside the body, `T` may use exactly the members
  guaranteed by its bounds (and the members of `Any`). An unbounded `T` supports only what
  every type supports.
- **Variance:** generic types are **invariant** in v1 (§ 4.3 rule 6). `Box<Cat>` is neither
  a subtype nor a supertype of `Box<Animal>`. Declaration-site or use-site variance is a
  post-v1 consideration (see Open questions).
- **Instantiation legality:** `Box<T>` may be instantiated with any `T` satisfying its
  bounds; violating a bound is a compile error reported at the use site.

Monomorphization is purely a compilation strategy; at the type-checking level a generic is
checked once against its bounds, then each instantiation is checked for bound satisfaction.

### 5.4 Inheritance

Implementation inheritance is limited to **one level** and is discouraged (the linter warns;
prefer interfaces). A class may extend at most one base class, which must be `open`, and may
additionally implement any number of interfaces. Overriding a base method requires the base
method to be `open` and the override to be marked `override` (static dispatch by default,
virtual is opt-in). The type rules for overrides:

- The override's parameter types are **identical** to the base (no parameter variance).
- The override's return type may be a **subtype** of the base return type (covariant
  return, including `This`).
- The override may not widen nullability (cannot return `T?` where the base returns `T`).

---

## 6. Explicit typing rule (no inference)

The checker requires a written type in every **binding position**:

| Position | Type required? |
|----------|:--------------:|
| Local declaration (`x: Int = ...`) | yes |
| Local declaration without initializer (`x: Int;`) | yes |
| Field | yes |
| Function/method parameter | yes |
| Function/method return | yes (absence means `Unit`) |
| Lambda parameter | yes |
| Loop element (`for (i: Int in ...)`) | yes |
| Type parameter bound | optional (unbounded allowed) |

What the checker still computes on its own (this is **not** declaration inference):

- the type of every **expression** (needed to check assignability),
- **literal** types via § 2,
- **flow narrowing** of nullability and type tests within a scope (§ 4.2),
- **generic argument** types when they are unambiguous from the value arguments of a call
  (e.g. `Box(42)` infers `Box<Long>`); an explicit `Box<Int>(42)` is always allowed and is
  required when the arguments do not determine the type parameters.

The guiding line: **a reader never has to run type inference in their head to know the type
of a named thing.** Every `name` has its type spelled next to it; only anonymous
intermediate expressions are typed by the compiler.

---

## 7. Equality and identity

- **Value types** (primitives, plain classes, data classes) have **no identity**; two
  values are interchangeable when equal. `==` on a plain class is **memberwise structural
  equality**; on a `data class` it is generated and guaranteed structural.
- **`shared` values** additionally have reference identity; `==` remains structural, a
  separate reference-identity check is available via the standard library.
- `==` requires both operands to have a common type; comparing unrelated types is a compile
  error rather than always-false.
- Overriding equality is done by conforming to the standard `Equatable` interface; the
  operator `==` dispatches to it.

---

## 8. Worked examples

### 8.1 Primitives, literals, coercion, explicit conversion

```rob
public fun sizes(): Unit {
    b: Byte   = 127;            // literal coerced to Byte
    s: Short  = 1000;
    i: Int    = 42;            // coerced to Int, not the Long default
    l: Long   = 42;            // Long
    f: Float  = 1.5;
    d: Double = 3.14;          // Double default
    wide: Long = i.toLong();   // explicit widening; 'i' alone is not a Long
}
```

### 8.2 Generic container with a bound and `This`

```rob
public interface Hashable { fun hash(): Long; }

public class Set<T: Hashable>() {
    private mut items: List<T> = List.empty();

    public fun add(item: T): Bool {
        for (existing: T in items) {
            if (existing.hash() == item.hash()) { return false; }
        }
        items.add(item);
        return true;
    }
}
```
`Set<T>` is monomorphized per element type; `T.hash()` is legal because the bound
`T: Hashable` guarantees it.

### 8.3 Optionality, existence, narrowing, `Nothing`

```rob
public fun lengthOr(text: String?, fallback: Int): Int {
    match (text) {
        Some(s) -> return s.length(),   // exists: 's' bound as String
        None    -> return fallback      // does not exist
    };
}

public fun require(value: String?): String {
    return value ?: panic("missing");   // panic(): Nothing, so result is String
}

// Existence vs emptiness: a cache lookup can distinguish
// "no entry" from "entry present but holds nothing".
public fun report(lookup: String??): String {
    return match (lookup) {
        None       -> "no entry",       // the key is absent
        Some(None) -> "entry is empty", // the key exists, value absent
        Some(Some(v)) -> "value ${v}"
    };
}
```

---

## 9. Open questions

- **TS-Open-1, Overflow policy.** Debug-panic / release-wrap is proposed (§ 1.1). Confirm
  against the perf targets, and decide whether a global `overflow = checked|wrapping` build
  flag exists.
- **TS-Open-2, Variance.** Generics are invariant in v1. Decide whether declaration-site
  variance (`out`/`in`) is a v1.x or v2 feature.
- **TS-Open-3, `Any` and top type.** Is there a universal `Any` interface all types conform
  to, and does it include primitives? Needed to define unbounded generic capabilities
  precisely.
- **TS-Open-4, Numeric literal default.** Integer literals default to `Long` (§ 2.1). This
  differs from Java/Kotlin (`Int`/32-bit). Confirm; a 32-bit default is the main alternative.
- **TS-Open-5, `This` in non-final classes.** Semantics of `This` under one-level
  inheritance (§ 5.4) versus interfaces need a couple of edge cases nailed down (e.g. a base
  method returning `This` called on a subclass value).
- **TS-Open-6, Operator overloading.** `==` dispatches to `Equatable`; decide whether other
  operators (`+`, `<`) are overloadable via interfaces (`Addable`, `Comparable`) or reserved
  to built-ins.
- **TS-Open-7, Optional-binding syntax.** `T?` is `Option<T>` (§ 4.1), so existence checks
  work today via `match`. The ergonomic `if (v: T = expr)` binding (§ 4.2) still needs to be
  ratified and added to `grammar.md`; confirm the exact form (also `while (v: T = expr)`?).

---

*Last updated: see git history. Any change to the type system semantics **must** update
this file.*
