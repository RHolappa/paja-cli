# Local validation

## Voice companions — 2026-10-31

The Mac and browser companions use the existing Rust fixture coordinator. Executed locally on macOS/ARM64, with the server speech path also exercised in its Linux/ARM64 Docker image:

| Check | Actual result |
| --- | --- |
| Rust formatting, strict Clippy, and locked tests | Passed; 19 tests including 18 CLI integration tests |
| `python3 -m unittest web.test_server -v` | Five passed: authentication/origin handling, real coordinator loop and replay, restart persistence, bounded input/path checks, WAV validation, and private routes |
| `swift test --package-path macos/PajaVoice --disable-automatic-resolution` | Two passed: literal private prompt handoff, silence and overlong-audio rejection before loading a model |
| `sh scripts/build-macos.sh` | Produced a locally signed app containing the CLI and pinned WhisperKit integration |
| `node web/check-browser.mjs` with installed Chromium | Passed: real fixture loop, login, real fake-device recording/conversion, Finnish language routing, successful synthetic transcript, cancellation/failure draft preservation, released tracks, desktop/mobile layout |
| Docker image build | Passed with Python 3.12, faster-whisper 1.2.1 and the recorded dependency resolution |
| Container HTTP smoke check | Passed real Gunicorn login and Rust init/build/test/improve/run with a read-only root filesystem and no external network |

Short synthetic recordings were generated with the installed macOS Samantha and Satu voices. WhisperKit's configured `large-v3-v20240930_626MB` model returned “Choose two lunches for a total of 8 euros.” and “Rakenna-agentti, joka auttaa valitsemaan edullisen lounaan.” The server's `small` model returned the same English text and “Rakennanagentti joka auttaa valitsemaan edullisen lounaan.” Server inference ran with Docker networking disabled. The English digit substitution is now an explicitly supported fixture phrase; Finnish transcription needs review. These two clips establish functional model integration, not broad recognition accuracy or agent quality.

The initial server live check exposed a PyAV compatibility error. The final path decodes the already validated PCM WAV directly into samples before inference and passed both offline checks. Native Core ML required execution outside the development sandbox to access its acceleration services. No human microphone recording, paid speech API, target-agent model, or production quality evaluation was used. Actual native microphone permission behavior and Safari/iOS microphone behavior still require a human/device check. Browser ASR responses in the browser suite are synthetic and remain distinguishable from the separate real-model file checks.

The historical fixture-only measurements below predate these companions. They do not measure speech latency or cloud capacity.

Executed on 2026-10-25 on macOS/ARM64 with rustc 1.99.1, Cargo 1.99.1, edition 2024, and the committed dependency resolution in Cargo.lock. These results concern deterministic control fixtures only. No live model, paid provider, worker session, hosted CI, or publication was exercised.

## Checks executed

| Check | Actual result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --locked --offline --all-targets -- -D warnings` | Passed without warnings |
| `cargo test --locked --offline` | 18 passed: 17 CLI integration tests and one elapsed-budget unit test |
| `cargo build --release --locked --offline` | Passed; optimized executable produced |
| `git diff --check` | Passed |
| `python3 scripts/check_repository.py` | Passed: 20 Markdown files, 13 documented command names, 3 JSON examples |

The CLI integration tests execute separate processes with isolated synthetic projects. They verify the original failing behavior, successful fixture repair, baseline retention, actual tool traces, source preservation, no-op improvement, request replay after revision changes, idempotency conflicts, historical evidence, frozen-suite tampering resistance, rejected regressions, budget exhaustion, denied tool requests, comparability across suites/settings, non-mutating dry runs, explicit machine input, strict input schemas, storage corruption detection, coordinator locking, duplicate process races, transaction rollback, init retries, and the human walkthrough.

The elapsed-budget unit test proves that an expired deadline prevents any tool dispatch. SQLite connection-loss rollback is tested; OS kill, machine power loss, and partial-init recovery are not claimed as tested. Each CLI call starts a fresh process, so retained versions, evidence, and receipts are read across process restarts.

The [repository check](../scripts/check_repository.py) checks documentation links, documented command names, JSON syntax, and obvious privacy mistakes. It does not establish runtime security or audit Git history. Rust integration tests exercise the actual fixture schema parser and runtime. The configured GitHub workflow has not been run remotely.

## Release-build measurements

Command: `python3 scripts/benchmark_cli.py --binary target/release/paja --iterations 30`.

| Operation | Median | p95 |
| --- | ---: | ---: |
| Init → build → test → improve → run (five processes) | 25.394 ms | 58.895 ms |
| Status process, including evidence read and JSON output | 3.592 ms | 7.873 ms |
| Four-trial test process, including evidence persistence | 4.252 ms | 13.544 ms |

The executable was 3,350,848 bytes (about 3.2 MiB). The release profile uses thin LTO, one code-generation unit, and stripped debug information. The implementation uses standard-library file locks, bounded fixture collections, integer money arithmetic, and synchronous SQLite transactions. There is no async runtime or background worker pool in this slice.

The benchmark performs one executable warm-up, then creates 30 independent temporary projects. Each iteration checks the known failing baseline, the passing repaired candidate, and the 800-cent manual result. Timings include Python process launch, CLI parsing, JSON output, and SQLite persistence on this machine. They are not isolated CPU microbenchmarks, cold-start guarantees, live-model latency, or evidence of parallel-worker capacity. Rerun the script on the intended environment before making performance claims.

## Remaining acceptance work

Milestone 1's owner-operated local example works, but the full milestone and release gate remain incomplete. Separately authenticated build/test/review workers, immutable worker inputs, IPC, fenced leases, scoped task completion, durable background jobs, disconnect recovery, cancellation, and their concurrency/failure tests still need implementation.

The current runtime is fixed fixture logic, not a sandbox for arbitrary target code. Real-model build/runtime integration, provider usage accounting, external-effect reconciliation, export, and optional release authorization are not implemented. Linux/macOS CI is configured; only local macOS validation is recorded here. No agent-quality or user-adoption claim follows from these results.
