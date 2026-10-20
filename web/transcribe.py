#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Explicit model provisioning, or bounded offline transcription on the VPS."""
import argparse
import json
import os
from pathlib import Path
import sys
import wave


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--download", action="store_true")
    parser.add_argument("--audio")
    parser.add_argument("--language", choices=["en", "fi"], default="en")
    args = parser.parse_args()
    model_name = os.environ.get("PAJA_SPEECH_MODEL", "small")
    if model_name not in {"tiny", "base", "small"}:
        raise ValueError("Unsupported speech model")
    root = Path(os.environ.get("PAJA_DATA", ".local/web-data")) / "models" / model_name
    from faster_whisper import WhisperModel, download_model
    if args.download:
        download_model(model_name, output_dir=str(root))
        print("Speech model downloaded; no audio was processed.")
        return
    if not args.audio or not (root / "model.bin").is_file():
        raise ValueError("Speech model missing. Run the deployment setup first.")
    model = WhisperModel(str(root), device="cpu", compute_type="int8", cpu_threads=2,
                         num_workers=1, local_files_only=True)
    # Browser audio is already normalized PCM. Decode it directly rather than
    # accepting arbitrary media containers or depending on PyAV's evolving API.
    import numpy as np
    with wave.open(args.audio, 'rb') as audio:
        if (audio.getnchannels(), audio.getsampwidth(), audio.getframerate()) != (1, 2, 16000) or not 1 <= audio.getnframes() <= 480000:
            raise ValueError('Unsupported audio')
        samples = np.frombuffer(audio.readframes(480000), dtype='<i2').astype(np.float32) / 32768.0
    segments, _ = model.transcribe(samples, language=args.language, task="transcribe",
                                   beam_size=1, vad_filter=True,
                                   condition_on_previous_text=False)
    text = " ".join(segment.text.strip() for segment in segments).strip()
    if not text or len(text.encode()) > 4096:
        raise ValueError("No usable speech detected; try a shorter recording.")
    print(json.dumps({"schema_version": "paja.transcript.v1", "text": text,
                      "language": args.language, "backend": "faster-whisper-vps"}))


if __name__ == "__main__":
    try:
        main()
    except Exception:
        # Do not print model traces, audio paths, or user input into container logs.
        print("Local transcription failed. Check model setup and try a shorter recording.", file=sys.stderr)
        sys.exit(1)
