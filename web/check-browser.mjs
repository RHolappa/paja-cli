// SPDX-License-Identifier: Apache-2.0
// Real browser/microphone plumbing and Rust coordinator; synthetic ASR response.
import { chromium } from 'playwright';
import { spawn } from 'node:child_process';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import assert from 'node:assert/strict';

const root = resolve(import.meta.dirname, '..');
const data = await mkdtemp(join(tmpdir(), 'paja-web-check-'));
const port = 43871, origin = `http://127.0.0.1:${port}`;
const password = 'synthetic-browser-key-0123456789';
await mkdir(join(data, 'models', 'small'), { recursive: true });
await writeFile(join(data, 'models', 'small', 'model.bin'), 'fixture readiness only');
const server = spawn('python3', ['web/server.py'], { cwd: root, env: { ...process.env, PAJA_PASSWORD: password, PAJA_DATA: data, PAJA_ORIGIN: origin, PORT: String(port) }, stdio: 'pipe' });
let browser;
try {
  for (let i = 0; i < 100; i++) {
    try { if ((await fetch(`${origin}/healthz`)).ok) break; } catch {}
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || undefined,
    args: ['--use-fake-ui-for-media-stream', '--use-fake-device-for-media-stream'] });
  const context = await browser.newContext({ permissions: ['microphone'], viewport: { width: 1280, height: 1100 } });
  await context.addInitScript(() => {
    window.capturedStreams = [];
    const original = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices);
    navigator.mediaDevices.getUserMedia = async (...args) => { const stream = await original(...args); window.capturedStreams.push(stream); return stream; };
  });
  const page = await context.newPage();
  const errors = []; page.on('pageerror', error => errors.push(error.message));
  await page.goto(origin);
  await page.locator('#password').fill(password);
  await page.locator('#login-form button').click();
  await page.locator('#workspace').waitFor({ state: 'visible' });
  await page.locator('#prompt').fill('A synthetic spoken lunch project');
  await page.locator('#create-form button').click();
  await page.locator('#project-controls').waitFor({ state: 'visible' });
  for (const action of ['build', 'test', 'improve']) {
    await page.locator(`[data-operation="${action}"]`).click();
    await page.waitForFunction(() => !document.querySelector('[data-operation="build"]').disabled);
    assert.equal(await page.locator('#pending').isVisible(), false);
  }
  assert.match(await page.locator('#counts').textContent(), /4 passed/);
  await page.locator('#example').click();
  await page.locator('[data-operation="run"]').click();
  await page.waitForFunction(() => document.querySelector('#summary').textContent.includes('€8.00'));
  let voiceCalls = 0;
  await page.route('**/api/voice?*', async route => {
    voiceCalls++;
    assert.match(route.request().url(), /language=fi/);
    const audio = route.request().postDataBuffer();
    assert.equal(audio.toString('ascii', 0, 4), 'RIFF'); assert.ok(audio.length > 44 && audio.length <= 960044);
    await route.fulfill({ json: { schema_version: 'paja.transcript.v1', text: 'Rakenna lounasavustaja.', language: 'fi', backend: 'synthetic-test' } });
  });
  await page.locator('#language').selectOption('fi');
  await page.locator('#record').click(); await page.locator('#recording').waitFor({ state: 'visible' });
  await page.waitForTimeout(1200); await page.locator('#stop').click();
  await page.waitForFunction(() => document.querySelector('#prompt').value === 'Rakenna lounasavustaja.');
  assert.equal(voiceCalls, 1);
  assert.ok(await page.evaluate(() => window.capturedStreams.every(stream => stream.getTracks().every(track => track.readyState === 'ended'))));
  await page.locator('#record').click(); await page.locator('#recording').waitFor({ state: 'visible' });
  await page.locator('#cancel').click();
  assert.equal(await page.locator('#prompt').inputValue(), 'Rakenna lounasavustaja.');
  assert.equal(voiceCalls, 1);
  assert.ok(await page.evaluate(() => window.capturedStreams.every(stream => stream.getTracks().every(track => track.readyState === 'ended'))));
  await page.unroute('**/api/voice?*');
  await page.route('**/api/voice?*', route => route.fulfill({ status: 503, json: { error: { message: 'Synthetic provider failure' } } }));
  await page.locator('#record').click(); await page.locator('#recording').waitFor({ state: 'visible' });
  await page.waitForTimeout(1100); await page.locator('#stop').click();
  await page.waitForFunction(() => document.querySelector('#message').textContent.includes('Synthetic provider failure'));
  assert.equal(await page.locator('#prompt').inputValue(), 'Rakenna lounasavustaja.');
  await mkdir(join(root, 'output', 'voice-browser'), { recursive: true });
  await page.screenshot({ path: join(root, 'output', 'voice-browser', 'desktop.png'), fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.screenshot({ path: join(root, 'output', 'voice-browser', 'mobile.png'), fullPage: true });
  assert.deepEqual(errors, []);
  console.log('Browser checks passed: login, real coordinator loop, synthetic ASR with real fake-mic recording, Finnish routing, cancel/failure draft preservation, microphone release, desktop/mobile layout.');
} finally {
  await browser?.close(); server.kill(); await rm(data, { recursive: true, force: true });
}
