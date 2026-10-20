#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Run inside the built image: actual HTTP server, authentication, and CLI."""
import http.cookiejar
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import urllib.request

with tempfile.TemporaryDirectory(prefix='paja-container-') as data:
    origin = 'http://localhost:8000'
    key = 'synthetic-container-check-0123456789'
    env = {**os.environ, 'PAJA_PASSWORD': key, 'PAJA_DATA': data, 'PAJA_ORIGIN': origin}
    server = subprocess.Popen(['gunicorn', '--bind', '127.0.0.1:8000', '--workers', '1', '--threads', '4', 'web.server:create_app()'], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        for _ in range(100):
            try:
                with urllib.request.urlopen(origin + '/healthz', timeout=1) as response:
                    assert response.status == 200
                break
            except OSError:
                time.sleep(0.1)
        else:
            raise RuntimeError('Container HTTP server failed to start')
        client = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
        def post(path, body):
            request = urllib.request.Request(origin + path, data=json.dumps(body).encode(), headers={'Content-Type': 'application/json', 'Origin': origin})
            with client.open(request, timeout=10) as response:
                value = json.load(response)
                assert value.get('ok'), value
                return value
        post('/api/login', {'password': key})
        created = post('/api/projects', {'name': 'container-check', 'prompt': 'Synthetic lunch brief', 'request_id': 'init-1'})
        project = created['project']['id']
        def operation(action, revision, **fields):
            return post('/api/projects/container-check/' + action, {'project': project, 'revision': revision, 'request_id': action, **fields})
        operation('build', 0)
        baseline = operation('test', 1)
        improved = operation('improve', 1, **{'from': baseline['run']['id']})
        assert improved['data']['report']['counts']['passed'] == 4
        result = operation('run', 2, prompt='Choose two lunches for a total of 8 euros.')
        assert result['data']['execution']['output']['total_cents'] == 800
        print('Container HTTP/login/coordinator loop passed.')
    finally:
        server.terminate()
        server.wait(timeout=10)
