#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Single-owner web client for the real Rust coordinator. No direct database access."""
from collections import defaultdict, deque
import hashlib
import hmac
from http.cookies import SimpleCookie
import io
import json
import os
from pathlib import Path
import re
import secrets
import subprocess
import sys
import tempfile
import threading
import time
from urllib.parse import parse_qs
import wave

HERE = Path(__file__).resolve().parent
NAME = re.compile(r"[A-Za-z0-9_-]{1,80}\Z")
KEY = re.compile(r"[A-Za-z0-9._:-]{1,128}\Z")


class HTTPError(Exception):
    def __init__(self, status, message):
        self.status, self.message = status, message


def validate_wav(data):
    try:
        with wave.open(io.BytesIO(data), "rb") as audio:
            frames = audio.getnframes()
            if (audio.getnchannels(), audio.getsampwidth(), audio.getframerate(), audio.getcomptype()) != (1, 2, 16000, "NONE"):
                raise ValueError()
            if not 1 <= frames <= 480000 or len(audio.readframes(frames)) != frames * 2:
                raise ValueError()
    except (wave.Error, EOFError, ValueError):
        raise HTTPError(400, "Use mono 16 kHz PCM WAV, at most 30 seconds.") from None


class Application:
    def __init__(self, root=None, binary=None, password=None, origin=None):
        self.root = Path(root or os.environ.get("PAJA_DATA", ".local/web-data")).resolve()
        self.binary = str(binary or os.environ.get("PAJA_BINARY", "target/debug/paja"))
        self.binary = str(Path(self.binary).resolve())
        self.password = password or os.environ.get("PAJA_PASSWORD", "")
        self.origin = (origin or os.environ.get("PAJA_ORIGIN", "http://127.0.0.1:8787")).rstrip("/")
        if len(self.password) < 24:
            raise ValueError("Set PAJA_PASSWORD to a random secret of at least 24 characters.")
        if not re.fullmatch(r"https://[A-Za-z0-9.-]+(?::[0-9]+)?|http://(?:127\.0\.0\.1|localhost):[0-9]+", self.origin):
            raise ValueError("PAJA_ORIGIN must be an HTTPS origin (or localhost for development).")
        self.projects = self.root / "projects"
        self.projects.mkdir(mode=0o700, parents=True, exist_ok=True)
        self.sessions = {}
        self.rates = defaultdict(deque)
        self.lock = threading.Lock()
        self.voice_lock = threading.Lock()

    def rate(self, key, limit):
        now = time.monotonic()
        with self.lock:
            queue = self.rates[key]
            while queue and queue[0] < now - 60:
                queue.popleft()
            if len(queue) >= limit:
                raise HTTPError(429, "Please wait a minute before trying again.")
            queue.append(now)

    def read(self, env, limit):
        if env.get("HTTP_TRANSFER_ENCODING"):
            raise HTTPError(400, "A bounded Content-Length is required.")
        try:
            length = int(env.get("CONTENT_LENGTH", "0"))
        except ValueError:
            raise HTTPError(400, "Invalid content length.") from None
        if not 0 < length <= limit:
            raise HTTPError(413, "Request is empty or too large.")
        data = env["wsgi.input"].read(length)
        if len(data) != length:
            raise HTTPError(400, "Incomplete request.")
        return data

    def body(self, env):
        if env.get("CONTENT_TYPE", "").split(";")[0] != "application/json":
            raise HTTPError(415, "Expected JSON.")
        try:
            value = json.loads(self.read(env, 16384))
        except (UnicodeError, ValueError):
            raise HTTPError(400, "Invalid JSON.") from None
        if not isinstance(value, dict):
            raise HTTPError(400, "Expected a JSON object.")
        return value

    def session(self, env):
        cookie = SimpleCookie()
        try:
            cookie.load(env.get("HTTP_COOKIE", ""))
            token = cookie["paja_session"].value
        except (KeyError, Exception):
            raise HTTPError(401, "Sign in to your Paja server.") from None
        with self.lock:
            if self.sessions.get(token, 0) < time.time():
                self.sessions.pop(token, None)
                raise HTTPError(401, "Sign in to your Paja server.")
        return token

    def cookie(self, token, age=28800):
        secure = "; Secure" if self.origin.startswith("https:") else ""
        return f"paja_session={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age={age}{secure}"

    def cli(self, directory, arguments):
        try:
            result = subprocess.run([self.binary, "-C", str(directory), "--json", *arguments],
                                    capture_output=True, timeout=30, check=False)
            value = json.loads(result.stdout)
        except subprocess.TimeoutExpired:
            raise HTTPError(504, "Coordinator timed out. Retry the same request to reconcile its receipt.") from None
        except (OSError, ValueError):
            raise HTTPError(503, "Coordinator unavailable.") from None
        if not isinstance(value, dict) or value.get("schema_version") != "paja.result.v1":
            raise HTTPError(502, "Invalid coordinator response.")
        return value

    def project(self, name):
        if not NAME.fullmatch(name):
            raise HTTPError(400, "Invalid project name.")
        directory = self.projects / name
        if directory.is_symlink() or not directory.is_dir():
            raise HTTPError(404, "Project not found.")
        identity = self.cli(directory, ["identity"])
        if not identity.get("ok"):
            raise HTTPError(409, "Could not read this project.")
        return directory, identity["project"]

    def prompt(self, value):
        if not isinstance(value, str) or not value.strip() or len(value.encode()) > 4096:
            raise HTTPError(400, "Enter a prompt of 1–4096 UTF-8 bytes.")
        return value.strip()

    def key(self, body):
        value = body.get("request_id")
        if not isinstance(value, str) or not KEY.fullmatch(value):
            raise HTTPError(400, "An explicit request_id is required.")
        return value

    def route(self, env):
        method, path = env["REQUEST_METHOD"], env.get("PATH_INFO", "/")
        if method == "GET" and path == "/healthz":
            return 200, {"ok": True}, []
        if method == "GET" and path in {"/", "/app.js", "/style.css"}:
            filename = {"/": "index.html"}.get(path, path[1:])
            content_type = {"/": "text/html", "/app.js": "text/javascript", "/style.css": "text/css"}[path]
            return 200, (HERE / "static" / filename).read_bytes(), [("Content-Type", content_type + "; charset=utf-8")]
        if method not in {"GET", "POST"}:
            raise HTTPError(405, "Method not allowed.")
        if method == "POST" and env.get("HTTP_ORIGIN") != self.origin:
            raise HTTPError(403, "Origin does not match this server.")
        if method == "POST" and path == "/api/login":
            self.rate("login", 10)
            password = self.body(env).get("password", "")
            if not isinstance(password, str) or not hmac.compare_digest(hashlib.sha256(password.encode()).digest(), hashlib.sha256(self.password.encode()).digest()):
                raise HTTPError(401, "Incorrect access key.")
            token = secrets.token_urlsafe(32)
            with self.lock:
                self.sessions = {key: expiry for key, expiry in self.sessions.items() if expiry > time.time()}
                if len(self.sessions) >= 128:
                    raise HTTPError(429, "Too many active sessions.")
                self.sessions[token] = time.time() + 28800
            return 200, {"ok": True}, [("Set-Cookie", self.cookie(token))]
        token = self.session(env)
        if method == "POST" and path == "/api/logout":
            with self.lock:
                self.sessions.pop(token, None)
            return 200, {"ok": True}, [("Set-Cookie", self.cookie("", 0))]
        if method == "GET" and path == "/api/projects":
            projects = []
            for folder in sorted(self.projects.iterdir())[:100]:
                if folder.is_dir() and not folder.is_symlink() and NAME.fullmatch(folder.name):
                    _, metadata = self.project(folder.name)
                    projects.append({"name": folder.name, **metadata})
            model = os.environ.get("PAJA_SPEECH_MODEL", "small")
            ready = (self.root / "models" / model / "model.bin").is_file()
            return 200, {"projects": projects, "speech_ready": ready}, []
        if method == "POST" and path == "/api/voice":
            self.rate("voice", 3)
            language = parse_qs(env.get("QUERY_STRING", "")).get("language", [""])[0]
            if language not in {"en", "fi"}:
                raise HTTPError(400, "Choose English or Finnish.")
            data = self.read(env, 960044)
            validate_wav(data)
            if not self.voice_lock.acquire(blocking=False):
                raise HTTPError(429, "The transcriber is busy. Please try again shortly.")
            try:
                with tempfile.TemporaryDirectory(prefix="paja-audio-") as temp:
                    file = Path(temp) / "prompt.wav"
                    file.write_bytes(data)
                    try:
                        child_env = {**os.environ, "PAJA_DATA": str(self.root), "HF_HUB_OFFLINE": "1"}
                        result = subprocess.run([sys.executable, str(HERE / "transcribe.py"), "--audio", str(file), "--language", language],
                                                capture_output=True, timeout=90, env=child_env, check=False)
                    except subprocess.TimeoutExpired:
                        raise HTTPError(504, "Transcription timed out. Your draft is unchanged.") from None
                    if result.returncode:
                        raise HTTPError(503, "Speech unavailable or no speech detected. Check model setup, or try a shorter recording.")
                    transcript = json.loads(result.stdout)
                    self.prompt(transcript.get("text"))
                    return 200, transcript, []
            finally:
                self.voice_lock.release()
        if method == "POST" and path == "/api/projects":
            body = self.body(env)
            if set(body) != {"name", "prompt", "request_id"} or not isinstance(body["name"], str) or not NAME.fullmatch(body["name"]):
                raise HTTPError(400, "Provide a project name, prompt, and request_id.")
            key, prompt = self.key(body), self.prompt(body["prompt"])
            if not (self.projects / body["name"]).exists() and len(list(self.projects.iterdir())) >= 100:
                raise HTTPError(409, "This owner workspace has reached its 100-project limit.")
            with tempfile.TemporaryDirectory(prefix="paja-input-") as temp:
                file = Path(temp) / "prompt.txt"; file.write_text(prompt)
                result = self.cli(self.projects, ["init", body["name"], "--goal-file", str(file), "--request-id", key])
            return 200, result, []
        match = re.fullmatch(r"/api/projects/([A-Za-z0-9_-]{1,80})/(status|build|test|improve|run|results)", path)
        if method == "POST" and match:
            name, operation = match.groups()
            body = self.body(env)
            allowed = {"project", "revision", "request_id", "prompt", "from", "run"}
            if set(body) - allowed:
                raise HTTPError(400, "Unknown operation field.")
            directory, metadata = self.project(name)
            if body.get("project") != metadata["id"]:
                raise HTTPError(409, "Project identity changed. Refresh before continuing.")
            args = [operation, "--project", metadata["id"]]
            if operation not in {"status", "results"}:
                revision = body.get("revision")
                if type(revision) is not int or not 0 <= revision <= 4294967295:
                    raise HTTPError(400, "Provide the displayed revision.")
                args += ["--revision", str(revision), "--request-id", self.key(body)]
            for action, field, flag in [("improve", "from", "--from"), ("results", "run", "--run")]:
                if operation == action:
                    value = body.get(field)
                    if not isinstance(value, str) or not KEY.fullmatch(value):
                        raise HTTPError(400, "Select a completed test run first.")
                    args += [flag, value]
            with tempfile.TemporaryDirectory(prefix="paja-input-") as temp:
                if operation == "run":
                    file = Path(temp) / "prompt.txt"; file.write_text(self.prompt(body.get("prompt")))
                    args += ["--input-file", str(file)]
                return 200, self.cli(directory, args), []
        raise HTTPError(404, "Not found.")

    def __call__(self, env, start_response):
        try:
            status, payload, headers = self.route(env)
        except HTTPError as error:
            status, payload, headers = error.status, {"ok": False, "error": {"message": error.message}}, []
        except Exception:
            status, payload, headers = 500, {"ok": False, "error": {"message": "Request failed. Check server configuration."}}, []
        if not isinstance(payload, bytes):
            payload = json.dumps(payload).encode()
            headers.append(("Content-Type", "application/json"))
        headers += [("Content-Length", str(len(payload))), ("Cache-Control", "no-store"),
                    ("X-Content-Type-Options", "nosniff"), ("Referrer-Policy", "no-referrer"),
                    ("Permissions-Policy", "microphone=(self), camera=(), geolocation=()"),
                    ("Content-Security-Policy", "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; media-src 'self' blob:; frame-ancestors 'none'; base-uri 'none'; form-action 'self'")]
        start_response(f"{status} Response", headers)
        return [payload]


def create_app():
    return Application()


if __name__ == "__main__":
    from socketserver import ThreadingMixIn
    from wsgiref.simple_server import WSGIServer, WSGIRequestHandler, make_server
    class ThreadedServer(ThreadingMixIn, WSGIServer):
        daemon_threads = True
    class QuietHandler(WSGIRequestHandler):
        def log_message(self, *args):
            pass
    with make_server("127.0.0.1", int(os.environ.get("PORT", "8787")), create_app(), ThreadedServer, QuietHandler) as server:
        print("Paja web ready on localhost. Use Docker/Caddy for HTTPS deployment.")
        server.serve_forever()
