# SPDX-License-Identifier: Apache-2.0
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
import wave
from web.server import Application, HTTPError, validate_wav


class WebTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.app = Application(self.temp.name, Path('target/debug/paja'), 'synthetic-test-key-0123456789', 'http://localhost:8787')
        self.cookie = ''

    def request(self, path, body=None, origin='http://localhost:8787', method=None, raw=None):
        content = raw if raw is not None else json.dumps(body).encode() if body is not None else b''
        route, _, query = path.partition('?')
        env = {'REQUEST_METHOD': method or ('POST' if body is not None or raw is not None else 'GET'),
               'PATH_INFO': route, 'QUERY_STRING': query, 'HTTP_ORIGIN': origin,
               'CONTENT_TYPE': 'audio/wav' if raw is not None else 'application/json',
               'CONTENT_LENGTH': str(len(content)), 'wsgi.input': io.BytesIO(content), 'HTTP_COOKIE': self.cookie}
        response = {}
        def start(status, headers):
            response['status'] = int(status.split()[0]); response['headers'] = dict(headers)
        data = b''.join(self.app(env, start))
        if 'Set-Cookie' in response['headers']:
            self.cookie = response['headers']['Set-Cookie'].split(';')[0]
        return response['status'], json.loads(data) if response['headers']['Content-Type'] == 'application/json' else data

    def login(self):
        self.assertEqual(self.request('/api/login', {'password': self.app.password})[0], 200)

    def test_auth_origin_and_private_routes(self):
        self.assertEqual(self.request('/api/projects')[0], 401)
        self.assertEqual(self.request('/api/login', {'password': self.app.password}, origin='https://attacker.example')[0], 403)
        self.assertEqual(self.request('/api/login', {'password': 'wrong'})[0], 401)
        self.login()
        self.assertIn('paja_session=', self.cookie)
        self.assertEqual(self.request('/api/projects')[0], 200)
        self.assertEqual(self.request('/api/logout', {})[0], 200)
        self.assertEqual(self.request('/api/projects')[0], 401)

    def test_complete_real_coordinator_loop_and_replay(self):
        self.login()
        body = {'name': 'meal', 'prompt': 'A spoken lunch idea', 'request_id': 'init-voice'}
        _, created = self.request('/api/projects', body)
        self.assertTrue(created['ok'], created)
        self.assertEqual(self.request('/api/projects', body)[1], created)
        project = created['project']['id']
        def operation(name, revision, **extra):
            status, result = self.request('/api/projects/meal/' + name, {'project': project, 'revision': revision, 'request_id': name, **extra})
            self.assertEqual(status, 200)
            self.assertTrue(result['ok'], result)
            return result
        operation('build', 0)
        baseline = operation('test', 1)
        self.assertEqual(baseline['data']['report']['counts']['failed'], 2)
        candidate = operation('improve', 1, **{'from': baseline['run']['id']})
        self.assertEqual(candidate['data']['report']['counts']['passed'], 4)
        run = operation('run', 2, prompt='Choose two lunches for a total of eight euros')
        self.assertEqual(run['data']['execution']['output']['total_cents'], 800)
        self.assertEqual(operation('run', 2, prompt='Choose two lunches for a total of eight euros'), run)
        historical = operation('results', 2, run=baseline['run']['id'])
        self.assertEqual(historical['data']['report']['counts']['failed'], 2)
        self.assertEqual(self.request('/api/projects')[1]['projects'][0]['candidate_revision'], 2)
        # Restart only the web layer; coordinator evidence stays on disk.
        self.app = Application(self.temp.name, Path('target/debug/paja'), self.app.password, self.app.origin)
        self.login()
        self.assertEqual(self.request('/api/projects')[1]['projects'][0]['candidate_revision'], 2)

    def test_rejects_paths_invalid_identity_and_unbounded_input(self):
        self.login()
        for name in ['../escape', '/tmp/escape', 'name;touch nope']:
            self.assertEqual(self.request('/api/projects', {'name': name, 'prompt': 'text', 'request_id': 'x'})[0], 400)
        self.assertEqual(self.request('/api/projects', {'name': 'valid', 'prompt': 'x' * 17000, 'request_id': 'x'})[0], 413)
        self.assertEqual(self.request('/api/projects', {'name': 'valid', 'prompt': 'text', 'request_id': 'x', 'command': 'sh'})[0], 400)
        self.request('/api/projects', {'name': 'valid', 'prompt': 'text', 'request_id': 'x'})
        self.assertEqual(self.request('/api/projects/valid/build', {'project': 'wrong', 'revision': 0, 'request_id': 'b'})[0], 409)

    def test_audio_validation_precedes_inference(self):
        self.login()
        self.assertEqual(self.request('/api/voice?language=fi', raw=b'not wave')[0], 400)
        self.assertEqual(self.request('/api/voice?language=de', raw=b'not wave')[0], 400)
        def wav(frames, channels=1):
            buffer = io.BytesIO()
            with wave.open(buffer, 'wb') as out:
                out.setnchannels(channels); out.setsampwidth(2); out.setframerate(16000); out.writeframes(b'\0' * frames * 2 * channels)
            return buffer.getvalue()
        validate_wav(wav(16000))
        for data in [wav(480001), wav(100, 2), wav(100)[:-2]]:
            with self.assertRaises(HTTPError):
                validate_wav(data)

    def test_no_arbitrary_static_files_or_source(self):
        for path in ['/server.py', '/.env', '/../../Cargo.toml']:
            self.assertNotEqual(self.request(path)[0], 200)
        self.assertEqual(self.request('/')[0], 200)


if __name__ == '__main__':
    unittest.main()
