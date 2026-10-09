# Guida al logging di Descar

Oct 9, 2026 · @g\_bian\_ders

Descar registra eventi con `tracing`, in testo su stderr, con `-v` = WARN, `-vv` = DEBUG, `-vvv` = TRACE e `-q` = nessun evento di log. L'implementazione è già eseguita e verificata su una copia del repository: 17 test di integrazione, 5 test unitari e 855 test nel workspace passano, `clippy` e `fmt` sono puliti. La sezione 1.3 elenca le decisioni che richiedono la tua conferma prima del merge.

## Esito e stato di verifica

Il logging è implementato in una copia di lavoro del repository e verificato con 855 test del workspace, 0 falliti. Questa guida permette di ripetere ogni passo su un clone pulito.

| Elemento | Valore |
| --- | --- |
| Repository e commit di partenza | `Giuseppe-Bianc/Descar`, commit `2af7adf` del 2026-10-08 |
| Branch di lavoro | `feat/logging`, solo locale, nessun push |
| Ultima versione stabile di Rust al 2026-10-09 | 1.99.0, rilasciata il 2026-10-01 |
| Toolchain usata per la verifica | rustc 1.97.0 e cargo 1.97.0 |
| File modificati | 10 file, 522 righe aggiunte, 75 rimosse (`Cargo.lock` escluso dal conteggio) |
| Test aggiunti | 17 di integrazione, 5 unitari, 3 sul parsing CLI |
| Test del workspace | 39 suite, 855 test passati, 0 falliti, snapshot `insta` invariati |
| `cargo clippy ... -D warnings` | nessun avviso, con i lint `pedantic` e `nursery` del workspace |
| `cargo audit` | 0 vulnerabilità su 78 crate (database RustSec con 1296 advisory) |
| Binario di rilascio | da 2.018.752 a 2.272.080 byte, +253.328 byte (+12,5%) |

### Che cosa non è verificato

La sandbox contiene solo rustc 1.97.0 e non può scaricare altre toolchain. Il repository non compila con quella versione: `crates/descar-core/src/syntax/parser.rs`, riga 735, usa `Option::map_or_default`, che in 1.97.0 è instabile (errore `E0658`). Per verificare ho sostituito quella riga con `.map(|t| t.span.clone()).unwrap_or_default()` solo nella sandbox. La sostituzione non è nella patch. Il passo 1.4 controlla che la tua toolchain compili la riga originale.

Tre verifiche restano a carico tuo o della CI del repository:

1. Esecuzione su Windows e su macOS. La sandbox è Linux x86\_64. La CI del repository esegue già entrambi i sistemi.
2. `cargo clippy` con la 1.99.0. Una versione più recente può aggiungere lint che la 1.97.0 non conosce.
3. Colori su una console Windows legacy. Il codice usa `console::colors_enabled_stderr()`, la stessa funzione che già governa gli stili di `console::style` nel binario.

### Come usare la guida

- Percorso A, manuale. Segui la sezione 4 passo per passo. Ogni passo indica il file, il contenuto e il risultato atteso.
- Percorso B, patch. Applica `descar-logging.patch` con `git apply`, poi esegui i controlli della sezione 7. Il comando è nel passo 4.1.

## 1. Analisi iniziale

Le dodici informazioni della tabella 1.1 sono classificate per origine. Tre decisioni (D1, D3, D4) cambiano un comportamento visibile all'utente e richiedono la tua conferma.

### 1.1 Informazioni classificate

Origine: **Fornita** = scritta da te. **Verificata** = letta nel repository o misurata. **Assunzione** = scelta necessaria senza dato. **Raccomandazione** = scelta mia, modificabile. **Da confermare** = richiede la tua risposta.

| # | Informazione | Valore | Origine |
| --- | --- | --- | --- |
| 1 | Tipo di applicazione | CLI di un compilatore per il linguaggio Descar (file `.dr`). Workspace con tre crate: `descar-core`, `descar-cli`, `descar`. Sottocomandi `compile` e `check` | Verificata (`Cargo.toml`, `cli.rs`) |
| 2 | Sistemi operativi | Linux, Windows, macOS, tutti x86\_64. La CI del repository usa `ubuntu-latest`, `macOS-latest`, `windows-latest` | Fornita, CI verificata |
| 3 | Target di deployment | Binario distribuito agli utenti | Fornita |
| 4 | Toolchain | Edition 2024, resolver 3, nessun `rust-version` (rimosso nel commit `2af7adf`). La CI prova stable, beta e nightly | Verificata |
| 5 | Requisiti di sviluppo e produzione | Formato testo in development, test e production | Fornita |
| 6 | Modalità di esecuzione | Processo breve, un file sorgente per invocazione, avviato da un sistema esterno | Verificata e fornita |
| 7 | Formato e destinazione | Testo. Su stderr: errori, warning, messaggi diagnostici. Su stdout: output funzionale | Fornita |
| 8 | Flag di verbosità | `-v` WARN, `-vv` DEBUG, `-vvv` TRACE, `-q` nessun log. Da mantenere | Fornita |
| 9 | Retention e rotazione | Gestite dal sistema che avvia la CLI. Il codice non scrive file di log e non ruota nulla | Fornita |
| 10 | Osservabilità, monitoraggio | Nessun requisito su metriche, tracing distribuito o esportazione. Non implementati | Non fornita |
| 11 | Livello senza flag | Non specificato. Applicato: ERROR | Assunzione D1 |
| 12 | Output funzionale esistente | Solo `--help` e `--version`. Il flag `-E` non esiste | Verificata, D2 |

### 1.2 Difetti trovati nel repository

Questi difetti precedono il mio intervento. Le prove vengono dall'esecuzione del binario originale e dal codice.

1. **Tutta la diagnostica esce su stdout.** `descar compile dr_files/simple_test.dr -vv` scrive su stdout "Compiling ...", la tabella delle dimensioni e "Compilation successful", e su stderr nulla. Il tuo contratto sui flussi è violato. Quando `--emit-ir` scriverà IR su stdout, queste righe lo corromperanno.
2. **La verbosità è un `match` duplicato con significati diversi.** `compile -v` stampa la tabella delle dimensioni. `check -v` la stampa solo da `-vv`. Non esistono livelli, filtri o contesto.
3. **`-q` e `-v` si accettano insieme.** `descar check f.dr -q -vv` termina con codice 0 e `-q` vince senza avviso.
4. **Quattro opzioni sono accettate e ignorate.** `main.rs` non legge mai `args.output`, `args.optimize`, `args.emit_ir`, `args.diagnostics`. Con `--emit-ir` il comando ha successo e non produce IR. Questo è un difetto del prodotto. Il logging lo rende visibile (D6).
5. **Una chiamata inutile.** Il file viene letto con `fs::read_to_string`, poi `fs::metadata` ne chiede la dimensione una seconda volta. Se la seconda chiamata fallisce, il codice stampa `ERROR:` e prosegue. Dopo una lettura riuscita la dimensione è già nota: `input.len()`.
6. **`insta` è una dipendenza normale di `descar-core`.** È una libreria per i test e compare nell'albero di `cargo tree -p descar -e normal`. Va in `[dev-dependencies]`. Non lo correggo: fuori dall'ambito del logging.
7. **La documentazione è errata.** `CLAUDE.md` afferma che `descar-core` non ha dipendenze esterne. `crates/descar-core/Cargo.toml` ne dichiara cinque: `insta`, `thiserror`, `console`, `regex`, `logos`.
8. **Nessuna versione minima di Rust dichiarata.** Il commit `2af7adf` ha rimosso `rust-version`, ma il codice richiede una toolchain più recente della 1.97.0 (vedi "Che cosa non è verificato").

