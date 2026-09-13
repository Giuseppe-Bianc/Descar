# Specifica tecnica Fase 01, Inizializzazione del contesto semantico

Serie: Descar Type Checker Technical Specifications

Document ID: DTC-01

Phase slug: semantic_context_initialization

Status: Normative baseline for design and implementation, with explicit open decisions

Repository baseline: Giuseppe-Bianc/Descar, commit aff20d7fcbbea23b9be431063f997e06466ad3da

## 1. Scopo e obiettivi

Costruisce il contesto persistente e deterministico nel quale opereranno tutte le fasi semantiche successive.

- Creare il `SemanticContext` principale e i suoi sottosistemi.

- Inizializzare built-in types e simboli predefiniti senza introdurre nomi di programma.

- Inizializzare il `DiagnosticEngine` e il sistema di source locations.

- Stabilire policy e identificatori stabili per SymbolId, ScopeId e TypeId.

- Verificare la coerenza minima dell'AST ricevuto come input.

## 2. Responsabilità specifiche

Responsibility 1: Creare il `SemanticContext` principale e i suoi sottosistemi.

Responsibility 2: Inizializzare built-in types e simboli predefiniti senza introdurre nomi di programma.

Responsibility 3: Inizializzare il `DiagnosticEngine` e il sistema di source locations.

Responsibility 4: Stabilire policy e identificatori stabili per SymbolId, ScopeId e TypeId.

Responsibility 5: Verificare la coerenza minima dell'AST ricevuto come input.

## 3. Perimetro funzionale

### 3.1 Incluso

- Context lifecycle.

- Type interning e canonical built-ins.

- Global scope root.

- Registrazione di builtin symbols.

- Configurazione linguistica e versioning del semantic model.

### 3.2 Escluso

- Costruzione dei lexical scope del programma oltre il global root.

- Raccolta di dichiarazioni utente.

- Name resolution.

- Type checking delle espressioni e statement.

- Control-flow analysis.

## 4. Prerequisiti e precondizioni

La fase può essere avviata solo quando le sue dipendenze sono soddisfatte e quando le strutture di input rispettano gli invarianti dichiarati dalle fasi precedenti.

- L'AST deve essere strutturalmente valido secondo il contratto parser -> semantic analyzer.

- Tutte le strutture richieste dalla fase precedente devono essere disponibili tramite API tipizzate, non tramite accesso ad hoc ai dettagli interni.

- Gli identificatori semantici devono avere lifetime sufficiente a coprire l'intera compilation session.

- Il componente deve fallire in modo diagnostically controlled e non tramite panic su input semanticamente errato.

## 5. Input, struttura, formato e validità

### 5.1 Contratto AST

Le strutture AST correnti rilevanti dipendono dalla fase. Il design deve usare enum/tag strutturali e non assumere un layout binario o mutabile del nodo.

### 5.2 Identità e lifetime

SymbolId, ScopeId e TypeId devono essere handle semantici stabili. L'implementazione può usare arena allocation, interning, index-based IDs o smart handles; il contratto funzionale non deve dipendere dalla rappresentazione concreta.

## 6. Output, struttura, formato e validità

- `SemanticContext`.

- `GlobalScopeId`.

- Canonical `TypeId` per tutti i builtin types.

- Registry di builtin symbols.

- Diagnostic sink pronto all'uso.

Un output è valido solo se soddisfa sia i requisiti della fase sia tutti gli invarianti dichiarati nella sezione 11. Output parziali sono ammessi solo quando esplicitamente marcati come recovery artifacts.

## 7. Modello dati e strutture interne

## 8. Trasformazioni ed elaborazioni

initialize() { validateAstShape(); createGlobalScope(); internBuiltins(); registerBuiltins(); initializeDiagnostics(); initializeCaches(); freezeContextConfiguration(); }

L'algoritmo precedente è pseudocodice normativo a livello di comportamento. L'implementazione concreta può usare visitor, indexed traversal, worklist, query engine o una combinazione, purché preservi gli stessi risultati e invarianti.

## 9. Regole di Type Checking applicate

Le regole di questa sezione sono requisiti per la fase. Le regole di type equivalence, compatibility, subtyping e conversione devono essere fornite da servizi condivisi e non reimplementate in moduli locali.

## 10. Successo, fallimento ed error handling

Successo: tutte le precondizioni sono soddisfatte e non esistono errori semantici classificati come invalidanti per questa fase. Fallimento: l'output non può soddisfare i suoi invarianti senza una decisione non deterministica o senza ignorare una regola obbligatoria. In presenza di errori recuperabili, la fase deve produrre output parziale valido con recovery markers.

### 10.1 Errori rilevabili

- AST interno incoerente o nodo impossibile.

- Builtin type duplicato o non registrabile.

- Builtin symbol name collision con reserved runtime namespace.

