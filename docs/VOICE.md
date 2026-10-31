# Native Mac voice prompting and the browser companion

Paja Voice turns a short English or Finnish recording into an editable prompt. Record, stop, review, then explicitly create a project or run the prompt. Build, test, improve, status, and results call the existing Rust coordinator. The target remains the deterministic meal-helper control fixture: arbitrary spoken goals are stored as briefs and do not yet generate new AI agents. Voice recognition and target-agent evaluation are separate capabilities.

## Native Mac app

Build on macOS 15 or later with Xcode (including its command-line tools) and the repository's pinned Rust toolchain:

```sh
sh scripts/build-macos.sh
```

Open `output/Paja Voice.app` in Finder, or copy it to Applications. The app contains the Rust CLI and needs no developer tools at runtime. This locally signed development bundle is not a notarized public release. The build uses the exact Swift package version and checked-in `Package.resolved`; dependency downloads happen on the first build.

Typing works immediately. **Download model** installs the multilingual WhisperKit model once; allow roughly 1 GB plus compilation cache space. **Record a prompt** asks for microphone permission only when pressed. Select English or Suomi, speak for up to 30 seconds, then Stop. Cancel and failure preserve the previous draft. Review and edit the transcript before creating a project or running it.

Use **Create project from prompt**, then **Build → Test → Improve → Results**. **Use lunch example → Run prompt** invokes the fixture's supported phrase. The original 11-euro result becomes 8 euros after the deterministic repair. These are control results, not evidence of AI-agent quality. The fixture's task grammar remains the one in [CLI](CLI.md); arbitrary Finnish transcripts are valid briefs but are not arbitrary executable tasks.

Project directories, the project list, and model assets live under the user's Application Support directory in `PajaVoice`. Audio uses a private temporary directory and is removed after transcription or recording cancellation. Cancellation fences a late transcript; inference already running may finish before temporary audio is removed. Closing the app terminates its recording process. No system audio or meeting capture is implemented.

## Browser companion

The browser app, Python server, server transcription, Docker setup, and browser tests now live in the separate private [Paja Web repository](https://github.com/RHolappa/paja-web). Repository access is required to view its source and self-hosting instructions. The hosted demo link and testing steps remain in [README](../README.md).

Paja Web consumes a pinned revision of this public CLI through its versioned JSON commands. It does not access the coordinator database directly. The native Mac app and the Rust coordinator remain here. Mac and browser workspaces are independent; no automatic synchronization is implemented.

## Native development checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
swift test --package-path macos/PajaVoice --disable-automatic-resolution
python3 scripts/check_repository.py
```

The optional native diagnostic commands `PajaVoice --prepare-model ASSETS tiny` and `PajaVoice --transcribe-file ASSETS AUDIO en` exercise local model setup and file transcription without requesting microphone permission. Use `fi` for Finnish and `large-v3-v20240930_626MB` to provision the app's default model. See [Validation](VALIDATION.md) for executed checks and their limits. Browser checks run in Paja Web.

## Privacy and scope

- Native speech is processed on the Mac. Model setup contacts public model hosts; downloaded dependencies and assets retain their respective licenses.
- Submitted prompts become private project metadata or execution records. Native recordings use private temporary storage and are removed after processing or cancellation. A late cancelled transcript cannot replace the draft.
- Disconnecting is not cancellation of coordinator work. Retries reuse their request IDs and exact reviewed input. Evidence stays attached to its original version.
- The Rust coordinator still runs only the fixed fixture. Voice input does not implement real target-agent generation, worker collaboration, or optional target publication.
- The browser companion has its own service and data-handling documentation in Paja Web. Its recordings are processed by its operator's server.

## Inspiration and provenance

[Biscotti](https://github.com/scosman/Biscotti) inspired the native, private speech interaction. Its [PolyForm Perimeter license](https://github.com/scosman/Biscotti/blob/main/LICENSE.md) is separate; no Biscotti code or assets were imported. The native integration is original Paja code using [WhisperKit](https://github.com/argmaxinc/argmax-oss-swift) as a pinned MIT-licensed dependency. The app bundle includes its license. The separately maintained browser companion uses faster-whisper on its operator’s server. No private reference source, configuration, identity, or assets were copied. Paja's Apache-2.0 license and existing notices remain intact.
