#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Dependency-free checks for this repository's documented contracts.

This is a lightweight working-tree check, not a secret scanner, full Markdown
parser, remote-link checker, or Git history audit. It also works in source
archives without Git. Never print a suspected secret's value.
"""

import argparse
from collections import Counter
import json
from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import unquote, urlsplit


IGNORED = {".git", ".local", ".paja", ".wrangler", "node_modules", "dist",
           "coverage", "output", "target", "__pycache__", ".pytest_cache"}
SENSITIVE = [
    ("private machine path", re.compile(r"(?:/Users|/home)/[A-Za-z0-9_.-]+/")),
    ("private Windows path", re.compile(r"[A-Za-z]:\\Users\\[A-Za-z0-9_.-]+\\")),
    ("private key", re.compile(r"-----BEGIN (?:[A-Z]+ )?PRIVATE KEY-----")),
    ("provider token", re.compile(r"\bsk-(?:proj-|svcacct-)?[A-Za-z0-9_-]{24,}")),
    ("GitHub token", re.compile(r"\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,})")),
    ("cloud access key", re.compile(r"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b")),
]


def repository_files(root):
    if (root / ".git").exists():
        result = subprocess.run(
            ["git", "-C", str(root), "ls-files", "--cached", "--others",
             "--exclude-standard", "-z"],
            check=True, capture_output=True,
        )
        return sorted({root / name.decode("utf-8") for name in result.stdout.split(b"\0") if name})
    return sorted(path for path in root.rglob("*")
                  if not any(part in IGNORED for part in path.relative_to(root).parts)
                  and (path.is_file() or path.is_symlink()))


def prose(text):
    """Remove fenced code while keeping line numbers for local link diagnostics."""
    lines, fence = [], None
    for line in text.splitlines():
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker:
            token = marker.group(1)
            if fence is None:
                fence = token
            elif token[0] == fence[0] and len(token) >= len(fence):
                fence = None
            lines.append("")
        else:
            lines.append(line if fence is None else "")
    return "\n".join(lines)


def anchors(text):
    result, seen = set(), Counter()
    for title in re.findall(r"^#{1,6}\s+(.+?)(?:\s+#+)?$", prose(text), re.MULTILINE):
        title = re.sub(r"\[([^]]+)\]\([^)]+\)", r"\1", title)
        title = re.sub(r"<[^>]+>", "", title).lower().replace("`", "")
        slug = re.sub(r"[^\w\- ]", "", title).replace(" ", "-")
        count = seen[slug]
        seen[slug] += 1
        result.add(f"{slug}-{count}" if count else slug)
    result.update(re.findall(r'<a\s+(?:name|id)=["\']([^"\']+)["\']', text))
    return result


def check(root):
    errors, texts = [], {}

    def fail(path, message, line=None):
        location = str(path.relative_to(root))
        errors.append(f"{location}{':' + str(line) if line else ''}: {message}")

    for path in repository_files(root):
        if path.is_symlink():
            fail(path, "symlink requires explicit publication review")
            continue
        if not path.is_file():
            continue
        if not path.resolve().is_relative_to(root):
            fail(path, "file resolves outside repository")
            continue
        name = path.name
        example = name.endswith(".example")
        if ((name == ".env" or name.startswith((".env.", ".dev.vars"))) and not example
                or name in {"secrets.md", ".secrets"}
                or path.suffix in {".pem", ".key"}):
            fail(path, "private configuration must not enter public source")
        try:
            value = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        texts[path] = value
        if path.suffix == ".json":
            try:
                json.loads(value)
            except json.JSONDecodeError as error:
                fail(path, "invalid JSON syntax", error.lineno)
        for label, pattern in SENSITIVE:
            for match in pattern.finditer(value):
                fail(path, f"possible {label}; inspect locally", value.count("\n", 0, match.start()) + 1)

    documents = {path: value for path, value in texts.items() if path.suffix == ".md"}
    for path, value in documents.items():
        body = prose(value)
        # Inline links and reference definitions used by the repository.
        links = list(re.finditer(r"\[[^]\n]+\]\((<[^>]+>|[^\s)]+)(?:\s+\"[^\"]*\")?\)", body))
        links += list(re.finditer(r"^\s*\[[^]]+\]:\s*(<[^>]+>|\S+)", body, re.MULTILINE))
        for match in links:
            href = match.group(1).strip("<>")
            line = body.count("\n", 0, match.start()) + 1
            url = urlsplit(href)
            if url.scheme in {"https", "http", "mailto"}:
                continue
            if url.scheme or url.netloc or url.path.startswith("/"):
                fail(path, "use a repository-relative local link or an explicit web URL", line)
                continue
            target = (path.parent / unquote(url.path)).resolve() if url.path else path
            if not target.is_relative_to(root):
                fail(path, "local link escapes repository", line)
            elif not target.exists():
                fail(path, f"missing local link target: {href}", line)
            elif url.fragment and target.suffix == ".md":
                if unquote(url.fragment) not in anchors(target.read_text(encoding="utf-8")):
                    fail(path, f"missing heading anchor: {href}", line)

    required = ["README.md", "LICENSE", "NOTICE", "CONTRIBUTING.md", "GOVERNANCE.md",
                "CODE_OF_CONDUCT.md", "SECURITY.md", "docs/PRODUCT.md", "docs/ROADMAP.md",
                "docs/CLI.md", "docs/DECISIONS.md", "docs/ARCHITECTURE.md",
                "docs/AGENT_PROTOCOL.md",
                ".github/workflows/repository.yml"]
    for name in required:
        if root / name not in texts:
            errors.append(f"{name}: required repository file missing")

    commands = set(re.findall(r"`paja ([a-z][a-z-]*)\b", texts.get(root / "docs/CLI.md", "")))
    if not commands:
        errors.append("docs/CLI.md: no documented commands")
    for path, value in documents.items():
        for command in set(re.findall(r"\bpaja ([a-z][a-z-]*)\b", value)) - commands:
            fail(path, f"command absent from docs/CLI.md: paja {command}")

    if errors:
        print("Repository checks failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print(f"Repository checks passed: {len(documents)} Markdown files, "
          f"{len(commands)} documented command names, "
          f"{sum(path.suffix == '.json' for path in texts)} JSON files parsed.")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        return check(args.root.resolve())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Repository checks could not complete ({type(error).__name__}).", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
