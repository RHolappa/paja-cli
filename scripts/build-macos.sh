#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
set -eu
cd "$(dirname "$0")/.."
if [ "$(uname -s)" != Darwin ]; then
  echo 'The native app must be built on macOS 15 or later with Xcode installed.' >&2
  exit 1
fi
cargo build --release --locked
swift build --package-path macos/PajaVoice -c release --disable-automatic-resolution
app='output/Paja Voice.app'
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp macos/PajaVoice/.build/release/PajaVoice "$app/Contents/MacOS/PajaVoice"
cp target/release/paja "$app/Contents/Resources/paja"
cp macos/PajaVoice/Info.plist "$app/Contents/Info.plist"
cp LICENSE NOTICE "$app/Contents/Resources/"
cp -f macos/PajaVoice/.build/checkouts/argmax-oss-swift/LICENSE "$app/Contents/Resources/WhisperKit-LICENSE"
cp -f macos/PajaVoice/.build/checkouts/argmax-oss-swift/NOTICES "$app/Contents/Resources/WhisperKit-NOTICES"
codesign --force --sign - "$app/Contents/Resources/paja"
codesign --force --sign - "$app"
printf 'Built %s. Open it in Finder; typing works before the optional model download.\n' "$app"
