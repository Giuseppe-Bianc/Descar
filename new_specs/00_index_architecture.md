# Indice della specifica tecnica del Type Checker Descar

Repository baseline: Giuseppe-Bianc/Descar, commit aff20d7fcbbea23b9be431063f997e06466ad3da

Questa raccolta contiene un documento autonomo per ciascuna fase. Nessun file viene scritto nella repository Descar.

## 1. Architettura della pipeline

AST valido
  -> 01 Context
  -> 02 Scopes
  -> 03 Declarations
  -> 04 Type resolution
  -> 05 Signature validation
  -> 06 Aggregates/members
  -> 07 Name resolution
  -> 08 Expression typing
  -> 09 Conversions
  -> 10 Overloads
  -> 11 Generics/constraints
  -> 12 Statement checking
  -> 13 Flow analysis
  -> 14 Global validation
  -> 15 Diagnostics/recovery
  -> 16 Semantic AST and handoff

## 2. Ordine e dipendenze

## 3. Contratti trasversali

- TypeId e SymbolId sono handle semantici stabili, non stringhe di convenienza.

- Type equivalence, compatibility, subtyping e conversion classification sono servizi centrali e non devono essere duplicati per fase.

- Gli errori semanticamente recoverable usano ErrorType/ErrorSymbol per limitare cascades.

- Ogni fase deve mantenere determinismo e verificabilità tramite test associati a requirement IDs.

- Il semantic analyzer deve poter operare senza modificare la repository durante la fase di progettazione e specifica.

## 4. Baseline reale del repository

- Il workspace Cargo contiene i crate `descar-core`, `descar-cli` e `descar`.

- La documentazione pubblica del progetto descrive `descar-core` come nucleo del compilatore e dichiara una fase semantica distinta prima dell'IR.

- L'AST espressione attuale contiene `Binary`, `Unary`, `Grouping`, `Literal`, `ArrayLiteral`, `Variable`, `Assign`, `Call` e `ArrayAccess`.

- L'AST statement attuale contiene dichiarazioni di variabili, funzioni, `if`, `while`, `for`, blocchi, `return`, `break`, `continue` e `main`.

- Il tipo sintattico attuale include interi firmati e non firmati, floating point, `Char`, `String`, `Bool`, `Custom`, array a dimensione fissa, `Vector`, `Void` e `NullPtr`.

- I parametri sono rappresentati con nome, annotazione di tipo e `SourceSpan`.

- Il sistema di errori riserva E2001-E2999 alla semantic analysis e dispone di `CompileError::TypeError` con codice, messaggio, span e help opzionale.

## 5. Scostamento tra AST attuale e target estensibile

## 6. Riferimenti

- Repository: Descar README. https://github.com/Giuseppe-Bianc/Descar/blob/aff20d7fcbbea23b9be431063f997e06466ad3da/README.md

- Repository: Descar AST expression model. https://github.com/Giuseppe-Bianc/Descar/blob/aff20d7fcbbea23b9be431063f997e06466ad3da/crates/descar-core/src/syntax/ast/expr.rs

- Repository: Descar AST statement model. https://github.com/Giuseppe-Bianc/Descar/blob/aff20d7fcbbea23b9be431063f997e06466ad3da/crates/descar-core/src/syntax/ast/stmt.rs

- Repository: Descar AST type model. https://github.com/Giuseppe-Bianc/Descar/blob/aff20d7fcbbea23b9be431063f997e06466ad3da/crates/descar-core/src/syntax/ast/ast_type.rs

- Repository: Descar semantic error taxonomy. https://github.com/Giuseppe-Bianc/Descar/blob/aff20d7fcbbea23b9be431063f997e06466ad3da/crates/descar-core/src/error/error_code.rs

- Repository: Descar CompileError. https://github.com/Giuseppe-Bianc/Descar/blob/aff20d7fcbbea23b9be431063f997e06466ad3da/crates/descar-core/src/error/compile_error.rs

- Repository: Descar source spans. https://github.com/Giuseppe-Bianc/Descar/blob/aff20d7fcbbea23b9be431063f997e06466ad3da/crates/descar-core/src/location/source_span.rs