### 1.3 Decisioni che richiedono la tua conferma

| ID | Decisione applicata | Tipo | Motivo | Come cambiarla |
| --- | --- | --- | --- | --- |
| D1 | Senza flag il livello massimo è ERROR | Assunzione | La mappatura confermata parte da `-v` = WARN. Il livello sotto WARN che segnala guasti è ERROR | Cambia `DEFAULT_LEVEL` in `logging.rs` e il caso `0` del test `maps_verbosity_to_level` |
| D2 | L'output funzionale di oggi è solo `--help` e `--version`. Il flag `-E` non esiste | Verificata | Ricerca nel repository: Descar ha `--emit-ir`, che non scrive nulla | Se `-E` è previsto, definiscine il comportamento prima di implementarlo |
| D3 | "Compiling ...", "Compilation successful" e la tabella delle dimensioni lasciano stdout. Diventano eventi INFO e DEBUG su stderr. **Senza flag l'utente non vede più "Compilation successful"** | Raccomandazione, cambio visibile | Applica la tua regola: stdout solo per l'output funzionale | Per mostrare l'esito senza flag, stampalo con `eprintln!` fuori dal logging e rispetta `-q` |
| D4 | Diagnostica del compilatore (errori di sintassi e di tipo) ed errori I/O fatali non sono eventi di log. Ignorano `-q` | Assunzione | Con `-q` un exit code 1 senza messaggio è inutilizzabile. Il testo di `--help` per `-q` dice "Suppress non-essential output" | Modifica `stop_frontend` e `handle_io_error` in `main.rs` |
| D5 | Nuove dipendenze `tracing` e `tracing-subscriber` | Da confermare | La costituzione (`.specify/memory/constitution.md`) consente solo crate approvati dopo valutazione di sicurezza e richiede di preferire la libreria standard | Se rifiuti, usa l'alternativa descritta nella sezione 3 |
| D6 | Un WARN per ogni opzione accettata e ignorata | Raccomandazione | Difetto 4 | Elimina `warn_ignored_options` e la sua chiamata |
| D7 | Nessun timestamp nelle righe | Raccomandazione | Il sistema esterno che raccoglie stderr aggiunge l'ora. L'output resta deterministico e testabile | Sostituisci `.without_time()` con `.with_timer(...)` |
| D8 | Nessuna variabile di ambiente seleziona il livello | Raccomandazione | Non richiesta. `RUST_LOG` in un binario distribuito cambierebbe il comportamento senza che l'utente lo sappia | Aggiungi in futuro una variabile `DESCAR_LOG` con `EnvFilter` |

### 1.4 Prerequisiti e verifica dell'ambiente

Esegui i comandi nell'ordine indicato. Passa al comando successivo solo se il risultato corrisponde a quello atteso.

1. Aggiorna la toolchain stabile.

   ```sh
   rustup update stable
   ```

   Risultato atteso: l'ultima riga riporta la versione installata. Al 2026-10-09 la versione stabile è 1.99.0.
2. Controlla le versioni.

   ```sh
   rustc --version
   cargo --version
   ```

   Risultato atteso: `rustc 1.99.0` e `cargo 1.99.0`, o versioni successive. Con una versione inferiore, ripeti il comando 1.
3. Installa i componenti di formattazione e analisi.

   ```sh
   rustup component add rustfmt clippy
   ```

   Risultato atteso: `rustfmt` e `clippy` risultano installati o aggiornati.
4. Clona il repository e crea il ramo di lavoro.

   ```sh
   git clone https://github.com/Giuseppe-Bianc/Descar.git
   cd Descar
   git switch -c feat/logging
   git rev-parse --short HEAD
   ```

   Risultato atteso: `Switched to a new branch 'feat/logging'`. L'ultimo comando stampa `2af7adf`. La patch di questa guida è stata verificata su quel commit.
5. Verifica la base di partenza prima di modificare qualunque file.

   ```sh
   cargo build --workspace
   cargo test --workspace --all-features
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```

   Risultato atteso: nessun errore. I test passano tutti: 38 suite, 830 test, 0 falliti. Se compare `error[E0658]: use of unstable library feature result_option_map_or_default`, la toolchain è troppo vecchia: vai al problema 9.1.

## 2. Progettazione del logging

Un solo punto del binario installa il sistema di log. I crate di libreria non lo installano. Tre canali restano separati: eventi di log, diagnostica del compilatore e output funzionale.

### 2.1 Architettura

| Canale | Contenuto | Scrittura | Effetto di `-q` |
| --- | --- | --- | --- |
| Eventi di log | Fasi, conteggi, durate, opzioni ignorate | `tracing` e `tracing-subscriber`, su stderr | Silenziati |
| Diagnostica del compilatore | Errori di lexer, parser e type checker formattati da `ErrorReporter` | `eprintln!` su stderr | Invariata |
| Errori fatali di I/O | `ERROR: I/O: failed to read ...` | `eprintln!` su stderr, funzione `handle_io_error` | Invariati |
| Output funzionale | `--help`, `--version` | `clap`, su stdout | Invariato |

Sequenza di esecuzione:

1. `Args::parse()` legge i flag.
2. `Args::logging()` restituisce i `LoggingArgs` del sottocomando scelto.
3. `logging::init()` calcola il livello con `level_filter()` e installa il subscriber `fmt`. Se il livello è spento, non installa nulla.
4. `main` apre lo span `compile` o `check` con il percorso del file.
5. Le funzioni emettono eventi con `warn!`, `info!`, `debug!` e `trace!`.
6. Il subscriber scrive ogni evento abilitato su stderr, una riga per evento.

`descar-core` e `descar-cli` non dipendono da `tracing`. Il core oggi non emette eventi, e la costituzione impone di evitare dipendenze inutili. Gli eventi descrivono le fasi viste dal binario: lexing, parsing, type checking. Per eventi interni al core aggiungi `tracing` a `descar-core` in un intervento separato.

### 2.2 Livelli

| Flag | Livello massimo |
| --- | --- |
| nessuno | ERROR (assunzione D1) |
| `-v` | WARN |
| `-vv` | DEBUG |
| `-vvv` o più | TRACE |
| `-q` | spento, ha precedenza su `-v` |

| Livello | Uso in Descar | Visibile con |
| --- | --- | --- |
| ERROR | Guasto interno inatteso e non fatale. Nessun evento lo usa oggi | Ogni flag tranne `-q` |
| WARN | Condizione anomala che non ferma il lavoro: opzione accettata e ignorata | `-v` e superiori |
| INFO | Esito del comando: `compilation succeeded`, `check succeeded` | `-vv` e superiori |
| DEBUG | Fasi della pipeline: opzioni, dimensione del file, conteggi, durate, motivo di arresto | `-vv` e superiori |
| TRACE | Inizio di ogni fase e lettura del file. La riga riporta `file:riga` | `-vvv` |