- Source map assente per una compilation unit che dichiara diagnostica obbligatoria.

### 10.2 Criteri di classificazione

- Error: violazione di una regola semantica obbligatoria del linguaggio.

- Warning: condizione consentita ma potenzialmente sospetta, solo se prevista dalla language policy.

- Fatal/Internal: violazione di un invariante del semantic engine o impossibilità di proseguire in modo sound.

- Recovery marker: stato interno usato per impedire cascade errors, non sostitutivo di un vero diagnostico.

## 11. Postcondizioni e invarianti

- Nessun handle prodotto dalla fase punta a memoria o strutture non più vive nella compilation session.

- Le relazioni strutturali create dalla fase sono acicliche salvo cicli esplicitamente ammessi dal modello, come i back-edge del CFG.

- Ogni elemento semanticamente risolto è identificabile in modo stabile.

- Gli errori recuperabili non corrompono i risultati validi prodotti in precedenza.

- L'ordine di iterazione delle mappe o degli hash non può modificare il risultato osservabile della fase.

## 12. Dipendenze

Dipendenze immediate:

- AST parser-completed.

La dipendenza è contrattuale: la fase non può assumere che una fase successiva abbia già prodotto i dati necessari.

## 13. Informazioni rese disponibili alle fasi successive

- `SemanticContext`.

- `GlobalScopeId`.

- Canonical `TypeId` per tutti i builtin types.

- Registry di builtin symbols.

- Diagnostic sink pronto all'uso.

- API query read-only per i dati ormai stabilizzati.

- Traceability source node -> semantic artifact.

- Failure/recovery status necessario a decidere se il lowering sia consentito.

## 14. Interazioni con altri componenti

## 15. Casi limite e condizioni eccezionali

- AST vuoto.

- Compilation unit senza dichiarazioni.

- AST con nodi che usano span di file diversi.

- Re-inizializzazione accidentale dello stesso context.

- Input semanticamente errato ma sintatticamente valido.

- Input vuoto o compilation unit minima.

- Dati già parzialmente annotati da un run precedente, se incremental mode sarà supportato.

- Overflow o limiti di profondità nelle strutture ricorsive.

- Memory pressure nelle arena e cache. La semantica non deve trasformare un out-of-resource in un falso errore di tipo.

## 16. Ambiguità e non determinismo

Qualunque scelta non esplicitamente determinata dalla specifica deve essere classificata come open decision. Non è ammesso usare l'ordine di iterazione di `HashMap`, l'indirizzo in memoria o l'ordine casuale di allocazione per risolvere ambiguities semantiche.

- Candidate ordering deve essere derivato da regole semanticamente definite.

- Source ordering può essere usato solo come tie-breaker diagnostico, mai come semantica implicita.

- Error recovery deve essere idempotent rispetto a una stessa causa primaria quando il nodo viene riesaminato.

## 17. Consistenza, completezza e validazione

- Ogni output deve essere validabile tramite invariant checker dedicato.

- Ogni rule ID deve essere collegabile almeno a un test positivo o negativo.

- Ogni error class deve avere almeno un fixture di regressione.

- I risultati devono essere deterministici a parità di AST, configuration e builtin environment.

- I servizi centrali di type relation devono essere riutilizzati da tutte le fasi che confrontano tipi.

## 18. Esempi tecnici

### Inizializzazione minima

ctx = SemanticContext::new();
assert!(ctx.global_scope().is_some());
assert_eq!(ctx.types().builtin(Builtin::I32), ctx.types().builtin(Builtin::I32));

## 19. Implementazione e punti di estensione

L'implementazione deve essere modulare. La fase deve dipendere da trait o interfacce semanticamente stabili dove questo riduce coupling e rende sostituibili le strategie interne.

- Preferire query services come `lookupName`, `resolveType`, `isEqualType`, `isSubtype`, `isAssignable`, `classifyConversion`.

- Separare dati di sintassi da metadata semantici; non usare stringhe come surrogate keys quando esistono TypeId/SymbolId.

- Conservare source spans e source-node identity per diagnostics e test snapshot.

- Prevedere feature gates per generics, classes, nullable types e overloads non ancora presenti nell'AST attuale.

- Evitare una dipendenza diretta della fase dall'IR, salvo il contratto di handoff dell'ultima fase.

## 20. Criteri di accettazione e verifica

ACCEPT: Verifica unicità dei builtin TypeId.

ACCEPT: Verifica che due query allo stesso builtin restituiscano la stessa identità.

ACCEPT: Verifica che il global scope sia sempre presente.

ACCEPT: Verifica determinismo degli ID a parità di input e configurazione.

1. Tutti i test normativi della fase devono passare senza dipendere dall'ordine accidentale delle collezioni.

1. Gli input validi devono produrre output conforme agli invarianti.