- External: Clang Internals Manual, Sema. https://clang.llvm.org/docs/InternalsManual.html

- External: Rust Reference, Name Resolution. https://doc.rust-lang.org/reference/names/name-resolution.html

- External: Rust Reference, Scopes. https://doc.rust-lang.org/reference/names/scopes.html

- External: Rust Reference, Type Coercions. https://doc.rust-lang.org/reference/type-coercions.html

- External: TypeScript Handbook, Narrowing. https://www.typescriptlang.org/docs/handbook/2/narrowing.html

- External: Java Language Specification, Chapter 16. https://docs.oracle.com/javase/specs/jls/se23/html/jls-16.html

### Tabella 1

| Fase | Componente | Dipende da | Artefatto |
| --- | --- | --- | --- |
| 01 | Specifica tecnica Fase 01, Inizializzazione del contesto semantico | AST | Documento autonomo |
| 02 | Specifica tecnica Fase 02, Costruzione degli scope | 01 | Documento autonomo |
| 03 | Specifica tecnica Fase 03, Raccolta delle dichiarazioni e costruzione iniziale della symbol table | 01, 02 | Documento autonomo |
| 04 | Specifica tecnica Fase 04, Risoluzione dei riferimenti di tipo | 01, 02, 03 | Documento autonomo |
| 05 | Specifica tecnica Fase 05, Validazione delle dichiarazioni e delle firme | 01, 02, 03, 04 | Documento autonomo |
| 06 | Specifica tecnica Fase 06, Risoluzione di membri, aggregate e relazioni di tipo | 01, 02, 03, 04, 05 | Documento autonomo |
| 07 | Specifica tecnica Fase 07, Risoluzione dei nomi | 01, 02, 03, 04, 05, 06 | Documento autonomo |
| 08 | Specifica tecnica Fase 08, Type Checking delle espressioni | 01, 02, 03, 04, 05, 06, 07 | Documento autonomo |
| 09 | Specifica tecnica Fase 09, Analisi e inserimento delle conversioni | 01, 02, 03, 04, 05, 06, 07, 08 | Documento autonomo |
| 10 | Specifica tecnica Fase 10, Inferenza dei generics e constraint solving | 01, 02, 03, 04, 05, 06, 07, 08, 09 | Documento autonomo |
| 11 | Specifica tecnica Fase 10, Risoluzione degli overload | 01, 02, 03, 04, 05, 06, 07, 08, 09, 10 | Documento autonomo |
| 12 | Specifica tecnica Fase 12, Type Checking degli statement | 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11 | Documento autonomo |
| 13 | Specifica tecnica Fase 13, Control Flow e analisi flow-sensitive | 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12 | Documento autonomo |
| 14 | Specifica tecnica Fase 14, Validazione semantica globale | 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13 | Documento autonomo |
| 15 | Specifica tecnica Fase 15, Diagnostica, classificazione e recovery | 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14 | Documento autonomo |
| 16 | Specifica tecnica Fase 16, Finalizzazione dell'AST semantico e handoff | 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15 | Documento autonomo |

### Tabella 2

| Capability | AST attuale | Specificato | Strategia |
| --- | --- | --- | --- |
| Expressions | Binary, Unary, Grouping, Literal, ArrayLiteral, Variable, Assign, Call, ArrayAccess | Completo per l'incremento iniziale | Estendere semantic side tables e aggiungere nodi solo quando richiesti |
| Statements | Var, Function, If, While, For, Block, Return, Break, Continue, Main | Flow-sensitive completo | CFG/flow layer indipendente dalla sintassi |
| Types | Primitive, Custom, Array, Vector, Void, NullPtr | Generics, optional/nullability e subtyping | TypeContext estendibile |
| Members/classes | Non presenti nel modello AST corrente | Specificati come extension point | Aggiungere aggregate AST in futura language revision |
| Diagnostics | E2001-E2999, TypeError con span/message/help | Structured diagnostics complete | Evolvere internals mantenendo compatibilità degli error codes |