Nessun evento usa ERROR perché ogni guasto fatale è un messaggio per l'utente (D4). Un evento ERROR duplicato stamperebbe lo stesso errore due volte al livello predefinito.

La mappatura che hai confermato ha un punto debole: INFO e DEBUG compaiono entrambi da `-vv`, quindi nessun flag mostra il solo esito del comando. Il codice mantiene la mappatura richiesta. Per cambiarla modifica `level_filter`.

### 2.3 Struttura degli eventi

Regole per ogni evento nuovo:

1. Il messaggio è una frase breve, in inglese, minuscola e fissa. Non contiene valori. Una ricerca con `grep` trova così ogni occorrenza.
2. I valori stanno in campi `nome = valore`. I nomi usano `snake_case`.
3. Un percorso o un testo scritto dall'utente si formatta con `?` (Debug), mai con `%` (Display). La sezione 6 spiega il motivo.
4. Un campo contiene conteggi, dimensioni, durate o nomi di fase. Mai testo del sorgente.
5. Una durata si registra come `elapsed = ?istante.elapsed()`.

Eventi implementati:

| Livello | Messaggio | Campi | Funzione |
| --- | --- | --- | --- |
| DEBUG | `command started` | `optimize`, `emit_ir`, `diagnostics`, `output` (solo `compile`) | `main` |
| WARN | `option is accepted but has no effect yet` | `option` | `warn_ignored_options` |
| TRACE | `reading source file` | nessuno | `read_input` |
| DEBUG | `source file size` | `bytes`, `si`, `iec` | `log_source_size` |
| TRACE | `lexing started` | `source_len` | `run_frontend` |
| DEBUG | `lexing finished` | `tokens`, `errors`, `elapsed` | `run_frontend` |
| TRACE | `parsing started` | `tokens` | `run_frontend` |
| DEBUG | `parsing finished` | `statements`, `errors`, `elapsed` | `run_frontend` |
| TRACE | `type checking started` | `statements` | `run_frontend` |
| DEBUG | `type checking finished` | `errors`, `elapsed` | `run_frontend` |
| DEBUG | `frontend stopped` | `stage`, `errors` | `stop_frontend` |
| INFO | `compilation succeeded` o `check succeeded` | nessuno | `main` |

### 2.4 Contesto associato agli eventi

Ogni evento emesso dentro `compile` o `check` porta lo span `compile{file="..."}` o `check{file="..."}`. Lo span ha livello ERROR, quindi è attivo a ogni livello di log. Uno span di livello INFO sarebbe disabilitato con `-v`, e un evento WARN perderebbe il nome del file.

### 2.5 Gestione degli errori

| Situazione | Comportamento |
| --- | --- |
| Un subscriber globale è già installato | `init` restituisce un errore. `main` stampa una riga `WARNING: logging is disabled: ...` su stderr e prosegue. Il codice di uscita non cambia |
| Errore del compilatore | Evento DEBUG `frontend stopped`, poi il messaggio per l'utente su stderr, poi `exit(1)` |
| Errore fatale di I/O | Messaggio `ERROR: I/O: ...` su stderr, poi `exit(1)`. Con `-q` il messaggio resta |
| Panic | Gestore predefinito di Rust su stderr. Nessun hook personalizzato. `-q` non lo nasconde |
| `process::exit(1)` | Nessun evento va perso. `stderr` in Rust non è bufferizzato e il codice non usa writer non bloccanti. Il test `debug_trace_records_each_frontend_stage_and_its_failure` lo verifica |

Non introdurre `tracing-appender` con scrittura non bloccante senza conservare il `WorkerGuard` fino all'uscita. Con `process::exit` il guard non verrebbe rilasciato e gli ultimi eventi andrebbero persi.

### 2.6 Formato

Una riga di testo per evento, con questa struttura:

```text
LIVELLO span{campo="valore"}: target: [file:riga:] messaggio campo=valore
```

Esempio reale con `-vv`:

```text
DEBUG compile{file="dr_files/simple_test.dr"}: descar: lexing finished tokens=84 errors=0 elapsed=123.298µs
 INFO compile{file="dr_files/simple_test.dr"}: descar: compilation succeeded
```

Il livello occupa cinque caratteri allineati a destra, quindi `INFO` e `WARN` iniziano con uno spazio. Il campo `file:riga` compare solo con `-vvv`. Non c'è timestamp (D7). I colori ANSI compaiono solo se stderr è un terminale.

### 2.7 Destinazione

Gli eventi vanno solo su stderr. Il codice lo impone con `.with_writer(io::stderr)`. Nessun evento usa stdout. Il test `failing_source_never_writes_to_stdout_at_any_level` lo verifica a ogni livello.

### 2.8 Configurazione per ambiente

| Ambiente | Formato | Destinazione | Livello | Differenze |
| --- | --- | --- | --- | --- |
| Sviluppo | Testo | stderr | Dai flag | Nessuna |
| Test | Testo | stderr del processo figlio, letto dai test | Dai flag | Nessuna |
| Produzione (binario distribuito) | Testo | stderr | Dai flag | Nessuna |

Un solo comportamento in tutti gli ambienti elimina la differenza tra ciò che provi e ciò che l'utente esegue. Non abilitare le feature `max_level_*` o `release_max_level_*` di `tracing`: rimuoverebbero DEBUG e TRACE dal binario di rilascio, e `-vv` e `-vvv` non mostrerebbero nulla. Il binario di rilascio costruito per questa guida stampa gli eventi DEBUG con `-vv`.

## 3. Scelta delle dipendenze

Aggiungi `tracing` 0.1.44 e `tracing-subscriber` 0.3.23 con le feature minime. Il costo misurato è di 8 crate nuovi nell'albero normale (da 55 a 63) e di 253.328 byte (+12,5%) nel binario di rilascio. La scelta richiede la tua approvazione (D5).

| Crate | Versione | Ruolo | Dichiarazione | Licenza | Rust minimo dichiarato |
| --- | --- | --- | --- | --- | --- |
| `tracing` | 0.1.44 | Macro per eventi e span: `debug!`, `warn!`, `error_span!` | `default-features = false`, feature `std` | MIT | 1.65.0 |
| `tracing-subscriber` | 0.3.23 | Subscriber `fmt`: scrive una riga di testo per evento | `default-features = false`, feature `fmt` e `ansi` | MIT | 1.65.0 |

La versione minima dichiarata dai due crate (1.65.0) è inferiore all'edition 2024 del workspace, che richiede almeno Rust 1.85. Nessun crate limita quindi la toolchain del progetto. I dati vengono da `cargo info` sull'indice di crates.io.

### 3.1 Feature disattivate

