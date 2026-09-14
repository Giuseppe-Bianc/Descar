<!--
Sync Impact Report
Version change: N/A → 1.0.0
Added Principles: 1-5
Added Sections: Section 2 (Constraints), Section 3 (Development Workflow)
-->
# Descar Constitution

## Core Principles

### Stack Tecnologico e Build

Il progetto è scritto in Rust, utilizza Cargo per gestione dipendenze, compilazione, test e attività di sviluppo. Il workspace comprende crate `descar-core`, `descar-cli`, `descar`. Non introdurre strumenti di altri ecosistemi.

### Architettura

Separazione chiara delle responsabilità tra crate: `descar-core` fornisce funzioni base del compilatore, `descar-cli` gestisce interfaccia a riga di comando, `descar` è entry point e composizione. Evitare dipendenze circolari o verso livelli superiori.

### Principi di Sviluppo Rust

Seguire convenzioni idiomatiche Rust: ownership/borrowing corretto, gestione esplicita di Result/Option, error handling idiomatico, API semplici, tipi espressivi, eliminare duplicazioni, mantenere leggibilità, evitare `unsafe` salvo motivazione documentata.

### Formattazione, Linting e Testing

Formattare con rustfmt (`cargo fmt --all -- --check`). Lint con Clippy (`cargo clippy --workspace --all-targets --all-features -- -D warnings`). Test con `cargo test` e snapshot testing con `insta` quando opportuno. Coprire casi ordinari, corner, edge, error handling, CLI testing. Garantire test deterministici, isolati.

### Governance, CI e Dipendenze

Versionare con Semantic Versioning. CI esegue fmt, clippy, test su più OS. Aggiungere dipendenze solo se necessarie, preferire standard library. Documentare modifiche con Conventional Commits. Rispettare la regola di precedenza: requisiti funzionali > architettura > Rust/Cargo > dipendenze > best practice; i vincoli di sicurezza, compliance e approvazione delle dipendenze non sono derogabili.

## Constraints

- No Maven, Gradle, Java, JUnit o altri strumenti non Rust.
- Rust edition deve rispettare versione dichiarata nel workspace.
- Dipendenze devono rispettare versioni definite in Cargo.toml.
- Evitare dipendenze inutili; preferire standard library.
- Utilizzare solo crate approvati dopo valutazione di sicurezza.

## Development Workflow

- Seguire Test-Driven Development: scrivere test che falliscono, implementare la soluzione minima, rifattorizzare.
- Ogni modifica al codice o al comportamento richiede un test corrispondente (unit, integrazione, snapshot). Le modifiche a documentazione, configurazione o governance richiedono la validazione appropriata.
- CI pipeline verifica formattazione, Clippy e tutti i test su Linux, macOS, Windows.
- Pull request richiedono CI verde e revisione conforme ai principi.
- Incrementare versione secondo Semantic Versioning per modifiche retro‑incompatible, aggiunta di funzionalità, o correzioni.

## Governance

- Constitution supera tutte le altre linee guida del progetto.
- Modifiche richiedono documentazione, approvazione e piano di migrazione.
- Tutti i PR devono verificare conformità a ciascun principio prima del merge.
- Utilizzare questo documento come fonte autoritaria per decisioni progettuali.

**Version**: 1.0.0 | **Ratified**: 2026-09-14  | **Last Amended**: 2026-09-14
