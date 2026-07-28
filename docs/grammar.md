# Rob Grammar & Syntax Tracker

> **Status:** living document. This file is the **single source of truth** for the entire
> Rob syntax. Any change to the syntax must be reflected here.
>
> **Issue:** [#1 Write EBNF grammar for the full MVP syntax](https://github.com/ErwanHeschung/roblang/issues/1)

---

## 0. Locked design decisions

Some early syntax sketches were **mutually inconsistent**. The decisions below resolve
those ambiguities and govern the whole grammar that follows.

| # | Question | Decision | Rationale |
|---|----------|----------|-----------|
| D1 | Declaration order (`name: Type` vs `Type name`) | **`name: Type` everywhere**: fields, locals, parameters, loop elements | Single, unambiguous with generics (`<`), Kotlin-consistent. Both forms had appeared (`repo: OrderRepository` and `List<Order> orders`). |
| D2 | Statement termination | **Mandatory semicolon `;`** | Whitespace-insensitive, formally simple to specify. |
| D3 | MVP scope | classes, interfaces, methods, fields, generics, statements, expressions, literals, plus data classes, async/await, enums with `match`, modules/imports | Explicit choice for issue #1. |
| D4 | Function keyword | `fun` | Java/Kotlin familiarity. |
| D5 | Nullability | Non-nullable types by default, `?` suffix (`T?`) | Null safety. Operators `?.`, `?:`, `!!`. |
| D6 | No type inference on declarations | The type is **mandatory** on every declaration (local, field, parameter, loop element). No `var`. | Readability: types explicit everywhere. |
| D7 | Shared allocation | Type qualifier `shared T` (opt-in reference counting) | No hidden heap allocation; sharing is always visible in the code. |
| D8 | Dispatch | Static by default; virtual is opt-in via `open` (declaration) plus `override` (redefinition) | Zero-cost by default, opt into virtual dispatch. |

### Flagged assumptions (to confirm)

- **`mut`**: local bindings and fields are **immutable by default**; the `mut` prefix allows
  reassignment. Required for a real language but not yet pinned down, to be validated.
  *(see [Open‑1](#open))*
- **String interpolation** `"...${expr}..."`: included (readability, Kotlin style).
- **`match`** (rather than `when`) chosen to align with the "Rust + Object" positioning.
- **`static`** and `init { }`: included minimally for class members and initialization.

---

## 1. Notation

Grammar expressed in **W3C-style EBNF**:

| Form | Meaning |
|------|---------|
| `::=` | rule definition |
| `'x'` | literal terminal |
| `A B` | sequence (A followed by B) |
| `A \| B` | alternative |
| `( … )` | grouping |
| `A?` | optional (0 or 1) |
| `A*` | repetition (0 or more) |
| `A+` | repetition (1 or more) |
| `[a-z]` | character class (lexical grammar) |
| `/* … */` | grammar comment |

Convention: **lexical non-terminals** (tokens) are in `UPPERCASE`, **syntactic
non-terminals** in `camelCase`.

---

## 2. Lexical grammar (tokens)

```ebnf
/* Whitespace & comments: ignored (except inside strings) */
WHITESPACE  ::= (' ' | '\t' | '\r' | '\n')+
LINE_COMMENT  ::= '//' (~('\n'))*
BLOCK_COMMENT ::= '/*' ( BLOCK_COMMENT | ~('*/') )* '*/'   /* nested */

/* Identifiers & keywords */
IDENT       ::= (LETTER | '_') (LETTER | DIGIT | '_')*
LETTER      ::= [A-Za-z]
DIGIT       ::= [0-9]

/* An IDENT cannot be a reserved keyword (§ 2.1). */

/* Integer literals */
INT_LIT     ::= DEC_LIT | HEX_LIT | OCT_LIT | BIN_LIT
DEC_LIT     ::= DIGIT (DIGIT | '_')*
HEX_LIT     ::= '0x' HEXDIGIT (HEXDIGIT | '_')*
OCT_LIT     ::= '0o' [0-7] ([0-7] | '_')*
BIN_LIT     ::= '0b' [01] ([01] | '_')*
HEXDIGIT    ::= [0-9a-fA-F]

/* Float literals */
FLOAT_LIT   ::= DEC_LIT '.' DEC_LIT EXPONENT?
              | DEC_LIT EXPONENT
EXPONENT    ::= ('e' | 'E') ('+' | '-')? DEC_LIT

/* Character & string literals */
CHAR_LIT    ::= "'" (ESCAPE | ~("'" | '\\')) "'"
STRING_LIT  ::= '"' STRING_PART* '"'
STRING_PART ::= ESCAPE
              | INTERPOLATION
              | ~('"' | '\\' | '$')
INTERPOLATION ::= '${' /* expression (§ 6) */ '}'
              | '$' IDENT
ESCAPE      ::= '\\' ('n' | 't' | 'r' | '\\' | '"' | "'" | '0' | '$'
                      | 'u' '{' HEXDIGIT+ '}')

/* Boolean & null literals */
BOOL_LIT    ::= 'true' | 'false'
NULL_LIT    ::= 'null'
```

### 2.1 Reserved keywords

```
package  import  as
class  interface  enum  data  fun  init
public  private  open  override  abstract  static
shared  mut
val                                   /* reserved, unused (D6), forbidden as an identifier */
if  else  while  for  in  return  break  continue  match
async  await
is  true  false  null  this  super
```

---

## 3. File structure (modules)

```ebnf
compilationUnit ::= packageDecl? importDecl* topLevelDecl* EOF

packageDecl     ::= 'package' qualifiedName ';'

importDecl      ::= 'import' qualifiedName ('.' '*')? ('as' IDENT)? ';'

qualifiedName   ::= IDENT ('.' IDENT)*

topLevelDecl    ::= classDecl
                  | interfaceDecl
                  | enumDecl
                  | functionDecl
                  | constDecl

constDecl       ::= modifiers 'static'? IDENT ':' type '=' expr ';'
```

---

## 4. Declarations

### 4.1 Modifiers

```ebnf
modifiers   ::= modifier*
modifier    ::= 'public' | 'private'      /* visibility */
              | 'open'                     /* allows override, virtual dispatch */
              | 'override'                 /* redefines an 'open' member */
              | 'abstract'                 /* member/class without implementation */
              | 'static'                   /* class member (not bound to an instance) */
```

### 4.2 Classes

```ebnf
classDecl   ::= modifiers 'class' IDENT typeParams? primaryCtor? superClause? classBody?

primaryCtor ::= '(' ctorParams? ')'
ctorParams  ::= ctorParam (',' ctorParam)*
ctorParam   ::= modifiers 'mut'? IDENT ':' type ('=' expr)?   /* param = field if modifiers/mut present */

superClause ::= ':' type (',' type)*      /* 0..1 super-class plus interfaces */

classBody   ::= '{' member* '}'
```

### 4.3 Data classes

```ebnf
/* Structural equality, destructuring and toString generated. */
dataDecl    ::= modifiers 'data' 'class' IDENT typeParams? '(' ctorParams ')' superClause? classBody?
```
> `dataDecl` is a production of `topLevelDecl` (added to the list below).

### 4.4 Interfaces

```ebnf
interfaceDecl ::= modifiers 'interface' IDENT typeParams? superClause? interfaceBody

interfaceBody ::= '{' member* '}'
/* Methods without a body are abstract; with a body they are default methods. */
```

### 4.5 Enums (sum types)

```ebnf
enumDecl    ::= modifiers 'enum' IDENT typeParams? superClause? '{' enumBody '}'

enumBody    ::= enumVariant (',' enumVariant)* ','? (';' member*)?

enumVariant ::= IDENT ('(' variantFields ')')?
variantFields ::= variantField (',' variantField)*
variantField  ::= IDENT ':' type            /* named field, Rust-style struct variant */
```

### 4.6 Members (fields, methods, init)

```ebnf
member      ::= fieldDecl
              | methodDecl
              | initBlock

fieldDecl   ::= modifiers 'mut'? IDENT ':' type ('=' expr)? ';'     /* D1, D6, D7 via 'type' */

methodDecl  ::= modifiers 'async'? 'fun' IDENT typeParams?
                '(' params? ')' (':' type)? funcBody

funcBody    ::= block | ';'                 /* ';' means abstract / no body */

initBlock   ::= 'init' block

params      ::= param (',' param)*
param       ::= 'mut'? IDENT ':' type ('=' expr)?
```

### 4.7 Free (top-level) functions

```ebnf
functionDecl ::= modifiers 'async'? 'fun' IDENT typeParams?
                 '(' params? ')' (':' type)? funcBody
```

> **Note:** `topLevelDecl` (§ 3) reads in practice as:
> `classDecl | dataDecl | interfaceDecl | enumDecl | functionDecl | constDecl`.

---

## 5. Types & generics

```ebnf
type        ::= 'shared'? nonNullType '?'?      /* '?' = nullable ; 'shared' = reference-counted heap */

nonNullType ::= namedType
              | funcType
              | tupleType
              | '(' type ')'

namedType   ::= qualifiedName typeArgs?

funcType    ::= '(' (type (',' type)*)? ')' '->' type

tupleType   ::= '(' type ',' type (',' type)* ')'

/* Generics */
typeParams  ::= '<' typeParam (',' typeParam)* '>'
typeParam   ::= IDENT (':' typeBound ('+' typeBound)*)?   /* bounds = interfaces */
typeBound   ::= namedType

typeArgs    ::= '<' type (',' type)* '>'
```

Compile-time monomorphization: semantic, no grammar impact.

---

## 6. Statements

```ebnf
block       ::= '{' statement* '}'

statement   ::= varDeclStmt
              | ';'                          /* empty statement */
              | block
              | ifStmt
              | whileStmt
              | forStmt
              | matchStmt
              | returnStmt
              | breakStmt
              | continueStmt
              | exprStmt

varDeclStmt ::= 'mut'? IDENT ':' type ('=' expr)? ';'   /* D1/D6: type mandatory */

exprStmt    ::= expr ';'                       /* includes assignments & calls (§ 7) */

ifStmt      ::= 'if' '(' expr ')' block ('else' (ifStmt | block))?

whileStmt   ::= 'while' '(' expr ')' block

forStmt     ::= 'for' '(' IDENT ':' type 'in' expr ')' block   /* iterates over an Iterable */

returnStmt  ::= 'return' expr? ';'
breakStmt   ::= 'break' ';'
continueStmt::= 'continue' ';'

matchStmt   ::= matchExpr ';'?                 /* match usable as a statement */
```

> `if` and `match` also exist in **expression position** (§ 7.3). In statement position,
> the `ifStmt` above is preferred (no `;` required after the block).

---

## 7. Expressions

Precedence from **lowest** (top) to **highest** (bottom). Each level reduces to the next.

```ebnf
expr            ::= assignExpr

assignExpr      ::= elvisExpr (assignOp assignExpr)?
assignOp        ::= '=' | '+=' | '-=' | '*=' | '/=' | '%='

elvisExpr       ::= orExpr ('?:' orExpr)*          /* Elvis operator */

orExpr          ::= andExpr ('||' andExpr)*
andExpr         ::= eqExpr ('&&' eqExpr)*
eqExpr          ::= compExpr (('==' | '!=') compExpr)*
compExpr        ::= typeCheckExpr (('<' | '>' | '<=' | '>=') typeCheckExpr)*
typeCheckExpr   ::= rangeExpr (('is' | 'as') type)?    /* type test / cast */
rangeExpr       ::= addExpr ('..' addExpr)?            /* range (for loops) */
addExpr         ::= mulExpr (('+' | '-') mulExpr)*
mulExpr         ::= unaryExpr (('*' | '/' | '%') unaryExpr)*

unaryExpr       ::= ('!' | '-' | '+' | 'await') unaryExpr
                  | postfixExpr

postfixExpr     ::= primaryExpr postfixOp*
postfixOp       ::= '.'  IDENT                       /* member access      */
                  | '?.' IDENT                       /* safe access */
                  | callSuffix                       /* call               */
                  | '[' expr ']'                     /* indexing           */
                  | '!!'                             /* non-null assertion */

callSuffix      ::= typeArgs? '(' args? ')'
args            ::= arg (',' arg)*
arg             ::= (IDENT '=')? expr                /* named arguments allowed */
```

### 7.1 Primary expressions

```ebnf
primaryExpr ::= literal
              | IDENT
              | 'this'
              | 'super'
              | '(' expr ')'                         /* parentheses */
              | tupleExpr
              | lambdaExpr
              | ifExpr
              | matchExpr
```

### 7.2 Lambdas & tuples

```ebnf
lambdaExpr  ::= '(' lambdaParams? ')' '->' (expr | block)
lambdaParams::= lambdaParam (',' lambdaParam)*
lambdaParam ::= IDENT ':' type                       /* explicit types (D1/D6) */

tupleExpr   ::= '(' expr ',' expr (',' expr)* ')'
```

### 7.3 `if` / `match` in expression position

```ebnf
ifExpr      ::= 'if' '(' expr ')' block 'else' (ifExpr | block)

matchExpr   ::= 'match' '(' expr ')' '{' matchArm (',' matchArm)* ','? '}'
matchArm    ::= pattern ('if' expr)? '->' (expr | block)   /* optional guard */
```

### 7.4 Patterns

```ebnf
pattern     ::= '_'                                  /* wildcard           */
              | literal                              /* literal pattern    */
              | variantPattern                       /* enum variant       */
              | tuplePattern
              | IDENT                                 /* variable binding   */

variantPattern ::= qualifiedName ('(' fieldPattern (',' fieldPattern)* ')')?
fieldPattern   ::= (IDENT '=')? pattern

tuplePattern   ::= '(' pattern ',' pattern (',' pattern)* ')'
```

---

## 8. Literals

```ebnf
literal ::= INT_LIT
          | FLOAT_LIT
          | STRING_LIT
          | CHAR_LIT
          | BOOL_LIT
          | NULL_LIT
```

---

## 9. Worked examples (on-paper parse)

### 9.1 The reference class snippet (per D1/D2)

```rob
class OrderService {
    private repo: OrderRepository;

    public fun totalFor(customer: Customer): Money {
        orders: List<Order> = repo.findBy(customer);
        return orders.map((o: Order) -> o.total).sum();
    }
}
```

Derivation: `classDecl` produces `member`(`fieldDecl`) plus `member`(`methodDecl`); the body
produces `varDeclStmt` (`orders: List<Order> = …`) plus `returnStmt` whose expression is a
`postfixExpr` chaining `.map(lambdaExpr)` then `.sum()`. ✅

### 9.2 Enum + match + generics + nullability

```rob
enum Shape {
    Circle(radius: Float),
    Rect(width: Float, height: Float);

    public fun area(): Float {
        return match (this) {
            Circle(radius)      -> 3.14159 * radius * radius,
            Rect(width, height) -> width * height
        };
    }
}

public fun describe(shape: Shape?): String {
    return shape?.area()?.toString() ?: "unknown";
}
```
Covers: `enumDecl` with field-carrying variants, `matchExpr`, `variantPattern`, `?.`, `?:`. ✅

---

## 10. Acceptance tracking, 20 reference programs

> **Issue #1 acceptance criterion:** "grammar parses (on paper) all 20 reference programs".
> The 20 programs are a separate deliverable; this table will track them once they exist.
> Check a box when a manual derivation has been validated against this grammar.

| # | Reference program | Key constructs | Parse OK |
|---|-------------------|----------------|:--------:|
| 1 | _(to create)_ | | ☐ |
| 2 | _(to create)_ | | ☐ |
| … | … | | ☐ |
| 20 | _(to create)_ | | ☐ |

Examples in § 9 already validated manually against the grammar.

---

## 11. Open questions

<a name="open"></a>

- **Open‑1, Mutability (`mut`).** Proposed model: immutable by default, `mut` opt-in.
  To confirm against the ownership model. Not yet decided.
- **Open‑2, Secondary constructors.** Only the primary constructor plus `init` is specified.
  Are overloaded constructors needed?
- **Open‑3, Method genericity vs `where` bounds.** Inline bounds (`<T: Ord>`) chosen;
  `where` clause not included for the MVP.
- **Open‑4, Collection literals.** Currently via calls (`List<Int>` built by functions).
  No dedicated `[…]` / `{…}` literal in v1.
- **Open‑5, Default visibility** (public or private when no modifier is given).

---

## 12. Complete syntax examples (every construct)

One annotated snippet per area. Together they exercise every production in the grammar.

### 12.1 Modules: package & imports

```rob
package app.orders;

import std.collections.List;
import std.collections.*;
import std.io.Console as Term;
```
Covers: `packageDecl`, `importDecl` (plain, wildcard `.*`, aliased `as`), `qualifiedName`.

### 12.2 Top-level constants & free functions

```rob
public static PI: Float = 3.14159;
private static MAX_RETRIES: Int = 3;

public fun max<T: Comparable>(a: T, b: T): T {
    return if (a > b) { a } else { b };
}

public fun greet(name: String = "world"): String {
    return "hello, ${name}!";
}
```
Covers: `constDecl` (with `static`, both visibilities), `functionDecl`, generic function
with a bound, `ifExpr`, default parameter value, string interpolation `${…}`.

### 12.3 Interfaces: generics, bounds, default & abstract methods

```rob
public interface Repository<K, V: Entity> {
    fun findById(id: K): V?;
    fun all(): List<V>;

    public fun exists(id: K): Bool {
        return findById(id) != null;
    }
}

public interface Comparable {
    fun compareTo(other: This): Int;
}
```
Covers: `interfaceDecl`, `typeParams` with a bound (`V: Entity`), abstract method
(`funcBody = ';'`), default method (with body), nullable return `V?`.

### 12.4 Enums: variants with fields, methods, match with guards

```rob
public enum Json {
    Null,
    Bool(value: Bool),
    Number(value: Float),
    Text(value: String),
    Array(items: List<Json>);

    public fun isEmpty(): Bool {
        return match (this) {
            Null            -> true,
            Text(value)     -> value.length() == 0,
            Array(items) if items.length() == 0 -> true,
            _               -> false
        };
    }
}
```
Covers: `enumDecl`, unit variant (`Null`), field-carrying variants, `member*` after `;`,
`matchExpr` as expression, `variantPattern` with binding, guard (`if …`), wildcard `_`.

### 12.5 Data classes

```rob
public data class Point<T: Numeric>(x: T, y: T);

public data class User(
    public id: Int,
    public mut name: String,
    email: String?
) {
    public fun isAnonymous(): Bool {
        return name.length() == 0;
    }
}
```
Covers: `dataDecl` (empty body form and body form), generic data class with bound,
`ctorParam` with visibility, `mut`, and a nullable field.

### 12.6 Classes: inheritance, generics, ctor, fields, init, static, dispatch

```rob
public open class Animal(private name: String) {
    open fun speak(): String {
        return "...";
    }
}

public class Cache<K, V>(capacity: Int) : Animal("cache"), Repository<K, V> {
    private mut entries: Map<K, V> = Map.empty();
    private capacity: Int = capacity;
    static hits: Int = 0;

    init {
        entries = Map.withCapacity(capacity);
    }

    public override fun speak(): String {
        return "cache of size ${entries.size()}";
    }

    public fun findById(id: K): V? {
        return entries.get(id);
    }

    public fun all(): List<V> {
        return entries.values();
    }

    public async fun warm(source: shared Repository<K, V>): Unit {
        items: List<V> = await source.all();
        for (item: V in items) {
            entries.put(item.key(), item);
        }
    }
}
```
Covers: `open` class, primary constructor, `superClause` (super-class + interface),
fields (`mut`, `static`, initialized from a ctor param), `initBlock`, `override`,
`async fun`, `await`, `for … in`, `shared` parameter type, nullable return, `Unit`.

### 12.7 Statements: all forms

```rob
public fun statements(n: Int): Int {
    mut total: Int = 0;                 // mutable var decl with init
    seen: Bool = false;                 // immutable var decl
    result: Int;                        // var decl without initializer
    ;                                   // empty statement

    for (i: Int in 0..n) {              // for-in over a range
        if (i % 2 == 0) {
            total += i;                 // augmented assignment
        } else if (i == 7) {
            continue;                   // continue
        } else {
            total = total + 1;          // plain assignment
        }
    }

    while (total > 100) {
        total -= 10;
        if (total == 0) { break; }      // break
    }

    match (n) {                         // match as a statement
        0 -> { result = 0; },
        _ -> { result = total; }
    };

    return result;                      // return with value
}
```
Covers: `varDeclStmt` (mut / immutable / no initializer), empty statement, `forStmt` with
`rangeExpr`, `ifStmt` with `else if` chain, `whileStmt`, `break`, `continue`, `matchStmt`,
assignment and all augmented `assignOp` forms, `returnStmt`.

### 12.8 Expressions & operators: full precedence ladder

```rob
public fun expressions(a: Int, b: Int, flag: Bool, obj: Widget?): Bool {
    sum: Int = a + b * 2 - a / 2 % 3;             // arithmetic precedence
    span: Range = a..b;                           // range expression
    cmp: Bool = a < b && b <= 10 || !flag;        // comparison / logical
    eq: Bool = a == b != flag;                    // equality chain
    checked: Bool = obj is Widget;                // type test
    casted: Widget = obj as Widget;               // cast
    name: String = obj?.label() ?: "none";        // safe call + elvis
    forced: Int = obj!!.id;                       // non-null assertion
    nested: Int = (a + b) * (a - b);              // parenthesized
    indexed: Int = table[a][b];                   // indexing
    called: Int = compute(x = a, y = b);          // named arguments
    return cmp && eq && checked;
}
```
Covers: arithmetic (`+ - * / %`), `rangeExpr` (`..`), comparison, `&&`/`||`, `!`,
equality, `is`, `as`, `?.`, `?:`, `!!`, parentheses, indexing `[]`, call with named args.

### 12.9 Lambdas, tuples, function types

```rob
public fun higherOrder(): (Int) -> Int {
    adder: (Int, Int) -> Int = (x: Int, y: Int) -> x + y;   // multi-line-free lambda
    logger: (String) -> Unit = (msg: String) -> {           // block-bodied lambda
        Term.print(msg);
    };
    pair: (Int, String) = (1, "one");                       // tuple value
    first: Int = pair.0;
    return (n: Int) -> adder(n, 1);                         // returns a lambda
}
```
Covers: `funcType` (as return type and local type), `lambdaExpr` (expression body and
block body), `lambdaParam` with explicit types, `tupleExpr`, `tupleType`, tuple access.

### 12.10 Patterns: every kind

```rob
public fun classify(pair: (Json, Int)): String {
    return match (pair) {
        (Json.Null, 0)               -> "empty",          // tuple + variant + literal
        (Json.Number(value), count)  -> "num ${count}",   // nested variant binding
        (Json.Text(value), _)        -> value,            // wildcard element
        (other, count) if count > 10 -> "many",           // binding + guard
        _                            -> "other"           // wildcard
    };
}
```
Covers: `tuplePattern`, `variantPattern` (qualified, with field binding), literal pattern,
wildcard `_`, variable binding, guard, and nested patterns.

### 12.11 Literals: every form

```rob
public fun literals(): Unit {
    dec: Int = 1_000_000;
    hex: Int = 0xFF_EC;
    oct: Int = 0o755;
    bin: Int = 0b1010_0001;
    real: Float = 6.022e23;
    small: Float = 3.14;
    ch: Char = '\n';
    unicodeCh: Char = '\u{1F600}';
    text: String = "line1\ttab and \"quotes\" and $small done";
    yes: Bool = true;
    nope: Bool = false;
    nothing: String? = null;
}
```
Covers: `INT_LIT` (dec with `_`, hex, oct, bin), `FLOAT_LIT` (exponent and fractional),
`CHAR_LIT` (escape and `\u{…}`), `STRING_LIT` (escapes + `$ident` interpolation),
`BOOL_LIT`, `NULL_LIT`.

---

*Last updated: see git history. Any change to the syntax **must** update this file.*