| Crate | Feature | Motivo |
| --- | --- | --- |
| `tracing` | `attributes` (default) | Abilita `#[instrument]` e trascina `tracing-attributes`. Il codice non lo usa |
| `tracing` | `log`, `log-always` | Ponte verso il crate `log`. Nessuna dipendenza del workspace usa `log` |
| `tracing` | `max_level_*`, `release_max_level_*` | Vietate: tolgono DEBUG e TRACE dal binario distribuito |
| `tracing-subscriber` | `tracing-log`, `smallvec` (default) | Ponte da `log` non necessario. `smallvec` è solo un'ottimizzazione |
| `tracing-subscriber` | `env-filter` | Abilita la selezione del livello con una variabile di ambiente. Escluso da D8. Trascinerebbe `matchers` |
| `tracing-subscriber` | `json`, `chrono`, `time`, `local-time` | Formato JSON e timestamp non richiesti (D7) |

### 3.2 Impatto misurato

| Misura | Prima | Dopo |
| --- | --- | --- |
| Pacchetti in `Cargo.lock` | 70 | 78 |
| Crate nell'albero normale (`cargo tree -e normal`) | 55 | 63 |
| Dimensione di `target/release/descar` | 2.018.752 byte | 2.272.080 byte |

Crate nuovi: `lazy_static` 1.5.1, `nu-ansi-term` 0.50.3, `pin-project-lite` 0.2.17, `sharded-slab` 0.1.7, `thread_local` 1.1.10, `tracing` 0.1.44, `tracing-core` 0.1.36, `tracing-subscriber` 0.3.23. Le licenze sono MIT oppure MIT OR Apache-2.0, compatibili con la licenza Apache-2.0 del progetto.

### 3.3 Valutazione di sicurezza richiesta dalla costituzione

1. Installa lo strumento: `cargo install cargo-audit --locked`.
2. Esegui `cargo audit` nella radice del repository.
3. Risultato ottenuto con cargo-audit 0.22.2: 1296 advisory caricati, 78 crate analizzati, 0 vulnerabilità, codice di uscita 0.

L'advisory RUSTSEC-2025-0055 (CVE-2025-58160) riguarda `tracing-subscriber`: versioni precedenti alla 0.3.20 scrivono nei log sequenze di escape ANSI provenienti da input non fidato. La versione 0.3.23 è corretta. Non abbassare il vincolo sotto `0.3.20`.

### 3.4 Alternative

| Alternativa | Crate nuovi | Cosa perdi | Giudizio |
| --- | --- | --- | --- |
| Solo libreria standard: macro su `eprintln!` e un livello globale in un `AtomicU8` | 0 | Campi strutturati, span di contesto, controllo `enabled`, compatibilità con l'ecosistema | Coerente con la costituzione. Circa 60 righe da mantenere. Scartata perché i requisiti chiedono struttura e contesto degli eventi |
| `log` più un logger scritto a mano | 1 | Span e campi stabili | Risparmia poco e perde gli span |
| `log` più `env_logger` | `log`, `env_logger` e le loro dipendenze | Span. Porta la variabile `RUST_LOG` come comportamento predefinito | In conflitto con D8 |
| `tracing` più `tracing-appender` | Almeno 1 in più | Nulla | Aggiunge scrittura su file e rotazione, che hai escluso. Non aggiungere |

Giudizio: la scelta di `tracing` costa il 12,5% di binario in più per un progetto che oggi emette una dozzina di eventi. Gli span e i campi la giustificano. Se rinunci a entrambi, la variante con sola libreria standard è più leggera e rispetta meglio la costituzione.

## 4. Implementazione

Applica i passi nell'ordine. I passi 4.3 e 4.4 scrivono test che falliscono, come richiede la costituzione del progetto (sviluppo guidato dai test). I passi 4.5 e 4.6 li fanno passare. Il passo 4.1 applica tutto con un solo comando e sostituisce i passi da 4.2 a 4.6 e 4.8.

| File | Azione | Passo |
| --- | --- | --- |
| `Cargo.toml` (radice) | Modifica: 2 righe | 4.2 |
| `crates/descar/Cargo.toml` | Modifica: 2 righe | 4.2 |
| `Cargo.lock` | Aggiornato da cargo | 4.2 |
| `crates/descar/tests/logging.rs` | Nuovo, 238 righe | 4.3 |
| `crates/descar-cli/tests/cli.rs` | Modifica: aggiunge 3 test | 4.4 |
| `crates/descar-cli/src/cli.rs` | Modifica: aggiunge `Args::logging` | 4.4 |
| `crates/descar/src/logging.rs` | Nuovo, 121 righe | 4.5 |
| `crates/descar/src/main.rs` | Sostituito | 4.6 |
| `README.md`, `CLAUDE.md` | Modifica | 4.8 |

### 4.1 Scorciatoia: applicare la patch

Copia `descar-logging.patch` nella radice del repository, sul ramo `feat/logging` creato nel passo 1.4. Esegui:

```sh
git apply --check descar-logging.patch
git apply descar-logging.patch
rm descar-logging.patch
git status --short
```

Risultato atteso: i primi tre comandi non stampano nulla. `git status --short` elenca 10 percorsi: otto con `  M ` (`CLAUDE.md`, `Cargo.lock`, `Cargo.toml`, `README.md`, `crates/descar-cli/src/cli.rs`, `crates/descar-cli/tests/cli.rs`, `crates/descar/Cargo.toml`, `crates/descar/src/main.rs`) e due con `??` (`crates/descar/src/logging.rs`, `crates/descar/tests/`). Se `HEAD` non è `2af7adf` e `git apply --check` fallisce, usa `git apply --3way`. Dopo l'applicazione passa alla sezione 7.

### 4.2 Aggiungere le dipendenze

Scopo: rendere disponibili `tracing` e `tracing-subscriber` al solo crate `descar`.

File `Cargo.toml` nella radice. Nella tabella `[workspace.dependencies]`, dopo la riga `logos = "0.16.1"`, aggiungi due righe. La tabella deve diventare:

```toml
[workspace.dependencies]
insta = { version = "1.48.0", features = ["yaml", "redactions", "filters"] }
clap = { version = "4.6.6", features = ["cargo", "derive"] }
thiserror = "2.0.20"
console = "0.16.4"
regex = "1.13.1"
logos = "0.16.1"
tracing = { version = "0.1.44", default-features = false, features = ["std"] }
tracing-subscriber = { version = "0.3.23", default-features = false, features = ["fmt", "ansi"] }
```

File `crates/descar/Cargo.toml`. Nella tabella `[dependencies]`, dopo la riga `console = { workspace = true }`, aggiungi due righe. La tabella deve diventare:

