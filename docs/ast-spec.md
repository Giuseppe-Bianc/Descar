# Specifica Tecnica: Full Typed AST del linguaggio Descar

**Versione:** 1.0  
**Data:** 2026-09-28  
**Stato:** Normativa

---

## Indice

1. [Convenzioni e notazione](#1-convenzioni-e-notazione)
2. [Architettura generale dell'AST](#2-architettura-generale-dellast)
3. [Infrastruttura di localizzazione](#3-infrastruttura-di-localizzazione)
4. [Valori numerici (`Number`)](#4-valori-numerici-number)
5. [Valori letterali (`LiteralValue`)](#5-valori-letterali-literalvalue)
6. [Operatori binari (`BinaryOp`)](#6-operatori-binari-binaryop)
7. [Operatori unari (`UnaryOp`, `UnaryOpSide`)](#7-operatori-unari-unaryop-unaryopside)
8. [Tipi (`Type`)](#8-tipi-type)
9. [Parametri di funzione (`Parameter`)](#9-parametri-di-funzione-parameter)
10. [Binding di variabile (`VarBinding`)](#10-binding-di-variabile-varbinding)
11. [Ramo alternativo (`ElseBranch`)](#11-ramo-alternativo-elsebranch)
12. [Espressioni (`Expr`)](#12-espressioni-expr)
13. [Istruzioni (`Stmt`)](#13-istruzioni-stmt)
14. [Sistema di tipizzazione](#14-sistema-di-tipizzazione)
15. [Regole di validità semantica](#15-regole-di-validit%C3%A0-semantica)
16. [Gerarchia di precedenza degli operatori](#16-gerarchia-di-precedenza-degli-operatori)
17. [Tabella dei simboli e scoping](#17-tabella-dei-simboli-e-scoping)
18. [Codici di errore](#18-codici-di-errore)
19. [Vincoli strutturali di validità dell'AST](#19-vincoli-strutturali-di-validit%C3%A0-dellast)
20. [Casi limite e ambiguità risolte](#20-casi-limite-e-ambiguit%C3%A0-risolte)

---

## 1. Convenzioni e notazione

### 1.1 Notazione dei tipi

I tipi strutturali sono descritti con la seguente sintassi pseudo-Rust:

```text
NomeTipo {
    campo: TipoCampo,               // campo obbligatorio
    campo?: TipoCampo,              // campo opzionale (Option<T>)
    campo: [TipoCampo],             // lista ordinata di zero o più elementi (Vec<T>)
    campo: [TipoCampo; N],          // lista di almeno N elementi
}
```

Le varianti di enum sono indicate come `Variante { campo: Tipo }`.

### 1.2 Terminologia

- **Nodo**: un'istanza di un tipo enum o struct dell'AST.
- **Variante**: uno dei casi distinti di un tipo enum.
- **Span**: un intervallo di posizioni nel sorgente, associato a ogni nodo.
- **Tipo Descar**: un'istanza dell'enum `Type`, rappresentante il tipo statico di un'espressione.
- **Tipo concreto**: un `Type` che non è `Void` né `NullPtr`.
- **Tipo numerico**: uno tra `I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `U64`, `F32`, `F64`.
- **Tipo intero**: uno tra `I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `U64`.
- **Tipo floating-point**: `F32` o `F64`.
- **Tipo primitivo**: qualsiasi tipo numerics, `Char`, `String`, `Bool`.
- **Tipo composto**: `Array { ... }` o `Vector { ... }`.
- **Programma**: lista ordinata di `Stmt` a livello globale.

### 1.3 Cardinalità

- `N ≥ 0`: zero o più elementi.
- `N ≥ 1`: almeno un elemento.
- `N = 1`: esattamente un elemento.
- `Option<T>`: zero o un elemento (`None` | `Some(T)`).

---

## 2. Architettura generale dell'AST

L'AST è composto da due categorie principali di nodi:

| Categoria | Tipo Rust | Descrizione |
| --- | --- | --- |
| Espressione | `Expr` | Costrutto che produce un valore tipato |
| Istruzione | `Stmt` | Costrutto che produce effetti o struttura il flusso |

Tutti i nodi `Expr` e `Stmt` portano un campo `span: SourceSpan` che identifica la loro estensione nel sorgente. Per `Stmt::Expression`, lo span è delegato all'`Expr` interna.

La radice di un programma Descar è una lista `Vec<Stmt>`. Non esiste un nodo radice esplicito di tipo `Program`; la lista stessa costituisce la radice.

**Relazioni fondamentali:**

```text
Vec<Stmt>  (radice del programma)
  └─ Stmt
       ├─ Stmt::Function { body: Box<Stmt>, ... }
       ├─ Stmt::MainFunction { body: Box<Stmt>, ... }
       ├─ Stmt::Block { statements: Vec<Stmt>, ... }
       ├─ Stmt::If { then_branch: Box<Stmt>, else_branch: ElseBranch, ... }
       ├─ Stmt::While { body: Box<Stmt>, ... }
       ├─ Stmt::For { body: Box<Stmt>, ... }
       ├─ Stmt::VarDeclaration { bindings: Vec<VarBinding>, ... }
       ├─ Stmt::Return { value: Option<Expr>, ... }
       ├─ Stmt::Break { ... }
       ├─ Stmt::Continue { ... }
       └─ Stmt::Expression { expr: Box<Expr> }
            └─ Expr
                 ├─ Expr::Binary { left: Box<Expr>, right: Box<Expr>, ... }
                 ├─ Expr::Unary { expr: Box<Expr>, ... }
                 ├─ Expr::Grouping { expr: Box<Expr>, ... }
                 ├─ Expr::Literal { value: LiteralValue, ... }
                 ├─ Expr::ArrayLiteral { elements: Vec<Expr>, ... }
                 ├─ Expr::Variable { name: String, ... }
                 ├─ Expr::Assign { target: Box<Expr>, value: Box<Expr>, ... }
                 ├─ Expr::Call { callee: Box<Expr>, arguments: Vec<Expr>, ... }
                 └─ Expr::ArrayAccess { array: Box<Expr>, index: Box<Expr>, ... }
```

---

## 3. Infrastruttura di localizzazione

Ogni nodo dell'AST porta informazioni di posizione sorgente tramite i seguenti tipi.

### 3.1 `SourceLocation`

Rappresenta una posizione puntuale nel sorgente.

L'implementazione Rust corrente è:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Default, Hash)]
pub struct SourceLocation {
    line: usize,
    column: usize,
    offset: usize,
    index: usize,
    utf8_offset: usize,
    code_point_offset: usize,
}
```

I campi sono privati e sono accessibili tramite i relativi getter. Il costruttore canonico è `SourceLocation::new(line, column, offset, index, utf8_offset, code_point_offset)`.

**Semantica dei campi:**

- `line`: numero di riga 1-based.
- `column`: numero di colonna 1-based.
- `offset`: offset in byte 0-based.
- `index`: indice di carattere 0-based.
- `utf8_offset`: offset UTF-8, oppure `UNKNOWN` se non calcolato.
- `code_point_offset`: offset in code point, oppure `UNKNOWN` se non calcolato.

**Costante speciale:** `UNKNOWN = usize::MAX` indica che un campo opzionale non è stato calcolato.

**Uguaglianza:** `SourceLocation` deriva `PartialEq` ed `Eq`; il confronto considera tutti i sei campi della struttura. Due posizioni con lo stesso `offset` ma valori diversi negli altri campi possono quindi essere diverse.

**Ordinamento:** `SourceLocation` deriva `Ord` e `PartialOrd`. L'ordinamento è quello derivato dalla struttura Rust e confronta i campi nell'ordine della loro dichiarazione:

```text
line
column
offset
index
utf8_offset
code_point_offset
```

Pertanto l'ordinamento non è basato esclusivamente su `offset`.

**Hash:** `SourceLocation` deriva `Hash`; il valore hash comprende i campi della struttura secondo la normale semantica del derive Rust.

**Default:** `SourceLocation::default()` produce una struttura con tutti i sei campi impostati a `0`. Questo valore è usato per rappresentare posizioni sintetiche o valori di default. Poiché non esiste un campo separato che distingua una posizione sintetica da una reale, `line = 0` e `column = 0` non devono essere interpretati come coordinate sorgente 1-based valide.

**Display:** la rappresentazione testuale implementata da `Display` è:

```text
line <line>:column <column>
```

L'implementazione non include `offset`, `index`, `utf8_offset` o `code_point_offset` nella stringa visualizzata.

### 3.2 `SourceSpan`

Rappresenta un intervallo di testo nel sorgente, con semantica **half-open** `[start, end)`.

```text
SourceSpan {
    file_path: Arc<str>,        // percorso del file sorgente, condiviso
    start:     SourceLocation,  // posizione iniziale (inclusa)
    end:       SourceLocation,  // posizione finale (esclusa)
}
```

**Costruttore:**

`SourceSpan::new(file_path, start, end)` non esegue alcuna validazione. È responsabilità del chiamante fornire un intervallo valido.

**Invarianti semantici richiesti dalla specifica:**

- `start.offset <= end.offset`.
- Se `start.offset == end.offset`, lo span ha lunghezza zero ed è quindi vuoto.
- La convenzione half-open determina che `start` sia incluso e `end` escluso.
- Gli span possono estendersi su più righe.
- Per gli span ordinari associati a un file sorgente, `file_path` identifica il file a cui appartengono le due posizioni. `SourceSpan::default()` usa invece una stringa vuota come percorso.

**Lunghezza:** `length()` restituisce la differenza tra gli offset byte di fine e inizio, usando la semantica UTF-8 in byte.

Un carattere multibyte, come `'€'`, contribuisce con 3 alla lunghezza, non con 1.

Se per errore il chiamante costruisce uno span con `end.offset < start.offset`, l'implementazione di `length()` usa una sottrazione saturante e restituisce `0`. Questo non rende però l'intervallo semanticamente ben formato secondo l'invariante precedente.

**`is_empty()`:**

`is_empty()` è implementato come equivalenza con `length() == 0`.

Per uno span ben formato, ciò equivale a:

```text
start.offset == end.offset
```

Non equivale invece necessariamente a `start == end`, perché `SourceLocation::Eq` confronta tutti i sei campi della posizione.

**`is_multiline()`:**

`is_multiline()` restituisce `true` quando:

```text
start.line != end.line
```

**`contains(location)`:**

`contains(location)` verifica esclusivamente l'offset byte secondo l'intervallo half-open:

```text
location.offset >= start.offset &&
location.offset < end.offset
```

Pertanto la verifica non confronta direttamente `line`, `column`, `index`, `utf8_offset` o `code_point_offset`.

**`overlaps(other)`:**

`overlaps(other)` verifica l'intersezione stretta tra gli intervalli half-open. Due span adiacenti non si sovrappongono.

**`merge(other)`:**

`merge(other)` produce il minimo intervallo che contiene entrambi gli span, scegliendo lo start con offset minore e l'end con offset maggiore.

L'implementazione conserva il `file_path` di `self`. La documentazione di `SourceSpan::merge` assume che i due span appartengano allo stesso file; la specifica non definisce il risultato di un merge tra file differenti come operazione semanticamente significativa.

**Default:** `SourceSpan::default()` ha `file_path = ""`, `start = SourceLocation::default()` ed `end = SourceLocation::default()`. È usato per nodi o dati sintetici.

**Display:** la rappresentazione testuale di `SourceSpan` dipende dalla sua lunghezza.

Per uno span non vuoto:

```text
<truncated_path>:<start_location>-<end_location>
```

Per uno span vuoto:

```text
<truncated_path>:<start_location>
```

dove `start_location` e `end_location` usano la rappresentazione di `SourceLocation::Display`.

Il percorso viene troncato per mostrare soltanto gli ultimi 2 componenti, con prefisso `..` quando il percorso originale contiene più componenti. Per il percorso di default vuoto, viene applicata la stessa funzione di troncamento del percorso.


---

## 4. Valori numerici (`Number`)

`Number` rappresenta un letterale numerico con il suo tipo concreto e valore, preservando la rappresentazione originale del sorgente.

```rust
enum Number {
    I8(i8),                      // [-128, 127],   suffisso sorgente: i8
    I16(i16),                    // [-32768, 32767], suffisso: i16
    I32(i32),                    // [-2^31, 2^31-1], suffisso: i32
    Integer(i64),                // [-2^63, 2^63-1], no suffisso (default intero con segno)
    U8(u8),                      // [0, 255],        suffisso: u8
    U16(u16),                    // [0, 65535],      suffisso: u16
    U32(u32),                    // [0, 2^32-1],     suffisso: u32
    UnsignedInteger(u64),        // [0, 2^64-1],     suffisso: u o U (default intero senza segno)
    Float32(f32),                // IEEE 754 single, suffisso: f
    Float64(f64),                // IEEE 754 double, no suffisso (default floating-point)
    Scientific32(f32, i32),      // base × 10^exp, IEEE 754 single, suffisso esponente + f
    Scientific64(f64, i32),      // base × 10^exp, IEEE 754 double, no suffisso
}
```

**Corrispondenza con `Type`:**

| `Number` | `Type` derivato |
| --- | --- |
| `I8(_)` | `Type::I8` |
| `I16(_)` | `Type::I16` |
| `I32(_)` | `Type::I32` |
| `Integer(_)` | `Type::I64` |
| `U8(_)` | `Type::U8` |
| `U16(_)` | `Type::U16` |
| `U32(_)` | `Type::U32` |
| `UnsignedInteger(_)` | `Type::U64` |
| `Float32(_)` | `Type::F32` |
| `Float64(_)` | `Type::F64` |
| `Scientific32(_, _)` | `Type::F32` |
| `Scientific64(_, _)` | `Type::F64` |

**Uguaglianza e hash per floating-point:** l'uguaglianza è definita per bit (`to_bits()`), non per valore matematico. Di conseguenza:

- `NaN == NaN` (stessa rappresentazione bit a bit) è `true`.
- `-0.0 == +0.0` è `false`.
- `Integer(42) != Float64(42.0)` anche se matematicamente equivalenti.
- Due varianti distinte non sono mai uguali.

**Validità:** tutti i valori di `Number` sono considerati validi a livello AST; la verifica dell'overflow o underflow avviene durante la fase lessicale, prima della costruzione dell'AST.

---

## 5. Valori letterali (`LiteralValue`)

```text
enum LiteralValue {
    Numeric(Number),        // qualsiasi letterale numerico
    StringLit(String),      // stringa tra virgolette doppie, escape già risolti
    CharLit(String),        // carattere tra virgolette singole, come String (può essere multibyte)
    Bool(bool),             // true | false
    NullPtr,                // nullptr
}
```

**Nota su `CharLit`:** il tipo Rust interno è `String`, non `char`, per supportare caratteri Unicode multi-codepoint come `'π'`. Il valore contenuto è la sequenza di byte del carattere dopo la rimozione delle virgolette, con le sequenze di escape già processate dal lessico.

**Nota su `StringLit`:** il valore contenuto è la stringa senza le virgolette delimitanti; le sequenze di escape (es. `\n`, `\\`, `\"`) sono già state trasformate in byte corrispondenti dal lessico.

**Tipo Descar associato:**

| `LiteralValue` | `Type` |
| --- | --- |
| `Numeric(n)` | determinato da `Number` (vedi §4) |
| `StringLit(_)` | `Type::String` |
| `CharLit(_)` | `Type::Char` |
| `Bool(_)` | `Type::Bool` |
| `NullPtr` | `Type::NullPtr` |

---

## 6. Operatori binari (`BinaryOp`)

`BinaryOp` è una enum `Copy` senza campi; ogni variante identifica un operatore distinto.

### 6.1 Tabella completa

| Variante | Simbolo sorgente | Categoria | Tipo risultato |
| --- | --- | --- | --- |
| `Add` | `+` | Aritmetico | tipo promosso degli operandi |
| `AddEqual` | `+=` | Aritm. composto | tipo del target |
| `Subtract` | `-` | Aritmetico | tipo promosso degli operandi |
| `SubtractEqual` | `-=` | Aritm. composto | tipo del target |
| `Multiply` | `*` | Aritmetico | tipo promosso degli operandi |
| `MultiplyEqual` | `*=` | Aritm. composto | tipo del target |
| `Divide` | `/` | Aritmetico | tipo promosso degli operandi |
| `DivideEqual` | `/=` | Aritm. composto | tipo del target |
| `Modulo` | `%` | Aritmetico | tipo promosso degli operandi |
| `ModuloEqual` | `%=` | Aritm. composto | tipo del target |
| `Equal` | `==` | Confronto | `Type::Bool` |
| `NotEqual` | `!=` | Confronto | `Type::Bool` |
| `Less` | `<` | Confronto | `Type::Bool` |
| `LessEqual` | `<=` | Confronto | `Type::Bool` |
| `Greater` | `>` | Confronto | `Type::Bool` |
| `GreaterEqual` | `>=` | Confronto | `Type::Bool` |
| `And` | `&&` | Logico | `Type::Bool` |
| `Or` | `\|\|` | Logico | `Type::Bool` |
| `BitwiseAnd` | `&` | Bitwise | tipo promosso degli operandi (intero) |
| `BitwiseAndEqual` | `&=` | Bitwise composto | tipo del target |
| `BitwiseOr` | `\|` | Bitwise | tipo promosso degli operandi (intero) |
| `BitwiseOrEqual` | `\|=` | Bitwise composto | tipo del target |
| `BitwiseXor` | `^` | Bitwise | tipo promosso degli operandi (intero) |
| `BitwiseXorEqual` | `^=` | Bitwise composto | tipo del target |
| `ShiftLeft` | `<<` | Shift | tipo promosso degli operandi (intero) |
| `ShiftLeftEqual` | `<<=` | Shift composto | tipo del target |
| `ShiftRight` | `>>` | Shift | tipo promosso degli operandi (intero) |
| `ShiftRightEqual` | `>>=` | Shift composto | tipo del target |

### 6.2 Operatori composti (compound assignment)

Un operatore composto è uno tra: `AddEqual`, `SubtractEqual`, `MultiplyEqual`, `DivideEqual`, `ModuloEqual`, `BitwiseAndEqual`, `BitwiseOrEqual`, `BitwiseXorEqual`, `ShiftLeftEqual`, `ShiftRightEqual`.

**Vincolo aggiuntivo per operatori composti:** il sotto-nodo `left` nell'`Expr::Binary` corrispondente deve identificare una variabile mutabile. La verifica avviene tramite estrazione del nome base della variabile (`base_variable_name`).

---

## 7. Operatori unari (`UnaryOp`, `UnaryOpSide`)

### 7.1 `UnaryOp`

```rust
enum UnaryOp {
    Negate,      // operatore - (negazione aritmetica)
    Not,         // operatore ! (negazione logica)
    BitwiseNot,  // operatore ~ (complemento bit a bit)
    Increment,   // operatore ++ (incremento)
    Decrement,   // operatore -- (decremento)
}
```

### 7.2 `UnaryOpSide`

```rust
enum UnaryOpSide {
    Prefix,   // operatore precede l'operando (es. -x, !flag, ++i)
    Postfix,  // operatore segue l'operando (es. i++, i--)
}
```

**Combinazioni valide:**

| `UnaryOp` | `Prefix` | `Postfix` |
| --- | --- | --- |
| `Negate` | sì | no |
| `Not` | sì | no |
| `BitwiseNot` | sì | no |
| `Increment` | sì | sì |
| `Decrement` | sì | sì |

**Nota:** l'AST non impedisce strutturalmente combinazioni invalide (es. `Negate` postfix), ma il type checker non riconosce tale forma e il parser non la genera. Una specifica di validità formale deve considerare invalide le combinazioni non elencate sopra.

**Tipo risultato degli operatori unari:**

| `UnaryOp` | Tipo richiesto dell'operando | Tipo risultato |
| --- | --- | --- |
| `Negate` | numerico | stesso tipo dell'operando |
| `Not` | `Bool` | `Bool` |
| `BitwiseNot` | intero | stesso tipo dell'operando |
| `Increment` | numerico | stesso tipo dell'operando |
| `Decrement` | numerico | stesso tipo dell'operando |

**Vincolo di mutabilità per `Increment`/`Decrement`:** l'operando deve essere una variabile mutabile o un accesso ad array di una variabile mutabile. La verifica avviene tramite `base_variable_name`.

---

## 8. Tipi (`Type`)

`Type` è l'enum centrale del sistema di tipizzazione dell'AST.

```rust
enum Type {
    // Tipi interi con segno
    I8,   // 8 bit,  [-128, 127]
    I16,  // 16 bit, [-32768, 32767]
    I32,  // 32 bit, [-2^31, 2^31-1]
    I64,  // 64 bit, [-2^63, 2^63-1]

    // Tipi interi senza segno
    U8,   // 8 bit,  [0, 255]
    U16,  // 16 bit, [0, 65535]
    U32,  // 32 bit, [0, 2^32-1]
    U64,  // 64 bit, [0, 2^64-1]

    // Tipi floating-point (IEEE 754)
    F32,  // single precision (32 bit)
    F64,  // double precision (64 bit)

    // Tipi scalari non numerici
    Char,    // carattere Unicode (rappresentato internamente come String)
    String,  // stringa dinamica
    Bool,    // valore booleano (true | false)

    // Tipo definito dall'utente
    Custom { name: Arc<str> },  // nome del tipo utente, non vuoto

    // Tipi composti
    Array {
        element_type: Box<Type>,  // tipo degli elementi, non Void, non NullPtr
        size: Box<Expr>,          // espressione di dimensione, valutabile a u64 > 0
    },

    Vector {
        element_type: Box<Type>,  // tipo degli elementi, non Void, non NullPtr
    },

    // Tipi speciali
    Void,     // assenza di valore (tipo di ritorno di funzioni senza valore)
    NullPtr,  // tipo del letterale nullptr
}
```

### 8.1 Vincoli di validità per `Type`

- **`Custom { name }`**: `name` deve essere non vuoto. Il type checker non valida se il tipo custom è effettivamente dichiarato (versione attuale; `TypeAlias` nella symbol table è previsto per uso futuro).
- **`Array { element_type, size }`**:
    - `element_type` non deve essere `Void` né `NullPtr`.
    - `size` deve essere un'espressione valutabile come intero positivo (`u64 > 0`) a compile time. Il type checker accetta solo `Expr::Literal { value: LiteralValue::Numeric(...) }` con valore non negativo come dimensione valida. Espressioni non valutabili staticamente sono ammesse strutturalmente ma causano `None` nella valutazione della dimensione.
- **`Vector { element_type }`**: `element_type` non deve essere `Void` né `NullPtr`.
- **`Void`**: usato esclusivamente come tipo di ritorno di funzioni (incluso `main`). Non può apparire come tipo di una variabile, parametro, o elemento di array/vector.
- **`NullPtr`**: tipo inferito dal letterale `nullptr`. Può essere assegnato a `Array { ... }`, `Vector { ... }`, o `Custom { ... }` (vedi §14.3).

### 8.2 Display

| `Type` | Rappresentazione testuale |
| --- | --- |
| `I8` | `"i8"` |
| `I16` | `"i16"` |
| `I32` | `"i32"` |
| `I64` | `"i64"` |
| `U8` | `"u8"` |
| `U16` | `"u16"` |
| `U32` | `"u32"` |
| `U64` | `"u64"` |
| `F32` | `"f32"` |
| `F64` | `"f64"` |
| `Char` | `"char"` |
| `String` | `"string"` |
| `Bool` | `"bool"` |
| `Custom { name }` | `name` (verbatim) |
| `Array { element_type, size }` | `"[{element_type}; {size_str}]"` dove `size_str` è il valore numerico se valutabile, altrimenti `"?"` |
| `Vector { element_type }` | `"vector<{element_type}>"` |
| `Void` | `"void"` |
| `NullPtr` | `"nullptr"` |

---

## 9. Parametri di funzione (`Parameter`)

```rust
struct Parameter {
    name:            String,     // identificatore del parametro, non vuoto
    type_annotation: Type,       // tipo annotato esplicitamente, non Void, non NullPtr
    span:            SourceSpan, // posizione nel sorgente
}
```

**Vincoli:**

- `name` deve essere un identificatore valido (pattern: `[a-zA-Z_][a-zA-Z0-9_]*` per ASCII, con estensione Unicode supportata).
- `type_annotation` non deve essere `Void` né `NullPtr`.
- I parametri di una stessa funzione devono avere nomi distinti; la verifica avviene nella symbol table.
- I parametri vengono inseriti nello scope di funzione come variabili mutabili (`mutable: true`).

---

## 10. Binding di variabile (`VarBinding`)

```rust
struct VarBinding {
    name:        String,       // identificatore della variabile, non vuoto
    initializer: Option<Expr>, // espressione di inizializzazione opzionale
}
```

**Semantica:** un `VarBinding` è un elemento di `Stmt::VarDeclaration`. Più binding nella stessa dichiarazione condividono lo stesso tipo annotato e lo stesso flag di mutabilità.

**Vincoli:**

- `name` deve essere un identificatore valido.
- Se `initializer` è presente (`Some(expr)`), il tipo inferito da `expr` deve essere assegnabile al `type_annotation` della dichiarazione (vedi §14.3).
- Se `initializer` è `None`, la variabile è dichiarata ma non inizializzata; il type checker non emette errori per la mancata inizializzazione (gestione runtime non specificata qui).
- I binding multipli nella stessa dichiarazione sono processati in ordine; ogni binding viene inserito nella symbol table indipendentemente.

---

## 11. Ramo alternativo (`ElseBranch`)

```rust
enum ElseBranch {
    None,               // assenza di ramo else
    Block(Box<Stmt>),   // blocco else { ... }, il Stmt interno deve essere Stmt::Block
    ElseIf(Box<Stmt>),  // catena else if (...) { ... }, il Stmt interno deve essere Stmt::If
}
```

**Vincoli:**

- `ElseBranch::Block(stmt)`: `stmt` deve essere una variante `Stmt::Block`.
- `ElseBranch::ElseIf(stmt)`: `stmt` deve essere una variante `Stmt::If`.
- Entrambe le varianti con box permettono catene arbitrariamente profonde di `else if`.

---

## 12. Espressioni (`Expr`)

Ogni variante di `Expr` porta un campo `span: SourceSpan`. Il metodo `span()` è implementato per tutte le varianti.

### 12.1 `Expr::Binary`

```
Binary {
    left:  Box<Expr>,   // operando sinistro
    op:    BinaryOp,    // operatore
    right: Box<Expr>,   // operando destro
    span:  SourceSpan,
}
```

**Semantica:** valuta `left`, valuta `right`, applica `op`. Per gli operatori composti (`AddEqual`, ecc.), `left` è anche il target dell'assegnamento.

**Vincoli di tipo:** vedi §14.4.

**Invariante di span:** `span` deve coprire integralmente `left.span()`, il simbolo dell'operatore, e `right.span()`.

### 12.2 `Expr::Unary`

```text
Unary {
    op:   UnaryOp,
    side: UnaryOpSide,
    expr: Box<Expr>,    // operando
    span: SourceSpan,
}
```

**Semantica:** applica `op` all'operando `expr`, sul lato indicato da `side`.

**Vincoli di tipo:** vedi §14.5.

### 12.3 `Expr::Grouping`

```text
Grouping {
    expr: Box<Expr>,
    span: SourceSpan,
}
```

**Semantica:** parentesi esplicite. Non ha effetto semantico autonomo: il tipo di `Grouping` è identico al tipo di `expr`. Serve esclusivamente a preservare la struttura dell'AST riflettendo le parentesi del sorgente.

**Vincolo:** `expr` non deve essere `None`; la parentesi vuota `()` non è un'espressione valida in Descar.

### 12.4 `Expr::Literal`

```text
Literal {
    value: LiteralValue,
    span:  SourceSpan,
}
```

**Semantica:** rappresenta un valore costante. Il tipo è determinato univocamente da `value` (vedi §5).

**Costruttori ausiliari:**

- `Expr::null_expr(span)` → `Literal { value: NullPtr, span }`
- `Expr::new_number_literal(n, span)` → `Literal { value: Numeric(n), span }`
- `Expr::new_bool_literal(b, span)` → `Literal { value: Bool(b), span }`
- `Expr::new_nullptr_literal(span)` → alias di `null_expr`
- `Expr::new_string_literal(s, span)` → `Literal { value: StringLit(s), span }`
- `Expr::new_char_literal(s, span)` → `Literal { value: CharLit(s), span }`

### 12.5 `Expr::ArrayLiteral`

```text
ArrayLiteral {
    elements: Vec<Expr>,  // N ≥ 0 espressioni, ma N = 0 produce errore semantico
    span:     SourceSpan,
}
```

**Semantica:** letterale di array. Il tipo risultante è `Type::Array { element_type: T, size: N }` dove `T` è il tipo comune degli elementi e `N` è il numero di elementi come `Expr::Literal { value: Numeric(Integer(len as i64)), span }`.

**Vincoli:**

- `elements.len() >= 1`: array vuoto (`{}`) è strutturalmente rappresentabile ma produce errore semantico `E2020`.
- Tutti gli elementi devono avere lo stesso tipo (no promozione implicita tra elementi); tipi misti producono `E2021`.
- Il tipo degli elementi non deve essere `Void` né `NullPtr`.

**Tipo del campo `size` generato:** `Expr::Literal { value: LiteralValue::Numeric(Number::Integer(len as i64)), span }` dove `span` è lo span dell'intero array literal.

### 12.6 `Expr::Variable`

```text
Variable {
    name: String,       // nome dell'identificatore, non vuoto
    span: SourceSpan,
}
```

**Semantica:** riferimento a una variabile dichiarata nello scope corrente o in un scope esterno. Il tipo è quello dichiarato nella symbol table per `name`.

**Vincoli:**

- `name` non deve essere vuoto.
- La variabile deve essere dichiarata in un scope accessibile; altrimenti errore `E2023`.
- Se `name` corrisponde a una funzione (non una variabile), errore `E2022`.

### 12.7 `Expr::Assign`

```text
Assign {
    target: Box<Expr>,  // espressione target, deve essere Variable o ArrayAccess
    value:  Box<Expr>,  // valore da assegnare
    span:   SourceSpan,
}
```

**Semantica:** assegna il valore prodotto da `value` alla locazione identificata da `target`.

**Target validi:**

- `Expr::Variable { name, ... }`: la variabile `name` deve essere mutabile (`mutable: true`).
- `Expr::ArrayAccess { array, index, ... }`: la variabile base dell'array deve essere mutabile.
- Qualsiasi altro tipo di `target` produce errore `E1003`.

**Tipo risultante:** il tipo di `target`.

**Vincolo di tipo:** il tipo di `value` deve essere assegnabile al tipo di `target` (vedi §14.3).

### 12.8 `Expr::Call`

```text
Call {
    callee:    Box<Expr>,   // deve essere Expr::Variable (nome della funzione)
    arguments: Vec<Expr>,   // N ≥ 0 espressioni argomento
    span:      SourceSpan,
}
```

**Semantica:** chiamata di funzione. Il tipo risultante è il tipo di ritorno della funzione chiamata.

**Vincoli:**

- `callee` deve essere `Expr::Variable { name, ... }`; altrimenti errore `E2026`.
- La funzione `name` deve essere dichiarata nella symbol table; altrimenti errore `E2027`.
- `arguments.len()` deve essere uguale al numero di parametri della funzione dichiarata; altrimenti errore `E2028`.
- Il tipo di ogni argomento deve essere assegnabile al tipo del parametro corrispondente (posizione 1-based); altrimenti errore `E2029`.
- Il type checker continua a visitare gli argomenti anche se `callee` non è valido o la funzione non esiste, per raccogliere più errori possibili.

**Tipo risultante:** `return_type` della `FunctionSymbol` corrispondente.

### 12.9 `Expr::ArrayAccess`

```text
ArrayAccess {
    array: Box<Expr>,  // espressione che produce un array o vector
    index: Box<Expr>,  // indice, deve avere tipo intero
    span:  SourceSpan,
}
```

**Semantica:** accesso a un elemento di array o vector. Il tipo risultante è il tipo dell'elemento.

**Vincoli:**

- Il tipo di `array` deve essere `Type::Array { element_type, ... }` o `Type::Vector { element_type }`.
- Il tipo di `index` deve essere un tipo intero (`I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `U64`); altrimenti errore `E2030`.
- Se il tipo di `array` non è né `Array` né `Vector`, errore `E2031`.

**Tipo risultante:** `element_type` dell'array o vector.

---

## 13. Istruzioni (`Stmt`)

Ogni variante di `Stmt` porta un campo `span: SourceSpan` (eccezione: `Stmt::Expression` delega lo span all'`Expr` interna tramite il metodo `span()`).

### 13.1 `Stmt::Expression`

```text
Expression {
    expr: Box<Expr>,
}
```

**Semantica:** valuta `expr` ignorando il valore prodotto. Usato per effetti collaterali (chiamate di funzione, assegnamenti, incrementi).

**Nota span:** il metodo `span()` di `Stmt::Expression` restituisce `expr.span()`, non un campo proprio.

### 13.2 `Stmt::VarDeclaration`

```text
VarDeclaration {
    bindings:        Vec<VarBinding>,  // N ≥ 1 binding
    type_annotation: Type,             // tipo comune a tutti i binding
    is_mutable:      bool,             // true: var, false: const
    span:            SourceSpan,
}
```

**Semantica:** dichiara una o più variabili dello stesso tipo nel scope corrente.

**Vincoli:**

- `bindings` deve contenere almeno un elemento.
- `type_annotation` non deve essere `Void` né `NullPtr`.
- Se il binding ha un inizializzatore, il suo tipo deve essere assegnabile a `type_annotation`.
- `is_mutable = false` significa che la variabile non può essere riassegnata (errore `E2024` su tentativi di assegnamento).
- Tutti i nomi nei binding devono essere univoci nello scope corrente (altrimenti errore `E2032`).

**Keyword sorgente:**

- `is_mutable = true` → keyword `var`
- `is_mutable = false` → keyword `const`

### 13.3 `Stmt::Function`

```text
Function {
    name:        String,          // nome della funzione, non vuoto
    parameters:  Vec<Parameter>,  // N ≥ 0 parametri
    return_type: Type,            // tipo di ritorno dichiarato
    body:        Box<Stmt>,       // deve essere Stmt::Block
    span:        SourceSpan,
}
```

**Semantica:** dichiara una funzione nel scope corrente (globale o annidato).

**Vincoli:**

- `name` deve essere un identificatore valido e non già dichiarato nello scope corrente (altrimenti `E2032`).
- I nomi dei parametri devono essere univoci tra loro (la verifica avviene durante la loro inserzione in scope).
- `body` deve essere `Stmt::Block`; il parser produce sempre un blocco come corpo.
- Se `return_type != Void`, il corpo deve contenere almeno un `Stmt::Return` raggiungibile in tutti i percorsi di controllo (vedi `function_has_return`); altrimenti errore `E2003`.
- I parametri vengono inseriti come variabili mutabili nello scope della funzione, **non** nello scope in cui la funzione è dichiarata.
- La funzione stessa viene dichiarata nello scope **esterno** alla funzione (scope corrente al momento della dichiarazione), prima di processare il corpo.

### 13.4 `Stmt::If`

```text
If {
    condition:   Box<Expr>,
    then_branch: Box<Stmt>,    // deve essere Stmt::Block
    else_branch: ElseBranch,
    span:        SourceSpan,
}
```

**Semantica:** esecuzione condizionale. `then_branch` è eseguito se e solo se `condition` è `true`.

**Vincoli:**

- Il tipo di `condition` deve essere `Type::Bool`; altrimenti errore `E2004`.
- `then_branch` deve essere `Stmt::Block`.
- `else_branch` segue le regole di `ElseBranch` (vedi §11).

### 13.5 `Stmt::While`

```text
While {
    condition: Box<Expr>,
    body:      Box<Stmt>,
    span:      SourceSpan,
}
```

**Semantica:** loop con condizione di guardia valutata prima di ogni iterazione.

**Vincoli:**

- Il tipo di `condition` deve essere `Type::Bool`; altrimenti errore `E2004`.
- All'interno di `body`, `break` e `continue` sono validi.
- Il type checker crea un `Block` scope per il corpo del while.

### 13.6 `Stmt::For`

```text
For {
    initializer: Option<Box<Stmt>>,  // Stmt::VarDeclaration, opzionale
    condition:   Option<Expr>,       // espressione booleana, opzionale
    increment:   Option<Expr>,       // espressione di incremento, opzionale
    body:        Box<Stmt>,
    span:        SourceSpan,
}
```

**Semantica:** loop a tre componenti. L'`initializer`, se presente, è eseguito una volta. La `condition`, se presente, è valutata prima di ogni iterazione (assenza = iterazione infinita). L'`increment`, se presente, è eseguito alla fine di ogni iterazione.

**Vincoli:**

- Se `condition` è `Some(expr)`, il tipo di `expr` deve essere `Type::Bool`; altrimenti errore `E2004`.
- Il type checker crea un `Block` scope che contiene l'intero costrutto for (incluso `initializer`).
- All'interno di `body`, `break` e `continue` sono validi.
- `initializer`, se presente, è strutturalmente un `Box<Stmt>`; nella pratica il parser produce solo `Stmt::VarDeclaration`.

### 13.7 `Stmt::Block`

```text
Block {
    statements: Vec<Stmt>,  // N ≥ 0 istruzioni
    span:       SourceSpan,
}
```

**Semantica:** sequenza di istruzioni in uno scope lessicale separato. Un blocco vuoto è valido.

**Vincoli:**

- Il type checker crea un `Block` scope per ogni blocco.
- Lo scope viene distrutto (pop) al termine del blocco.

### 13.8 `Stmt::Return`

```text
Return {
    value: Option<Expr>,
    span:  SourceSpan,
}
```

**Semantica:** termina l'esecuzione della funzione corrente, opzionalmente restituendo un valore.

**Vincoli:**

- Deve trovarsi lessicalmente all'interno di un `Stmt::Function` o `Stmt::MainFunction`; altrimenti errore `E2005`.
- Se la funzione corrente ha `return_type = Void`: `value` deve essere `None`; un valore produce errore `E2006`.
- Se la funzione corrente ha `return_type != Void`: `value` deve essere `Some(expr)`, dove il tipo di `expr` è assegnabile al `return_type`; assenza del valore produce errore `E2008`; tipo incompatibile produce errore `E2007`.
- `main` ha `return_type = Void` implicito; un `return` in `main` non deve avere valore.

### 13.9 `Stmt::Break`

```text
Break {
    span: SourceSpan,
}
```

**Semantica:** interrompe il loop più interno (while o for).

**Vincoli:** deve trovarsi lessicalmente all'interno di un corpo di `Stmt::While` o `Stmt::For`; altrimenti errore `E2009`.

### 13.10 `Stmt::Continue`

```text
Continue {
    span: SourceSpan,
}
```

**Semantica:** salta alla prossima iterazione del loop più interno.

**Vincoli:** deve trovarsi lessicalmente all'interno di un corpo di `Stmt::While` o `Stmt::For`; altrimenti errore `E2010`.

### 13.11 `Stmt::MainFunction`

```text
MainFunction {
    body: Box<Stmt>,  // deve essere Stmt::Block
    span: SourceSpan,
}
```

**Semantica:** punto di ingresso del programma. Equivalente semanticamente a una `Function` con `name = "main"`, `parameters = []`, `return_type = Void`.

**Vincoli:**

- Deve comparire al massimo una volta per programma (non enforced strutturalmente nell'AST, ma semanticamente richiesto; la violazione produce `E2032` nella symbol table se presente due volte).
- `body` deve essere `Stmt::Block`.
- Non accetta parametri.
- Il tipo di ritorno è implicitamente `Void`; un `return` nel corpo non deve avere valore.
- Trattato dal type checker esattamente come una funzione `void` senza parametri.

---

## 14. Sistema di tipizzazione

### 14.1 Tipi primitivi

| Tipo | Categoria | Descrizione |
| --- | --- | --- |
| `I8` | intero con segno | 8 bit, complemento a 2 |
| `I16` | intero con segno | 16 bit |
| `I32` | intero con segno | 32 bit |
| `I64` | intero con segno | 64 bit |
| `U8` | intero senza segno | 8 bit |
| `U16` | intero senza segno | 16 bit |
| `U32` | intero senza segno | 32 bit |
| `U64` | intero senza segno | 64 bit |
| `F32` | floating-point | IEEE 754 singola precisione |
| `F64` | floating-point | IEEE 754 doppia precisione |
| `Char` | carattere | sequenza Unicode (≥1 code point) |
| `String` | stringa | sequenza di caratteri |
| `Bool` | booleano | `true` o `false` |

### 14.2 Gerarchia di promozione numerica

La gerarchia di promozione è totalmente ordinata, dal rango più alto al più basso:

```text
F64 > F32 > U64 > I64 > U32 > I32 > U16 > I16 > U8 > I8
```

**Regola di promozione:** dati due tipi `T1` e `T2` entrambi numerici, il tipo promosso è quello con rango più alto. Se hanno lo stesso rango (stesso tipo), il risultato è quel tipo.

**Implementazione:** la promozione è calcolata tramite `promote_numeric_types(t1, t2)` con cache globale per coppie non nella gerarchia standard.

**Non-promozione:** tipi non numerici (`Bool`, `Char`, `String`, `Array`, `Vector`, `Custom`, `Void`, `NullPtr`) non partecipano alla promozione. Se uno o entrambi gli operandi non sono numerici, non avviene promozione e la compatibilità deve essere verificata esplicitamente.

### 14.2.1 Distinzione normativa tra ranking, promozione e assegnabilità

La **gerarchia di promozione** del §14.2 e le regole di `is_assignable(source, target)` del §14.3 definiscono concetti distinti.

1. Il **ranking** (`F64 > F32 > U64 > I64 > U32 > I32 > U16 > I16 > U8 > I8`) è un ordinamento usato per determinare il tipo comune nelle operazioni numeriche che ammettono promozione.
2. La **promozione numerica degli operatori** è una conversione implicita reale degli operandi verso il tipo promosso. Non è soltanto un confronto di ranking.
3. Una conversione necessaria alla promozione degli operatori **non implica** che `is_assignable(source, target)` debba restituire `true`.
4. Il termine **widening** usato nel titolo e nella tabella del §14.3 indica esclusivamente le conversioni ammesse in un contesto di assegnazione. Non deve essere interpretato come sinonimo della promozione usata dagli operatori.
5. Il fatto che una conversione sia usata durante la promozione di un operatore non la rende automaticamente disponibile per inizializzazioni, assegnamenti, argomenti di funzione o `return`.

### 14.2.2 Conversioni consentite esclusivamente dalla promozione degli operatori

Per rendere deterministica la promozione, sono definite esplicitamente le conversioni implicite degli operandi verso il tipo promosso.

| Da (`source`) | A (`target`) nella promozione operatoriale | Natura della conversione |
| --- | --- | --- |
| Qualsiasi tipo intero con segno (`I8`, `I16`, `I32`, `I64`) | `F32` | conversione implicita operatoriale, può essere lossy |
| Qualsiasi tipo intero senza segno (`U8`, `U16`, `U32`, `U64`) | `F32` | conversione implicita operatoriale, può essere lossy |
| Qualsiasi tipo intero numerico | `F64` | conversione implicita operatoriale, può essere lossy |
| `F32` | `F64` | conversione implicita operatoriale |
| Conversioni tra interi della stessa catena di signedness secondo la gerarchia | tipo intero di rango superiore | conversione implicita operatoriale |
| `I8`/`I16`/`I32`/`I64` ↔ `U8`/`U16`/`U32`/`U64` | nessuna promozione diretta tra catene signed e unsigned | richiede compatibilità prevista dalla specifica; in assenza, errore |

**Regola fondamentale:** le conversioni della presente tabella sono una relazione propria della promozione operatoriale. Non estendono `is_assignable`.

In particolare:

- `I64 -> F32` è consentita durante la promozione di un operatore numerico.
- `U64 -> F32` è consentita durante la promozione di un operatore numerico.
- `is_assignable(I64, F32) == false` resta invariato.
- `is_assignable(U64, F32) == false` resta invariato.
- `I64 + F32` viene promosso a `F32`.
- `U64 + F32` viene promosso a `F32`.

La presenza di `I64 -> F32` e `U64 -> F32` in questo paragrafo **non** va letta come aggiunta alla tabella di assegnabilità del §14.3.

**Algoritmo normativo di `promote_numeric_types(t1, t2)`:**

```text
se t1 o t2 non è numerico:
    nessuna promozione

se entrambi sono floating-point:
    restituisci il tipo floating-point di rango più alto

se esattamente uno è floating-point:
    restituisci il tipo floating-point dell'operando già floating-point
    l'operando intero viene convertito verso quel tipo

se entrambi sono interi:
    se hanno la stessa signedness:
        restituisci il tipo intero di rango più alto nella stessa catena
    altrimenti:
        nessuna promozione
```

Con questo algoritmo, il ranking totale del §14.2 viene applicato solo all'interno di una combinazione per la quale la promozione è semanticamente ammessa. Il ranking, da solo, non autorizza una conversione signed → unsigned o unsigned → signed.

### 14.2.3 Semantica della conversione intero → floating-point nella promozione

Una conversione implicita da un tipo intero a `F32` o `F64` durante la promozione è una conversione numerica effettiva.

1. Il valore intero viene convertito nel valore floating-point rappresentabile corrispondente.
2. La conversione può essere **lossy**, cioè può perdere precisione, anche quando non si verifica alcun overflow.
3. La conversione a `F32` usa la semantica IEEE 754 della conversione a binary32 con arrotondamento `roundTiesToEven`.
4. La conversione a `F64` usa la semantica IEEE 754 della conversione a binary64 con arrotondamento `roundTiesToEven`.
5. Per `I64` e `U64`, quindi, non è garantita la preservazione esatta di tutte le unità del valore quando il valore contiene più bit significativi di quanti siano rappresentabili esattamente nel formato floating-point di destinazione.
6. Il backend deve materializzare questa conversione prima dell'operazione, secondo il tipo promosso. Non è corretto eseguire l'operazione mantenendo l'operando nel tipo originario e limitarsi a trattare il ranking come un'informazione descrittiva.

### 14.3 Regole di assegnabilità (`is_assignable(source, target)`)

`source` è assegnabile a `target` nei seguenti casi:

**Promozioni numeriche esplicite (widening):**

| Da (`source`) | A (`target`) |
| --- | --- |
| `I8` | `I16`, `I32`, `I64`, `F32`, `F64` |
| `I16` | `I32`, `I64`, `F32`, `F64` |
| `I32` | `I64`, `F32`, `F64` |
| `I64` | `F64` |
| `U8` | `U16`, `U32`, `U64`, `F32`, `F64` |
| `U16` | `U32`, `U64`, `F32`, `F64` |
| `U32` | `U64`, `F32`, `F64` |
| `U64` | `F64` |
| `F32` | `F64` |

**Promozioni non numeriche:**

- `Char` → `String` (assegnabile).
- `NullPtr` → `Array { ... }` (qualsiasi array).
- `NullPtr` → `Vector { ... }` (qualsiasi vector).
- `NullPtr` → `Custom { ... }` (qualsiasi tipo custom).

**Tipi compositi:**

- `Array { element_type: E1, size: S1 }` → `Array { element_type: E2, size: S2 }` se e solo se:
  1. `is_assignable(E1, E2)` è `true`, e
  2. le dimensioni `S1` e `S2` sono entrambe valutabili e il loro valore è uguale (`get_size(S1) == get_size(S2)`).
- `Vector { element_type: E1 }` → `Vector { element_type: E2 }` se e solo se `is_assignable(E1, E2)`.

**Caso identità:** `source == target` (stesso tipo, confronto strutturale).

**Caso fallback:** tutti gli altri casi sono `false` (non assegnabile).

**Nota sull'assenza di promozione signed↔unsigned:** `I8` non è assegnabile a `U8`, `U32` non è assegnabile a `I64`, ecc. La gerarchia è separata (signed e unsigned sono catene distinte che convergono solo verso `F32`/`F64`).

### 14.3.1 Separazione tra `is_assignable` e promozione operatoriale

`is_assignable(source, target)` risponde esclusivamente alla domanda se un valore di tipo `source` possa essere usato dove è richiesto `target` nei contesti di assegnazione definiti dalla specifica.

La funzione di promozione degli operatori risponde invece alla domanda quale tipo comune e quale sequenza di conversioni implicite debbano essere applicati agli operandi di un operatore numerico.

Pertanto le seguenti espressioni sono semanticamente differenti:

```text
is_assignable(I64, F32)       == false
promote_numeric_types(I64,F32) == F32

is_assignable(U64, F32)       == false
promote_numeric_types(U64,F32) == F32
```

La seconda riga di ciascuna coppia non modifica il risultato della prima.

**Conseguenza per il backend:** il type checker può produrre un risultato `F32` per un'operazione `I64 + F32` o `U64 + F32` anche se non esiste un'assegnabilità generale verso `F32`. Il backend deve quindi supportare la conversione implicita richiesta dall'operatore e non deve implementare la promozione richiamando semplicemente `is_assignable`.

### 14.4 Regole di tipo per operatori binari

**Operatori aritmetici** (`Add`, `Subtract`, `Multiply`, `Divide`, `Modulo` e le loro varianti `*Equal`):

- Entrambi gli operandi devono essere numerici dopo promozione.
- Il tipo risultante è il tipo promosso dei due operandi.
- Se gli operandi non sono compatibili (dopo promozione), errore `E2013`.
- Se l'operando sinistro non è numerico, errore aggiuntivo `E2016`.

**Operatori di confronto** (`Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual`):

- Entrambi gli operandi devono avere tipi compatibili (promozione numerica applicata se entrambi numerici).
- Il tipo risultante è sempre `Type::Bool`.
- Se non compatibili, errore `E2014`.

**Operatori logici** (`And`, `Or`):

- Entrambi gli operandi devono avere tipo `Bool` (nessuna promozione).
- Il tipo risultante è `Type::Bool`.
- Se non compatibili, errore `E2012`.
- Se l'operando sinistro non è `Bool`, errore aggiuntivo `E2017`.

**Operatori bitwise e shift** (`BitwiseAnd`, `BitwiseOr`, `BitwiseXor`, `ShiftLeft`, `ShiftRight` e le loro varianti `*Equal`):

- Entrambi gli operandi devono essere tipi interi.
- Promozione numerica applicata.
- Il tipo risultante è il tipo promosso degli operandi interi.
- Se non interi, errore `E2011`.

**Operatori composti** (tutti i `*Equal`):

- Richiedono che `left` sia una variabile mutabile (o array access su variabile mutabile).
- L'immutabilità dell'obiettivo produce errore `E2024` **prima** di qualsiasi verifica di tipo.

### 14.4.1 Semantica operativa della promozione negli operatori

Per ogni operatore che dichiara nel presente paragrafo **promozione numerica applicata**, il type checker e il backend devono interpretare la promozione come una trasformazione degli operandi:

```text
T1 e1, T2 e2
      |
      | promote_numeric_types
      v
P, convert(e1 -> P), convert(e2 -> P)
      |
      | applicazione dell'operatore
      v
risultato di tipo P
```

dove `P` è il tipo promosso.

La conversione è concettualmente presente anche se l'AST non contiene un nodo `Cast` o un nodo `Conversion`. Non è necessario introdurre un nuovo tipo di nodo AST per rappresentarla.

**Esempi normativi:**

```text
I64 + F32  ->  F32
  left:  I64 -> F32
  right: F32 -> F32
  operation: F32 + F32

U64 + F32  ->  F32
  left:  U64 -> F32
  right: F32 -> F32
  operation: F32 + F32

F32 + F64  ->  F64
  left:  F32 -> F64
  right: F64 -> F64
  operation: F64 + F64
```

Il risultato dell'operatore è il tipo promosso, come già stabilito nel §14.4.

**Signed e unsigned:** la gerarchia di ranking resta definita come nel §14.2, ma il ranking da solo non autorizza una conversione tra le due catene di interi. In assenza di un tipo floating-point che fornisca il tipo comune previsto dalle regole applicabili, una coppia signed/unsigned incompatibile produce `E2013`.

### 14.5 Regole di tipo per operatori unari

| `UnaryOp` | Tipo richiesto | Tipo risultato | Errore |
| --- | --- | --- | --- |
| `Negate` | numerico | stesso tipo | `E2018` |
| `Not` | `Bool` | `Bool` | `E2019` |
| `BitwiseNot` | intero | stesso tipo | `E2018` |
| `Increment` | numerico, var mutabile | stesso tipo | `E2024` (immutabile), `E2018` (non numerico) |
| `Decrement` | numerico, var mutabile | stesso tipo | `E2024` (immutabile), `E2018` (non numerico) |

**Priorità degli errori per `Increment`/`Decrement`:** la verifica di mutabilità avviene prima della verifica del tipo. Se la variabile è immutabile, viene emesso `E2024` e la funzione restituisce `None` (nessun `E2018`).

### 14.6 Verifica ritorno nelle funzioni (`function_has_return`)

Una funzione "ha un ritorno" se il corpo soddisfa ricorsivamente le seguenti condizioni:

- `Stmt::Return { .. }`: sì.
- `Stmt::Block { statements }`: sì se almeno una delle istruzioni ha un ritorno.
- `Stmt::While { body }` o `Stmt::For { body }`: sì se il corpo ha un ritorno.
- `Stmt::If { then_branch, else_branch }`: sì se e solo se `then_branch` ha un ritorno **e** `else_branch` ha un ritorno (entrambi i rami devono restituire).
    - `ElseBranch::None`: il ramo else non ha un ritorno → la condizione è `false`.
    - `ElseBranch::Block(stmt)` o `ElseBranch::ElseIf(stmt)`: ricorsivo su `stmt`.
- Tutti gli altri `Stmt`: no.

**Condizione di errore:** se `return_type != Void` e `!function_has_return(body)`, errore `E2003`.

### 14.6.1 Interpretazione normativa di `function_has_return`

Ai fini della versione 1.0, il termine "return completo" utilizzato nel §13.3 **non indica una vera analisi di raggiungibilità o di controllo di flusso**.

La proprietà richiesta alle funzioni non-`Void` è definita esclusivamente dal predicato `function_has_return` specificato nel §14.6.

Pertanto `function_has_return` deve essere interpretato come una **verifica strutturale della presenza di un percorso di ritorno secondo le regole ricorsive del §14.6**, e non come una dimostrazione che ogni percorso di esecuzione termina con un `return`.

In particolare:

- `Stmt::Return` soddisfa la proprietà.
- `Stmt::Block` soddisfa la proprietà se almeno una istruzione contenuta la soddisfa.
- `Stmt::While` soddisfa la proprietà se il proprio `body` la soddisfa.
- `Stmt::For` soddisfa la proprietà se il proprio `body` la soddisfa.
- `Stmt::If` soddisfa la proprietà se e solo se sia il `then_branch` sia l'`else_branch` la soddisfano.
- Un `else` assente non soddisfa la proprietà.

Di conseguenza, la presenza di un `return` all'interno di un `while` o di un `for` è sufficiente per `function_has_return`, indipendentemente dalla possibilità che la condizione del loop impedisca l'ingresso nel corpo oppure che il loop non venga eseguito.

Esempio:

```text
fun f(): i32 {
    while (false) {
        return 1;
    }
}
```

In base alla definizione di §14.6, `function_has_return(body)` restituisce `true`. La funzione pertanto **non produce `E2003` per assenza di percorso di ritorno**.

Questa regola è intenzionalmente distinta da una futura analisi di control flow. Un'eventuale analisi che dimostri la raggiungibilità dei `return`, l'irraggiungibilità di codice o la terminazione dei loop appartiene a una fase semantica diversa e non è richiesta dalla versione 1.0.

In caso di contrasto interpretativo tra la formulazione descrittiva del §13.3 e la definizione algoritmica del §14.6, **la definizione algoritmica del §14.6 è normativa e prevale** per la versione 1.0.

---

## 15. Regole di validità semantica

Un'istanza dell'AST è **semanticamente valida** se e solo se soddisfa **tutte** le seguenti condizioni:

### 15.1 Regole di scoping

1. Ogni `Expr::Variable { name }` deve riferirsi a un nome dichiarato in un scope accessibile (corrente o antenato).
2. Ogni `Expr::Call { callee: Variable { name } }` deve riferirsi a un nome dichiarato come funzione in un scope accessibile.
3. Nessun identificatore deve essere dichiarato due volte nello stesso scope (né due variabili, né due funzioni con lo stesso nome, né una variabile e una funzione con lo stesso nome).
4. I parametri di una funzione sono visibili solo all'interno del corpo della funzione.
5. Le variabili dichiarate in un blocco non sono visibili al di fuori del blocco.

### 15.2 Regole di mutabilità

6. Assegnare a una variabile dichiarata con `is_mutable = false` è un errore.
7. Applicare `Increment` o `Decrement` a una variabile immutabile è un errore.
8. Applicare un operatore composto a una variabile immutabile è un errore.

### 15.3 Regole di tipo

9. Il tipo di ogni inizializzatore di variabile deve essere assegnabile al tipo annotato.
10. La condizione di `if`, `while`, e `for` deve avere tipo `Bool`.
11. Ogni operatore binario richiede i tipi degli operandi come specificato in §14.4.
12. Ogni operatore unario richiede il tipo dell'operando come specificato in §14.5.
13. Ogni argomento di una chiamata di funzione deve avere tipo assegnabile al tipo del parametro corrispondente.
14. Un `return` con valore deve avere tipo assegnabile al tipo di ritorno della funzione.
15. Un `return` senza valore è valido solo in funzioni void.
16. Un `return` con valore è invalido in funzioni void.
17. Un `return` fuori da qualsiasi funzione è un errore.
18. Tutti gli elementi di un `ArrayLiteral` devono avere lo stesso tipo.
19. Un `ArrayLiteral` vuoto è un errore.
20. L'indice in un `ArrayAccess` deve avere tipo intero.
21. L'operando di un `ArrayAccess` deve avere tipo `Array` o `Vector`.

### 15.4 Regole di controllo del flusso

22. `break` è valido solo all'interno di un loop (`while` o `for`).
23. `continue` è valido solo all'interno di un loop.
24. Una funzione non-void deve avere un percorso di ritorno in tutti i rami (vedi §14.6).

### 15.5 Regole strutturali

25. Il target di un'assegnazione deve essere `Expr::Variable` o `Expr::ArrayAccess`.
26. Il callee di una chiamata di funzione deve essere `Expr::Variable`.
27. Il numero di argomenti di una chiamata deve corrispondere al numero di parametri dichiarati.

---

## 16. Gerarchia di precedenza degli operatori

La precedenza è definita dal modulo `crates/descar-core/src/syntax/precendence.rs` tramite binding power Pratt. La struttura `Precedence` contiene due valori:

```rust
pub struct Precedence {
    pub left: u8,
    pub right: u8,
}
```

`binding_power(token)` restituisce la coppia `(left, right)` associata al token. Per i token non classificati come operatori infix/postfix restituisce `(0, 0)`.

### 16.1 Operatori infix e postfix

La tabella normativa deve corrispondere alla funzione `Precedence::binding_power`:

| Binding power (left, right) | Token | Categoria |
| --- | --- | --- |
| (2, 1) | `=`, `+=`, `-=`, `&=`, `|=`, `%=`, `^=`, `*=`, `/=`, `<<=`, `>>=` | assegnazione / assegnazione composta |
| (4, 3) | `||` | logico |
| (6, 5) | `&&` | logico |
| (8, 7) | `==`, `!=` | uguaglianza |
| (10, 9) | `<`, `<=`, `>`, `>=` | confronto |
| (12, 11) | `|` | bitwise OR |
| (14, 13) | `^` | bitwise XOR |
| (16, 15) | `&` | bitwise AND |
| (18, 17) | `<<`, `>>` | shift |
| (20, 19) | `+`, `-` | aritmetico |
| (22, 21) | `*`, `/`, `%` | aritmetico |
| (27, 26) | `(`, `[`, `.`, `++`, `--` | postfix / accesso |

La voce con binding power `(27, 26)` comprende token che non sono tutti operatori binari. La coppia documenta comunque la precedenza restituita da `Precedence::binding_power` per chiamata, accesso, membro e incremento/decremento postfix.

**Nota:** il token `.` è incluso nella tabella di precedenza perché `binding_power` gli assegna `(27,26)`. La sua eventuale rappresentazione nell'AST e il relativo supporto sintattico sono definiti dalle sezioni dedicate alle espressioni e non dalla sola tabella di precedenza.

### 16.2 Operatori unari prefix

La funzione `Precedence::unary_binding_power` assegna:

| Binding power (left, right) | Token |
| --- | --- |
| (24, 23) | `!`, `-`, `~`, `++`, `--` |

Questi token sono interpretati come operatori prefix quando vengono gestiti da `nud`. Per `++` e `--`, la forma postfix usa invece la coppia `(27,26)` di §16.1.

### 16.3 Regola effettiva del Pratt parser

Il parser corrente implementa il ciclo Pratt con la seguente condizione:

```text
lbp = Precedence::binding_power(token).left

se lbp <= min_bp:
    arresta il parsing dell'espressione corrente

altrimenti:
    consuma il token tramite led
```

Equivalentemente, un operatore viene consumato quando:

```text
left_bp > min_bp
```

Per gli operatori infix, il ramo destro viene analizzato usando il relativo `right_bp`:

```text
right = parse_expr(right_bp)
```

Con l'implementazione corrente, tutte le coppie infix definite da `Precedence::binding_power` hanno `left_bp > right_bp`. Di conseguenza, il successivo operatore con la stessa precedenza soddisfa nuovamente la condizione `left_bp > min_bp` nel ramo destro.

Perciò, con il parser corrente, gli operatori infix della stessa classe di precedenza sono associati a destra:

```text
a - b - c
=> a - (b - c)

a + b + c
=> a + (b + c)

a = b = c
=> a = (b = c)
```

La coppia `(2,1)` rende quindi l'assegnazione destra-associativa, ma la stessa relazione `left_bp > right_bp` è presente anche per le altre classi infix. La documentazione non deve descrivere queste classi come sinistra-associative finché la regola del parser rimane quella sopra riportata.

La precedenza relativa resta determinata dal valore di `left_bp`: un operatore con binding power maggiore viene consumato prima di uno con binding power minore.

### 16.4 Mapping degli operatori di assegnazione

Il token `=` appartiene alla classe di binding power `(2,1)), ma non viene rappresentato come `BinaryOp`. Il parser lo converte in `Expr::Assign`.

Gli operatori composti `+=`, `-=`, `&=`, `|=`, `%=`, `^=`, `*=`, `/=`, `<<=`, `>>=` vengono invece rappresentati come `Expr::Binary` con la corrispondente variante `BinaryOp`.

Questa distinzione è parte della specifica AST e deve rimanere coerente con la tabella di precedenza.

## 17. Tabella dei simboli e scoping

### 17.1 Struttura della symbol table

La symbol table è uno stack di `Scope`. Lo stack ha sempre almeno un elemento (lo scope globale).

```text
SymbolTable {
    scopes:           Vec<Scope>,               // stack di scope
    current_function: Option<FunctionSymbol>,   // funzione corrente per return-type checking
}
```

```text
Scope {
    kind:       ScopeKind,                       // tipo di scope
    symbols:    HashMap<Arc<str>, Symbol>,        // mappa nome → simbolo
    defined_at: Option<SourceSpan>,              // posizione dove lo scope è aperto
}
```

### 17.2 `ScopeKind`

```rust
enum ScopeKind {
    Global,    // scope globale (sempre presente, sempre il primo)
    Function,  // scope di funzione (aperto da visit_function)
    Block,     // scope di blocco (aperto da visit_block, visit_while, visit_for)
}
```

### 17.3 `Symbol`

```rust
enum Symbol {
    Variable(VariableSymbol),
    Function(FunctionSymbol),
    TypeAlias(Type),           // uso futuro
}
```

```text
VariableSymbol {
    name:            Arc<str>,
    ty:              Type,
    mutable:         bool,
    defined_at:      SourceSpan,
    last_assignment: Option<SourceSpan>,
}
```

```text
FunctionSymbol {
    name:        Arc<str>,
    parameters:  Vec<Parameter>,
    return_type: Type,
    defined_at:  SourceSpan,
}
```

### 17.4 Regole di risoluzione dei nomi

- La ricerca di un simbolo avviene dallo scope più interno (top of stack) verso quello più esterno (bottom = global).
- Un simbolo in uno scope interno **nasconde** (shadow) un simbolo con lo stesso nome in uno scope esterno.
- `contains_current(name)` e `current_symbol(name)` guardano solo lo scope corrente (per detectare ridichiarazioni).
- `lookup(name)`, `lookup_variable(name)`, `lookup_function(name)` attraversano tutti gli scope dall'interno verso l'esterno.
- Se un nome è trovato in uno scope esterno ma il simbolo è del tipo sbagliato (es. cercato come variabile ma è funzione), la ricerca si ferma a quel simbolo (non continua negli scope ancora più esterni).

### 17.5 Ordine delle operazioni nella dichiarazione di funzione

1. La funzione è dichiarata **nello scope corrente** (scope esterno alla funzione).
2. Viene aperto un nuovo scope `Function`.
3. Il contesto di funzione è impostato (`enter_function`).
4. I parametri sono dichiarati **nel nuovo scope di funzione**.
5. Il corpo della funzione è analizzato.
6. Viene verificata la presenza di un ritorno (se non void).
7. Il contesto di funzione è rimosso (`exit_function`).
8. Lo scope di funzione è chiuso.

**Implicazione:** la funzione può chiamare sé stessa ricorsivamente (il suo nome è già nello scope esterno al momento di analisi del corpo).

---

## 18. Codici di errore

I codici di errore semantici generati dal type checker sono i seguenti:

| Codice | Condizione |
| --- | --- |
| `E1003` | Target di assegnazione non valido (non `Variable` né `ArrayAccess`) |
| `E1005` | Operatore binario non riconosciuto (generato nel parser) |
| `E2002` | Tipo non assegnabile al tipo annotato o al tipo del target |
| `E2003` | Funzione non-void senza percorso di ritorno completo |
| `E2004` | Condizione non booleana in `if`, `while`, o `for` |
| `E2005` | `return` fuori da qualsiasi funzione |
| `E2006` | `return` con valore in funzione void |
| `E2007` | Tipo del valore di ritorno incompatibile con il tipo di ritorno dichiarato |
| `E2008` | `return` senza valore in funzione non-void |
| `E2009` | `break` fuori da un loop |
| `E2010` | `continue` fuori da un loop |
| `E2011` | Operatore bitwise/shift con operandi non interi |
| `E2012` | Operatore logico (`&&`, `\|\|`) con operandi non booleani |
| `E2013` | Operatore aritmetico con operandi non numerici/incompatibili |
| `E2014` | Operatore di confronto con tipi incompatibili |
| `E2016` | Operazione aritmetica su tipo non numerico |
| `E2017` | Operazione logica su tipo non booleano |
| `E2018` | Operatore unario (`-`, `~`, `++`, `--`) con operando non numerico/non intero |
| `E2019` | Operatore `!` con operando non booleano |
| `E2020` | Array literal vuoto |
| `E2021` | Array literal con elementi di tipo misto |
| `E2022` | Uso di un nome di funzione come variabile |
| `E2023` | Variabile non dichiarata |
| `E2024` | Assegnazione/mutazione di variabile immutabile |
| `E2025` | Assegnazione a variabile non dichiarata (in `Expr::Assign`) |
| `E2026` | Callee non è un nome di funzione |
| `E2027` | Funzione non dichiarata |
| `E2028` | Numero di argomenti errato nella chiamata |
| `E2029` | Tipo di argomento incompatibile con il tipo del parametro |
| `E2030` | Indice di array non intero |
| `E2031` | Indicizzazione di tipo non array/vector |
| `E2032` | Identificatore già dichiarato nello scope corrente |

---

## 19. Vincoli strutturali di validità dell'AST

Indipendentemente dalla semantica, un'istanza dell'AST è **strutturalmente valida** se e solo se:

1. Ogni `Box<Expr>` o `Box<Stmt>` è non-null (invariante garantito da Rust; non applicabile per implementazioni in altri linguaggi).
2. `Stmt::VarDeclaration.bindings` ha lunghezza ≥ 1.
3. `Stmt::Function.body` è una variante `Stmt::Block`.
4. `Stmt::MainFunction.body` è una variante `Stmt::Block`.
5. `Stmt::If.then_branch` è una variante `Stmt::Block`.
6. `ElseBranch::Block(stmt)`: `stmt` è una variante `Stmt::Block`.
7. `ElseBranch::ElseIf(stmt)`: `stmt` è una variante `Stmt::If`.
8. `Expr::Variable.name` è non vuoto.
9. `Expr::Call.callee` è una variante `Expr::Variable` (vincolo semantico, non strutturale; l'AST ammette altri callee ma il type checker li rifiuta).
10. `Type::Custom.name` è non vuoto.
11. `Type::Array.element_type` non è `Type::Void` né `Type::NullPtr`.
12. `Type::Vector.element_type` non è `Type::Void` né `Type::NullPtr`.
13. `Parameter.type_annotation` non è `Type::Void` né `Type::NullPtr`.
14. `Stmt::VarDeclaration.type_annotation` non è `Type::Void` né `Type::NullPtr`.
15. `Number::Scientific32(base, _)` e `Number::Scientific64(base, _)`: non ci sono vincoli strutturali sul valore dell'esponente.
16. `SourceSpan.start.offset <= SourceSpan.end.offset` (span well-formed).

---

## 20. Casi limite e ambiguità risolte

### 20.1 Array vuoto

`Expr::ArrayLiteral { elements: vec![], span }` è strutturalmente valido ma produce errore semantico `E2020`. Il type checker restituisce `None` come tipo dopo aver registrato l'errore.

### 20.2 `nullptr` come tipo

`Type::NullPtr` è il tipo inferito da `Expr::Literal { value: LiteralValue::NullPtr, ... }`. Non può essere usato come tipo annotato (in parametri, dichiarazioni, array) ma può essere assegnato a `Array`, `Vector`, o `Custom`.

### 20.3 Promozione signed/unsigned

La gerarchia di promozione **non** produce promozioni cross-signed. `I8` è promosso solo verso `I16`, `I32`, `I64`, `F32`, `F64`; non verso `U8`, `U16`, ecc. `U8` è promosso solo verso `U16`, `U32`, `U64`, `F32`, `F64`. Pertanto `I32 + U32` non è ammesso senza cast esplicito; il type checker emette `E2013`.

### 20.3.1 Chiarimento su `I64 + F32`, `U64 + F32` e sulla gerarchia totale

La regola del §20.3 che vieta la promozione **diretta** tra le due catene signed e unsigned rimane valida. Essa non vieta le conversioni intero → floating-point usate esclusivamente dalla promozione operatoriale del §14.2.2.

Di conseguenza:

| Espressione | Tipo promosso | Motivazione |
| --- | --- | --- |
| `I64 + F32` | `F32` | `I64` viene convertito implicitamente in `F32` per l'operatore |
| `U64 + F32` | `F32` | `U64` viene convertito implicitamente in `F32` per l'operatore |
| assegnamento `I64 -> F32` | non assegnabile | la conversione non appartiene a `is_assignable` |
| assegnamento `U64 -> F32` | non assegnabile | la conversione non appartiene a `is_assignable` |
| `I32 + U32` | errore `E2013` | nessuna promozione diretta tra catene signed/unsigned |

La frase del §20.3 relativa a `I32 + U32` va quindi interpretata come una regola sulla coppia di **interi signed/unsigned**, non come un divieto generale di conversione da intero a floating-point durante gli operatori.

Il ranking totale del §14.2 non costituisce da solo un insieme di conversioni valide. Esso seleziona il tipo promosso **dopo** che le regole di compatibilità della promozione operatoriale hanno stabilito quali conversioni sono ammissibili.

### 20.4 Funzione che chiama sé stessa (ricorsione)

Valida: la funzione è inserita nella symbol table prima che il suo corpo sia analizzato (vedi §17.5).

### 20.5 Shadowing

Un nome può essere ridichiarato in uno scope interno a quello in cui è stato dichiarato. Il simbolo interno nasconde quello esterno. Non è un errore. È un errore solo ridichiarare lo stesso nome **nello stesso scope**.

### 20.6 `for` senza condizione

`Stmt::For { condition: None, ... }` è valido; rappresenta un loop infinito (`for(;;)`). Non produce errori semantici relativi al tipo della condizione.

### 20.7 Dimensioni di array non valutabili staticamente

`Type::Array { size: expr }` dove `expr` non è un letterale intero positivo: la dimensione è trattata come `None` da `get_size`. Due array con dimensioni non valutabili vengono confrontati strutturalmente (`expr1 == expr2`). L'assegnabilità tra due array con dimensioni non valutabili restituisce `false` se `get_size` restituisce `None` per entrambi.

### 20.8 Confronto tra `Char` e `String`

`Char` è assegnabile a `String` (la promozione `Char → String` è definita in `is_assignable`). Non vale l'inverso.

### 20.9 `Grouping` e mutabilità

`Expr::Grouping { expr: Box<Expr::ArrayAccess { ... } }` è trattato da `base_variable_name` come equivalente all'`ArrayAccess` interno per le verifiche di mutabilità. La verifica di mutabilità attraversa trasparentemente `Grouping` e `ArrayAccess` per estrarre il nome base della variabile.

### 20.10 Operatori `++`/`--` su array access

`Increment` o `Decrement` applicati a `Expr::ArrayAccess { array, ... }` verifica la mutabilità della variabile base di `array` tramite `base_variable_name`. Se la variabile base è immutabile, errore `E2024`.

### 20.11 `main` e symbol table

`Stmt::MainFunction` viene processato come `visit_function("main", &[], &Type::Void, body, span)`. Il nome `"main"` viene inserito nella symbol table globale come `Symbol::Function`. Una seconda dichiarazione di `main` produrrebbe `E2032`.

### 20.12 Uguaglianza strutturale di `Type`

Due istanze di `Type` sono uguali (nel senso di `PartialEq`) se e solo se sono la stessa variante con i campi ricorsivamente uguali. `Type::Array { element_type: Box::new(Type::I32), size: e1 }` è uguale a `Type::Array { element_type: Box::new(Type::I32), size: e2 }` solo se `e1 == e2` strutturalmente (non per valore semantico). Il confronto per valore semantico della dimensione usa `is_same_type`, che chiama `get_size`.

### 20.13 Errori multipli e continuazione

Il type checker **non si ferma** al primo errore: raccoglie tutti gli errori in `Vec<CompileError>` e li restituisce tutti al termine di `check()`. Quando la verifica di un sotto-nodo fallisce (ritorna `None`), il type checker continua l'analisi degli altri sotto-nodi (es. continua a visitare gli argomenti di una chiamata anche se la funzione non esiste).

---

*Fine della specifica tecnica dell'AST Descar — versione 1.0*