1. Gli input invalidi devono produrre almeno il diagnostico primario previsto.

1. I casi di recovery devono continuare l'analisi senza panic e senza cascata non controllata.

1. Il test di integrazione deve dimostrare che le informazioni richieste dalla fase successiva sono effettivamente disponibili.

## 21. Decisioni progettuali ancora aperte

- Set completo dei builtin types da definire nella specifica del linguaggio.

- Nome e semantica dei builtin symbols.

- Policy per sessioni incrementali o IDE.

- Persistenza degli ID tra compilation unit distinte.

Queste decisioni non sono requisiti impliciti. Devono essere deliberate in una language specification separata oppure in ADR versionati prima di codificarne il comportamento come vincolo definitivo.

## 22. Mappatura al repository Descar

Il repository corrente fornisce già alcune strutture sulle quali questa fase dovrà innestarsi; dove una funzione richiesta non esiste ancora, la specifica la tratta come nuova componente estensibile senza modificare la repository in questa attività.

- Il workspace Cargo contiene i crate `descar-core`, `descar-cli` e `descar`.

- La documentazione pubblica del progetto descrive `descar-core` come nucleo del compilatore e dichiara una fase semantica distinta prima dell'IR.

- L'AST espressione attuale contiene `Binary`, `Unary`, `Grouping`, `Literal`, `ArrayLiteral`, `Variable`, `Assign`, `Call` e `ArrayAccess`.

- L'AST statement attuale contiene dichiarazioni di variabili, funzioni, `if`, `while`, `for`, blocchi, `return`, `break`, `continue` e `main`.

- Il tipo sintattico attuale include interi firmati e non firmati, floating point, `Char`, `String`, `Bool`, `Custom`, array a dimensione fissa, `Vector`, `Void` e `NullPtr`.

- I parametri sono rappresentati con nome, annotazione di tipo e `SourceSpan`.

- Il sistema di errori riserva E2001-E2999 alla semantic analysis e dispone di `CompileError::TypeError` con codice, messaggio, span e help opzionale.

Punto di estensione raccomandato: introdurre il componente sotto `descar-core` mantenendo separato il namespace semantico da `syntax` e `error` dove il coupling non è necessario.

## 23. Riferimenti

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

| Input | Struttura | Significato | Validità |
| --- | --- | --- | --- |
| AST parser-completed. | AST / semantic service / context | Dato consumato dalla fase | Validato prima dell'uso |
| Source map o equivalente per la conversione span -> location. | AST / semantic service / context | Dato consumato dalla fase | Validato prima dell'uso |
| Configurazione del linguaggio. | AST / semantic service / context | Dato consumato dalla fase | Validato prima dell'uso |
| Eventuali builtin declarations fornite dal runtime/toolchain. | AST / semantic service / context | Dato consumato dalla fase | Validato prima dell'uso |

### Tabella 2

| Struttura | Campi minimi | Semantica | Invariante |
| --- | --- | --- | --- |
| SemanticContext | symbol arena, type arena, scope root, diagnostics | Contesto globale | Univocità degli handle |
| Scope | id, parent, owner, namespaces | Regione di visibilità | Parent chain acyclic |
| Symbol | id, name, kind, declaration, type/signature ref | Entità dichiarata | One declaration identity per symbol |
| Type | id, kind, parameters/payload | Tipo canonico | Equality indipendente dalla spelling |
| FlowState | reachable, assigned, narrowed | Stato di percorso, se applicabile | Merge conservativo |
| Diagnostic | code, severity, span, message, related | Errore o informazione semantica | Deterministico |

### Tabella 3

| ID | Normative level | Requirement | Verification intent |
| --- | --- | --- | --- |
| CTX-REQ-001 | MUST | ogni tipo builtin deve avere identità canonica unica per sessione di compilazione. | Implementable and testable. |
| CTX-REQ-002 | MUST | ogni SymbolId, ScopeId e TypeId deve essere univoco nella compilation unit. | Implementable and testable. |
| CTX-REQ-003 | MUST | il contesto deve essere creato in stato iniziale vuoto rispetto ai simboli utente. | Implementable and testable. |
| CTX-REQ-004 | MUST | nessuna API di semantic analysis deve dipendere da stringhe di tipo non canonicalizzate. | Implementable and testable. |

### Tabella 4

| Componente | Interazione | Vincolo |
| --- | --- | --- |
| Parser / AST | consume | No mutation required unless semantic AST strategy explicitly chosen |
| DiagnosticEngine | emit/query | Stable codes and spans |
| TypeContext | query/update | Centralized relations and canonical types |
| SymbolTable | lookup/insert/query | No duplicate independent registries |
| IR Generator | handoff only | Must receive resolved semantics, not unresolved syntax |
| Tests / snapshots | verify | Stable output and deterministic ordering |