```toml
[dependencies]
descar-cli = { path = "../descar-cli" }
descar-core = { path = "../descar-core" }
clap = { workspace = true }
console = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

Non modificare `crates/descar-core/Cargo.toml` e `crates/descar-cli/Cargo.toml`.

Esegui:

```sh
cargo build -p descar
grep -c '^name = ' Cargo.lock
cargo tree -p descar -e normal | grep tracing
```

Risultato atteso: la compilazione termina con `Finished` e nessun errore. Il conteggio di `Cargo.lock` passa da 70 a 78. L'ultimo comando elenca `tracing v0.1.44`, `tracing-core v0.1.36` e `tracing-subscriber v0.3.23`.

### 4.3 Scrivere i test di integrazione (devono fallire)

Scopo: fissare il contratto del logging prima del codice. Ogni test avvia il binario `descar` come processo figlio e legge stdout, stderr e codice di uscita.

Crea la cartella `crates/descar/tests/`. Crea il file `crates/descar/tests/logging.rs` con questo contenuto:

```rust
//! End-to-end tests for the logging contract of the `descar` binary.
//!
//! Contract under test:
//! - log events go to stderr, never to stdout;
//! - stdout carries only functional output (`--version`, help);
//! - `-v`, `-vv`, `-vvv` and `-q` select the maximum level;
//! - `-q` silences log events but not compiler diagnostics or fatal errors;
//! - source text is never copied into log events.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const LEVELS: [&str; 5] = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

fn fixture(name: &str) -> String {
    format!("{}/../../dr_files/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_descar"))
        .args(args)
        .env_remove("NO_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .output()
        .expect("the descar binary should start")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn has_level(text: &str, level: &str) -> bool {
    text.lines().any(|line| line.split_whitespace().next() == Some(level))
}

#[test]
fn default_run_is_silent_on_success() {
    let output = run(&["compile", &fixture("simple_test.dr")]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
}

#[test]
fn single_v_shows_warnings_and_hides_lower_levels() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-O", "basic", "-v"]);
    let err = stderr(&output);

    assert!(output.status.success());
    assert!(has_level(&err, "WARN"), "stderr was: {err}");
    for hidden in ["INFO", "DEBUG", "TRACE"] {
        assert!(!has_level(&err, hidden), "{hidden} must be hidden at -v, stderr was: {err}");
    }
    assert_eq!(stdout(&output), "");
}

#[test]
fn double_v_shows_info_and_debug_and_hides_trace() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-vv"]);
    let err = stderr(&output);

    assert!(output.status.success());
    assert!(has_level(&err, "INFO"), "stderr was: {err}");
    assert!(has_level(&err, "DEBUG"), "stderr was: {err}");
    assert!(!has_level(&err, "TRACE"), "stderr was: {err}");
    assert_eq!(stdout(&output), "");
}

#[test]
fn triple_v_shows_trace_with_source_location() {
    let output = run(&["check", &fixture("simple_test.dr"), "-vvv"]);
    let err = stderr(&output);

    assert!(output.status.success());
    assert!(has_level(&err, "TRACE"), "stderr was: {err}");
    assert!(err.contains("main.rs:"), "trace lines must carry file:line, stderr was: {err}");
    assert_eq!(stdout(&output), "");
}

#[test]
fn more_than_three_v_saturates_at_trace() {
    let output = run(&["check", &fixture("simple_test.dr"), "-vvvvvv"]);

    assert!(output.status.success());
    assert!(has_level(&stderr(&output), "TRACE"));
}

#[test]
fn quiet_silences_log_events_even_with_verbose() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-q", "-vvv"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
}

#[test]
fn quiet_keeps_compiler_diagnostics_and_exit_code() {
    let output = run(&["compile", &fixture("break_outside_loop.dr"), "-q"]);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("E2009"), "stderr was: {err}");
    for level in ["WARN", "INFO", "DEBUG", "TRACE"] {
        assert!(!has_level(&err, level), "no log event may appear with -q, stderr was: {err}");
    }
    assert_eq!(stdout(&output), "");
}

#[test]
fn quiet_keeps_fatal_io_errors() {
    let output = run(&["compile", "this_file_does_not_exist.dr", "-q"]);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("ERROR: I/O"), "stderr was: {err}");
    assert_eq!(stdout(&output), "");
}

#[test]
fn failing_source_never_writes_to_stdout_at_any_level() {
    let path = fixture("break_outside_loop.dr");

    for flags in [&[][..], &["-v"], &["-vv"], &["-vvv"]] {
        let mut args = vec!["compile", path.as_str()];
        args.extend_from_slice(flags);

        let output = run(&args);

        assert_eq!(output.status.code(), Some(1), "flags: {flags:?}");
        assert_eq!(stdout(&output), "", "flags: {flags:?}");
        assert!(stderr(&output).contains("E2009"), "flags: {flags:?}");
    }
}

#[test]
fn debug_trace_records_each_frontend_stage_and_its_failure() {
    let output = run(&["compile", &fixture("break_outside_loop.dr"), "-vv"]);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("lexing finished"), "stderr was: {err}");
    assert!(err.contains("parsing finished"), "stderr was: {err}");
    assert!(err.contains("type checking finished"), "stderr was: {err}");
    assert!(err.contains("frontend stopped"), "stderr was: {err}");
}

#[test]
fn version_goes_to_stdout_only() {
    let output = run(&["--version"]);

    assert!(output.status.success());
    assert!(stdout(&output).contains(env!("CARGO_PKG_VERSION")));
    assert_eq!(stderr(&output), "");
}

#[test]
fn help_without_command_goes_to_stdout_only() {
    let output = run(&[]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("Usage"));
    assert_eq!(stderr(&output), "");
}

#[test]
fn redirected_stderr_has_no_ansi_escape_codes() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-O", "basic", "-vvv"]);
    let err = stderr(&output);

    assert!(!err.is_empty());
    assert!(!err.contains('\u{1b}'), "stderr must be plain text when it is not a terminal: {err:?}");
}

#[test]
fn every_log_line_is_plain_text_starting_with_a_level() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-O", "basic", "-vvv"]);
    let err = stderr(&output);

    assert!(!err.is_empty());
    for line in err.lines() {
        let first = line.split_whitespace().next().unwrap_or_default();
        assert!(LEVELS.contains(&first), "line does not start with a level: {line:?}");
    }
}

#[test]
fn log_events_carry_command_context() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-vv"]);
    let err = stderr(&output);

    assert!(err.contains("compile{"), "events must carry the command span, stderr was: {err}");
    assert!(err.contains("simple_test.dr"), "stderr was: {err}");
}

#[test]
fn source_text_is_never_written_to_logs() {
    let secret = "super-secret-token-12345";
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    fs::create_dir_all(&dir).expect("the target tmp directory should be creatable");
    let path = dir.join("logging_secret_probe.dr");
    fs::write(&path, format!("main {{\n    var token: string = \"{secret}\"\n}}\n"))
        .expect("the probe should be written");

    let output = run(&["compile", path.to_str().expect("UTF-8 path"), "-vvv"]);
    let err = stderr(&output);

    assert!(output.status.success(), "stderr was: {err}");
    assert!(!err.is_empty());
    assert!(!err.contains(secret), "source text leaked into logs: {err}");
    assert_eq!(stdout(&output), "");
}

