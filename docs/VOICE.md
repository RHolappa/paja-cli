# Voice prompting on Mac and in the browser

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

## Browser on a Docker host

Use a Linux Docker host with Docker Compose v2, Python 3, at least 2 CPU cores, 4 GB RAM, and enough disk for builds, models, and retained project history. Start with 40 GB. Point a hostname's DNS A record to the host and allow inbound TCP 80/443. Only the HTTPS proxy has published ports.

From this source checkout on the host:

```sh
sh scripts/deploy-vps.sh paja.example.com
```

The script creates a private `deploy/.env` if absent, builds the image, downloads the default multilingual `small` speech model, and starts the app and Caddy. Caddy obtains and renews the HTTPS certificate. The owner access key is `PAJA_PASSWORD` in `deploy/.env`; it is not printed in build logs. Keep that file private and share the key only with the owner. Visitors need only the HTTPS link, key, and a browser with microphone support. Microphones require HTTPS or localhost.

Updates use the same script and preserve configuration and named volumes. `paja-voice_paja_data` contains projects, immutable evidence, and models. Caddy has separate certificate/configuration volumes. Back up the data volume while the app is stopped to obtain a consistent SQLite copy. Docker Compose `down` preserves volumes; do not use `down -v` unless deleting all retained projects is intended.

```sh
docker compose --env-file deploy/.env -f deploy/compose.yaml stop app
# Back up the paja-voice_paja_data volume with your host's backup tooling.
docker compose --env-file deploy/.env -f deploy/compose.yaml start app
```

The operator may select `tiny`, `base`, or `small` with `PAJA_SPEECH_MODEL`, then rerun setup. Smaller models use fewer resources but need independent accuracy checks. Ordinary transcription sets offline mode and uses provisioned model files. No GPU or paid inference account is needed. CPU throughput depends on the host and is intentionally limited to one recording at a time, three requests per minute, and a 90-second transcription deadline.

The browser keeps the editable draft in local storage. Audio is bounded and converted to mono 16 kHz PCM WAV. Permission denial, Cancel, Escape, hiding the page, and transcription failure leave the draft intact and release microphone tracks. Cancelling after upload discards the result; server work can continue until completion or its deadline. Audio is in a private temporary directory, backed by container tmpfs, and removed when processing ends. The browser never interprets speech as a shell command or auto-submits a completed transcript.

## Local browser development and checks

No frontend build or node runtime is needed to serve the app. For the fixture loop only:

```sh
cargo build --locked
# Supply a private random key of at least 24 characters through the environment.
PAJA_PASSWORD="$YOUR_LOCAL_KEY" python3 web/server.py
```

Open localhost port 8787. The microphone stays disabled until model assets exist. For server speech, use the Docker image or install [the tested web requirements](../web/requirements.lock.txt) in an isolated Python 3.12 environment and run `python3 web/transcribe.py --download`. That explicit step downloads models; ordinary requests do not. Docker uses the tested transitive Python version lock. Model hosting revisions and container image digests are not yet fully locked.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 -m unittest web.test_server -v
swift test --package-path macos/PajaVoice --disable-automatic-resolution
npm ci --prefix web
# Use an installed Playwright Chromium, or provide CHROMIUM_PATH.
node web/check-browser.mjs
python3 scripts/check_repository.py
```

Browser tests exercise real recording plumbing with Chrome's fake microphone and a synthetic transcription response. They test the actual Rust loop through the Python service, but do not prove speech recognition. The optional native diagnostic commands `PajaVoice --prepare-model ASSETS tiny` and `PajaVoice --transcribe-file ASSETS AUDIO en` exercise local model setup and file transcription without requesting microphone permission. Use `fi` for Finnish and `large-v3-v20240930_626MB` to provision the app's default model. See [Validation](VALIDATION.md) for executed checks and their limits.

## Privacy and scope

- Native speech is processed on the Mac. Browser speech reaches the operator's VPS over HTTPS. Neither path sends audio to a third-party speech API.
- Model setup contacts public model hosts; the downloaded software and model assets retain their respective licenses. The operator pays for hosting, storage, and network use.
- The web deployment is one shared owner workspace. The access key grants access to all its projects. Sessions expire after eight hours and are cleared on service restart. There are no individual users, worker credentials, role separation, or automatic Mac/cloud synchronization.
- Prompts explicitly submitted to init/run become private project metadata or execution records. Audio/transcripts are not logged. Browser drafts and pending request receipts remain in that browser's local storage; Lock workspace does not erase them.
- A network disconnect is not cancellation of coordinator work. Retrying a saved request reuses its request ID and exact reviewed input. Historical evidence remains tied to its original version.
- The Rust coordinator still runs only the safe fixed fixture. This companion deployment does not implement real target-agent generation, arbitrary code execution, worker collaboration, or optional target publication.

## Inspiration and provenance

[Biscotti](https://github.com/scosman/Biscotti) inspired the native, private speech interaction. Its [PolyForm Perimeter license](https://github.com/scosman/Biscotti/blob/main/LICENSE.md) is separate; no Biscotti code or assets were imported. The native integration is original Paja code using [WhisperKit](https://github.com/argmaxinc/argmax-oss-swift) as a pinned MIT-licensed dependency. The app bundle includes its license. The server uses [faster-whisper](https://github.com/SYSTRAN/faster-whisper), also MIT-licensed. No private reference source, configuration, identity, or assets were copied. Paja's Apache-2.0 license and existing notices remain intact.
