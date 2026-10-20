#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Measure local process startup and the fixture loop; never calls a provider."""
import argparse
import json
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/release/paja"))
    parser.add_argument("--iterations", type=int, default=30)
    args = parser.parse_args()
    if not 1 <= args.iterations <= 1000:
        parser.error("iterations must be between 1 and 1000")
    binary = args.binary.resolve(strict=True)

    def call(root, *arguments):
        result = subprocess.run([str(binary), "-C", str(root), *arguments],
                                capture_output=True, text=True, check=True)
        return json.loads(result.stdout) if "--json" in arguments else None

    loops, starts, tests = [], [], []
    with tempfile.TemporaryDirectory(prefix="paja-bench-") as directory:
        root = Path(directory)
        # A warm-up avoids measuring the initial executable page faults as steady state.
        subprocess.run([str(binary), "--version"], capture_output=True, check=True)
        for index in range(args.iterations):
            start = time.perf_counter_ns()
            project = call(root, "init", f"meal-{index}", "--goal", "Synthetic meal control",
                           "--request-id", "init", "--json")["project"]["id"]
            work = root / f"meal-{index}"

            def request(*arguments):
                return call(work, "--json", "--project", project, *arguments)

            request("build", "--revision", "0", "--request-id", "build")
            before = time.perf_counter_ns()
            baseline = request("test", "--revision", "1", "--request-id", "test")
            tests.append((time.perf_counter_ns() - before) / 1_000_000)
            improved = request("improve", "--revision", "1", "--request-id", "improve",
                               "--from", baseline["run"]["id"])
            invocation = request("run", "--revision", "2", "--request-id", "run",
                                 "--input", "Choose two lunches for a total of eight euros")
            assert baseline["data"]["report"]["counts"]["failed"] == 2
            assert improved["data"]["adopted"] is True
            assert improved["data"]["report"]["counts"]["passed"] == 4
            assert invocation["data"]["execution"]["output"]["total_cents"] == 800
            loops.append((time.perf_counter_ns() - start) / 1_000_000)
            before = time.perf_counter_ns()
            request("status")
            starts.append((time.perf_counter_ns() - before) / 1_000_000)

    def summary(samples):
        ordered = sorted(samples)
        return {"median_ms": round(statistics.median(samples), 3),
                "p95_ms": round(ordered[min(len(ordered) - 1, (95 * len(ordered) + 99) // 100 - 1)], 3)}

    print(json.dumps({"schema_version": "paja.benchmark.v1", "evidence": "control_fixture",
                      "platform": platform.system(), "architecture": platform.machine(),
                      "iterations": args.iterations, "binary_bytes": binary.stat().st_size,
                      "five_command_loop": summary(loops), "status_process": summary(starts),
                      "four_trial_test_process": summary(tests),
                      "limitations": "Local warm process measurements including SQLite persistence; not model latency, parallel-worker capacity, or a cross-platform guarantee."}, indent=2))


if __name__ == "__main__":
    main()