#[cfg(unix)]
#[test]
fn control_characters_in_paths_are_escaped_in_logs() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    fs::create_dir_all(&dir).expect("the target tmp directory should be creatable");
    let path = dir.join("evil\n INFO forged line \u{1b}[31mred\u{1b}[0m.dr");
    fs::copy(fixture("simple_test.dr"), &path).expect("the probe should be copied");

    let output = run(&["check", path.to_str().expect("UTF-8 path"), "-vv"]);
    let err = stderr(&output);

    assert!(output.status.success(), "stderr was: {err}");
    assert!(!err.is_empty(), "the probe needs log events to inspect");
    assert!(!err.contains('\u{1b}'), "an escape character reached stderr: {err:?}");
    for line in err.lines() {
        let first = line.split_whitespace().next().unwrap_or_default();
        assert!(LEVELS.contains(&first), "a forged or split line reached stderr: {line:?}");
    }
}
```

Esegui:

```sh
cargo test -p descar --test logging
```

Risultato atteso: il comando termina con errore e la riga finale è `test result: FAILED. 5 passed; 12 failed`. I 12 test che falliscono descrivono il comportamento nuovo. I 5 che passano proteggono comportamenti esistenti da non rompere: `quiet_keeps_compiler_diagnostics_and_exit_code`, `quiet_keeps_fatal_io_errors`, `quiet_silences_log_events_even_with_verbose`, `version_goes_to_stdout_only`, `help_without_command_goes_to_stdout_only`. Se un test del primo gruppo passa già, il test non verifica nulla: correggilo prima di proseguire.

### 4.4 Aggiungere l'accessore `Args::logging`

Scopo: `main` deve leggere i `LoggingArgs` senza `match` sul sottocomando. `Args` oggi non offre un accessore.

File `crates/descar-cli/tests/cli.rs`. Aggiungi in fondo al file questi tre test:

```rust
#[test]
fn logging_accessor_returns_none_without_command() {
    let args = Args::try_parse_from(["descar"]).expect("root command should parse");
    assert!(args.logging().is_none());
}

#[test]
fn logging_accessor_returns_arguments_of_compile_command() {
    let args = Args::try_parse_from(["descar", "compile", "program.dr", "-vv"]).expect("compile should parse");
    let logging = args.logging().expect("compile carries logging arguments");
    assert_eq!(logging.verbose, 2);
    assert!(!logging.quiet);
}

#[test]
fn logging_accessor_returns_arguments_of_check_command() {
    let args = Args::try_parse_from(["descar", "check", "program.dr", "-q"]).expect("check should parse");
    let logging = args.logging().expect("check carries logging arguments");
    assert_eq!(logging.verbose, 0);
    assert!(logging.quiet);
}
```

Esegui `cargo test -p descar-cli --test cli`. Risultato atteso: errore di compilazione `E0599: no method named logging found for struct Args`, ripetuto tre volte.

File `crates/descar-cli/src/cli.rs`. Inserisci questo blocco subito prima della riga `/// Top-level Descar CLI commands.`, cioè dopo la struct `Args` e prima dell'enum `Command`:

```rust
impl Args {
    /// Returns the logging arguments of the selected command.
    ///
    /// Returns `None` when no command was selected, because the help text is then the only output.
    #[must_use]
    pub const fn logging(&self) -> Option<&LoggingArgs> {
        match &self.command {
            Some(Command::Compile(args)) => Some(&args.logging),
            Some(Command::Check(args)) => Some(&args.logging),
            None => None,
        }
    }
}
```

Esegui `cargo test -p descar-cli`. Risultato atteso: tutte le suite passano. La suite `cli` riporta `19 passed` (16 test esistenti più 3 nuovi) e la suite `cli_snapshots` riporta `11 passed`. Gli snapshot `insta` non cambiano, perché l'accessore non modifica `--help`.

### 4.5 Creare `logging.rs`

Scopo: un solo modulo contiene la mappatura dei flag e l'installazione del subscriber. Non eseguire ancora i test: il file viene compilato solo dopo che `main.rs` contiene `mod logging;` (passo 4.6).

Crea il file `crates/descar/src/logging.rs` con questo contenuto:

```rust
//! Logging setup for the `descar` binary.
//!
//! Only the binary installs a subscriber. Library crates emit `tracing` events and never
//! configure where the events go.
//!
//! Contract:
//! - log events are plain text lines written to stderr;
//! - stdout is never used for log events;
//! - the flags `-v`, `-vv`, `-vvv` and `-q` select the maximum level (see [`level_filter`]).

use descar_cli::cli::LoggingArgs;
use std::error::Error;
use std::io;
use tracing::level_filters::LevelFilter;

/// Maximum level used when the user passes neither `-v` nor `-q`.
///
/// Assumption to confirm: the requested mapping starts at `-v` = `WARN`, so the level below
/// `WARN` that still reports failures is `ERROR`.
pub const DEFAULT_LEVEL: LevelFilter = LevelFilter::ERROR;

/// Maps the CLI flags to the maximum level of log events.
///
/// | Flags              | Level                 |
/// |--------------------|-----------------------|
/// | `-q` (any `-v`)    | off                   |
/// | none               | [`DEFAULT_LEVEL`]     |
/// | `-v`               | `WARN`                |
/// | `-vv`              | `DEBUG`               |
/// | `-vvv` or more     | `TRACE`               |
///
/// `-q` has precedence over `-v`.
#[must_use]
pub const fn level_filter(verbose: u8, quiet: bool) -> LevelFilter {
    if quiet {
        return LevelFilter::OFF;
    }

    match verbose {
        0 => DEFAULT_LEVEL,
        1 => LevelFilter::WARN,
        2 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    }
}

/// Installs the global subscriber for the selected level.
///
/// When the level is off, no subscriber is installed, so every event has no cost.
/// Colors are used only when stderr is a terminal and the user did not disable colors.
///
/// # Errors
///
/// Returns an error when a global subscriber is already installed.
pub fn init(args: &LoggingArgs) -> Result<(), Box<dyn Error + Send + Sync>> {
    let level = level_filter(args.verbose, args.quiet);
    if level == LevelFilter::OFF {
        return Ok(());
    }

    let with_location = level == LevelFilter::TRACE;

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(io::stderr)
        .with_ansi(console::colors_enabled_stderr())
        .with_target(true)
        .with_file(with_location)
        .with_line_number(with_location)
        .without_time()
        .try_init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_verbosity_to_level() {
        let cases = [
            (0, LevelFilter::ERROR),
            (1, LevelFilter::WARN),
            (2, LevelFilter::DEBUG),
            (3, LevelFilter::TRACE),
            (4, LevelFilter::TRACE),
            (u8::MAX, LevelFilter::TRACE),
        ];

        for (verbose, expected) in cases {
            assert_eq!(level_filter(verbose, false), expected, "verbose = {verbose}");
        }
    }

    #[test]
    fn quiet_turns_logging_off_for_every_verbosity() {
        for verbose in [0, 1, 2, 3, u8::MAX] {
            assert_eq!(level_filter(verbose, true), LevelFilter::OFF, "verbose = {verbose}");
        }
    }

    #[test]
    fn default_level_is_the_level_selected_without_flags() {
        assert_eq!(level_filter(0, false), DEFAULT_LEVEL);
    }

    #[test]
    fn quiet_installs_no_subscriber_and_succeeds_every_time() {
        let args = LoggingArgs { verbose: 3, quiet: true };

        assert!(init(&args).is_ok());
        assert!(init(&args).is_ok());
    }

    #[test]
    fn second_installation_of_a_subscriber_is_reported_as_error() {
        let args = LoggingArgs { verbose: 1, quiet: false };

        assert!(init(&args).is_ok());
        assert!(init(&args).is_err());
    }
}
```

Spiegazione:

| Elemento | Funzione |
| --- | --- |
| `DEFAULT_LEVEL` | Unico punto da cambiare per la decisione D1 |
| `level_filter` | Funzione pura e `const`. Il test la verifica senza installare nulla. `-q` è controllato prima di `-v`. Il ramo `_` accetta tre o più `-v`, quindi `-vvvvvv` equivale a `-vvv` |
| `init`, livello spento | Restituisce `Ok(())` senza installare alcun subscriber. Con nessun subscriber, `tracing` scarta ogni evento senza formattarlo |
| `with_max_level(level)` | Filtro statico per livello. Non usa `EnvFilter`, quindi nessuna variabile di ambiente cambia il livello (D8) |
| `with_writer(io::stderr)` | Destinazione di tutti gli eventi |
| `with_ansi(console::colors_enabled_stderr())` | Colori solo se stderr è un terminale. La funzione `console` rispetta `NO_COLOR`, `TERM=dumb` e `CLICOLOR_FORCE` ed è già la fonte dei colori dei messaggi di errore del binario |
| `with_file`, `with_line_number` | Attivi solo a TRACE |
| `without_time()` | Nessun timestamp (D7) |
| `try_init()` | Restituisce un errore se esiste già un subscriber globale. `init()` senza `try_` andrebbe in panic |

### 4.6 Sostituire `main.rs`

Scopo: dichiarare il modulo, installare il logging, sostituire i `println!` diagnostici con eventi e rendere uniforme il codice dei due comandi.

Sostituisci tutto il contenuto di `crates/descar/src/main.rs` con questo:

```rust
mod logging;

use console::style;
use descar_core::error::compile_error::CompileError;
use descar_core::lex::lexer::{Lexer, lexer_tokenize_with_errors};
use descar_core::semantic::type_checker::TypeChecker;
use std::path::Path;
use std::time::Instant;
use std::{fs, process};

use clap::{CommandFactory, Parser};
use descar_cli::cli::{Args, Command, CompileArgs, OptimizationLevel};
use descar_core::error::error_reporter::ErrorReporter;
use descar_core::syntax::ast::Stmt;
use descar_core::syntax::parser::JsavParser;
use tracing::{debug, error_span, info, trace, warn};

use descar_core::file::{FileSizeInfo, SizeSystems};

/// Reports an I/O error using the CLI error formatting.
///
/// The error category and underlying error value are printed to standard error.
/// This message is a user-facing error, not a log event, so `-q` does not hide it.
fn handle_io_error<T: std::fmt::Display>(error_type: &str, e: T) {
    eprintln!("{} {}: {}\n", style("ERROR:").red().bold(), style(error_type).red(), style(e).yellow());
}

/// Logs the size of the source text at debug level.
///
/// The values are formatted only when the debug level is enabled.
fn log_source_size(bytes: usize) {
    let size = FileSizeInfo::new(u64::try_from(bytes).unwrap_or(u64::MAX));
    debug!(
        bytes = size.bytes,
        si = %size.format(&SizeSystems::SI_SYSTEM),
        iec = %size.format(&SizeSystems::IEC),
        "source file size"
    );
}

/// Reads a source file, reporting an I/O error and exiting if the read fails.
fn read_input(path: &Path) -> String {
    trace!("reading source file");
    let input = fs::read_to_string(path).unwrap_or_else(|e| {
        handle_io_error("I/O", format!("failed to read '{}': {}", path.to_string_lossy(), e));
        process::exit(1);
    });
    log_source_size(input.len());
    input
}
/// Returns the path as UTF-8, reporting an I/O error and exiting if conversion fails.
fn path_to_str(path: &Path) -> &str {
    path.to_str().unwrap_or_else(|| {
        handle_io_error("I/O", format!("invalid file path '{}'", path.to_string_lossy()));
        process::exit(1);
    })
}

/// Prints the diagnostics of a failed stage to stderr and exits with status code 1.
///
/// The diagnostics are the output of the compiler, not log events, so `-q` does not hide them.
fn stop_frontend(stage: &'static str, error_reporter: &ErrorReporter, errors: Vec<CompileError>) -> ! {
    debug!(stage, errors = errors.len(), "frontend stopped");
    eprintln!("{}", error_reporter.report_errors(errors));
    process::exit(1);
}

/// Lexes, parses, and type-checks source input.
///
/// Returns the parsed statements when all stages succeed. If any stage reports
/// diagnostics, prints them and exits with status code 1.
fn run_frontend(file_path: &str, input: &str) -> Vec<Stmt> {
    trace!(source_len = input.len(), "lexing started");
    let started = Instant::now();
    let mut lexer = Lexer::new(file_path, input);
    let (tokens, lexer_errors) = lexer_tokenize_with_errors(&mut lexer);
    debug!(tokens = tokens.len(), errors = lexer_errors.len(), elapsed = ?started.elapsed(), "lexing finished");
    let error_reporter = ErrorReporter::new(lexer.get_line_tracker().clone());

    if !lexer_errors.is_empty() {
        stop_frontend("lexing", &error_reporter, lexer_errors);
    }

    trace!(tokens = tokens.len(), "parsing started");
    let started = Instant::now();
    let (statements, parser_errors) = JsavParser::new(&tokens).parse();
    debug!(statements = statements.len(), errors = parser_errors.len(), elapsed = ?started.elapsed(), "parsing finished");
    if !parser_errors.is_empty() {
        stop_frontend("parsing", &error_reporter, parser_errors);
    }

    trace!(statements = statements.len(), "type checking started");
    let started = Instant::now();
    let mut type_checker = TypeChecker::new();
    let type_checker_errors = type_checker.check(&statements);
    debug!(errors = type_checker_errors.len(), elapsed = ?started.elapsed(), "type checking finished");
    if !type_checker_errors.is_empty() {
        stop_frontend("type checking", &error_reporter, type_checker_errors);
    }

    statements
}

/// Reads a source file and runs the frontend on it.
fn analyze(path: &Path) -> Vec<Stmt> {
    let input = read_input(path);
    let file_path_str = path_to_str(path);
    run_frontend(file_path_str, &input)
}

/// Warns about options that the parser accepts but the compiler does not use yet.
///
/// Remove an entry when the matching option is implemented.
fn warn_ignored_options(args: &CompileArgs) {
    let options = [
        ("--output", args.output.is_some()),
        ("--optimize", args.optimize != OptimizationLevel::None),
        ("--emit-ir", args.emit_ir),
        ("--diagnostics", args.diagnostics),
    ];

    for (option, _) in options.into_iter().filter(|&(_, is_set)| is_set) {
        warn!(option, "option is accepted but has no effect yet");
    }
}

fn main() {
    let args = Args::parse();

    if let Some(Err(error)) = args.logging().map(logging::init) {
        eprintln!("{} logging is disabled: {error}", style("WARNING:").yellow().bold());
    }

    match args.command {
        None => Args::command().print_help().expect("failed to print help"),
        Some(Command::Compile(args)) => {
            let _command = error_span!("compile", file = ?args.input).entered();
            debug!(optimize = ?args.optimize, emit_ir = args.emit_ir, diagnostics = args.diagnostics, output = ?args.output, "command started");
            warn_ignored_options(&args);

            let _statements = analyze(args.input.as_path());
            info!("compilation succeeded");
        }
        Some(Command::Check(args)) => {
            let _command = error_span!("check", file = ?args.input).entered();
            debug!("command started");

            let _statements = analyze(args.input.as_path());
            info!("check succeeded");
        }
    }
}
```

Spiegazione:

| Elemento | Funzione |
| --- | --- |
| `mod logging;` | Compila il modulo del passo 4.5 |
| `handle_io_error` | Invariata. Il commento dichiara che è un messaggio per l'utente e non un evento, quindi `-q` non lo nasconde (D4) |
| `log_source_size` | Sostituisce `print_file_size_report`. Usa `input.len()` e non più `fs::metadata`. I campi `si` e `iec` vengono calcolati solo se DEBUG è abilitato, perché le macro di `tracing` valutano i campi solo per eventi attivi |
| `read_input` | Aggiunge un evento TRACE prima della lettura e registra la dimensione dopo |
| `stop_frontend` | Funzione con tipo `!` che sostituisce tre blocchi identici. Emette l'evento DEBUG prima di stampare la diagnostica e terminare |
| `run_frontend` | Misura ogni fase con `Instant`. Registra solo conteggi e durate, mai token o testo |
| `analyze` | Elimina la duplicazione tra `compile` e `check`. L'ordine originale resta: prima la lettura del file, poi la conversione UTF-8 del percorso |
| `warn_ignored_options` | Un WARN per ogni opzione accettata e non usata (D6). Elimina la voce quando implementi l'opzione |
| `error_span!` | Crea lo span `compile` o `check` con il campo `file` formattato con `?` |
| `main` | Installa il logging subito dopo il parsing degli argomenti. Un errore di installazione stampa un avviso e non ferma il comando |

Comportamenti cambiati rispetto al codice originale:

1. `Compiling ...`, la tabella delle dimensioni e `Compilation successful` non escono più su stdout.
2. Senza flag il comando termina in silenzio se ha successo (D3).
3. `compile -v` non stampa più la tabella delle dimensioni. Con `-v` compaiono solo i WARN. Le dimensioni sono visibili da `-vv`.
4. Un errore di `fs::metadata` non può più verificarsi, perché la chiamata è stata rimossa.

### 4.7 Eseguire test, formattazione e analisi statica

```sh
cargo fmt --all
cargo test -p descar -p descar-cli
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Risultato atteso:

| Comando | Risultato |
| --- | --- |
| `cargo fmt --all` | Nessun output. Se cambia una riga, accetta la modifica |
| `cargo test -p descar -p descar-cli` | `5 passed` per i test unitari `logging::tests`, `17 passed` per `logging`, `19 passed` per `cli`, `11 passed` per `cli_snapshots`. Nessun test fallito |
| `cargo clippy ... -D warnings` | Termina con `Finished` e nessun avviso |
| `cargo fmt --all -- --check` | Nessun output, codice di uscita 0 |

Se un test del passo 4.3 fallisce ancora, leggi il messaggio `stderr was:` del test: riporta l'output reale del binario.

### 4.8 Aggiornare la documentazione

Scopo: la regola sui flussi deve essere scritta dove la leggono i prossimi sviluppatori e gli agenti di codice.

File `README.md`, due modifiche.

Modifica 1. Nella sezione `## Main features` sostituisci la riga della verbosità con:

```markdown
- Verbosity control (`-v`, `-vv`, `-vvv`) and silent mode (`-q`), see [Logging](#logging)
```

Modifica 2. Inserisci questa sezione subito prima di `## Practical examples`:

````markdown
## Logging

Descar writes log events as plain text lines to **stderr**. **stdout** carries only functional output
(`--version`, `--help`). Log events never go to stdout.

| Flags | Maximum level shown |
|---|---|
| none | `ERROR` |
| `-v` | `WARN` |
| `-vv` | `DEBUG` (includes `INFO`) |
| `-vvv` or more | `TRACE` (adds `file:line`) |
| `-q` | no log events (wins over `-v`) |

`-q` silences log events only. Compiler diagnostics (syntax and type errors) and fatal I/O errors are
still written to stderr, and the exit code is not changed.

Example:

```sh
descar compile dr_files/simple_test.dr -vv 2> descar.log
```

Colors are used only when stderr is a terminal and `NO_COLOR` is not set. Log retention and rotation are
the responsibility of the system that starts `descar`.

Log events never contain source text. Library crates only emit `tracing` events. Only the `descar` binary
installs the subscriber (`crates/descar/src/logging.rs`).
````

File `CLAUDE.md`, tre modifiche.

Modifica 1. Sostituisci la riga `- Verbosity: add ...` con:

```markdown
- Verbosity: add `-v` (repeat) or `-q` to quiet. Levels: none=`ERROR`, `-v`=`WARN`, `-vv`=`DEBUG`, `-vvv`=`TRACE`, `-q`=off
```

Modifica 2. Sostituisci la riga del crate `descar` nell'elenco dell'architettura con:

```markdown
- `descar`: binary entry point, wires CLI to `main`, uses `descar_cli::cli`. Owns logging setup (`src/logging.rs`, `tracing` + `tracing-subscriber`).
```

Modifica 3. Dopo il paragrafo che inizia con `Core crate pure Rust`, inserisci:

```markdown
## Logging rules

- Log events go to stderr. stdout is only for functional output (`--version`, `--help`, future IR output).
- Only the binary crate installs a subscriber. Library crates may emit `tracing` events and must not install one.
- Event messages are static text. Variable data goes in fields (`tokens = n`), never in the message.
- Never log source text, token text, environment variable values or credentials. Log counts, sizes and spans.
- Format paths and other user input with `?` (Debug) so control characters are escaped. Do not use `%` for them.
- Compiler diagnostics and fatal errors are not log events. They are printed with `eprintln!` and ignore `-q`.
```

### 4.9 Verifica finale e commit

```sh
cargo test --workspace --all-features
git status --short
```

Risultato atteso: 39 suite, 855 test passati, 0 falliti (prima della modifica: 38 suite, 830 test). `git status --short` mostra gli stessi 10 percorsi del passo 4.1.

Commit suggeriti, uno per scopo, secondo le convenzioni del repository:

```sh
git add Cargo.toml Cargo.lock crates/descar/Cargo.toml
git commit -m "build: add tracing and tracing-subscriber to the descar crate"

git add crates/descar-cli
git commit -m "feat(cli): add Args::logging accessor"

git add crates/descar README.md CLAUDE.md
git commit -m "feat(cli): send diagnostics to stderr through tracing with -v, -vv, -vvv and -q"
```

Il secondo comando `git add` non include `crates/descar/Cargo.toml`, già incluso nel primo commit. Il terzo comando include i file rimanenti di `crates/descar`: `src/logging.rs`, `src/main.rs` e `tests/logging.rs`.
