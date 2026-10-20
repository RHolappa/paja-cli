// SPDX-License-Identifier: Apache-2.0
const $ = (id) => document.getElementById(id);
let projects = [], selected, latestRun, busy = false, speechReady = false;
let recording = null, generation = 0, speechAbort;
let pending;
try { $('prompt').value = localStorage.getItem('paja-draft') || ''; pending = JSON.parse(localStorage.getItem('paja-pending') || 'null'); } catch {}
function message(text) { $('message').textContent = text; }
function saveDraft() { try { localStorage.setItem('paja-draft', $('prompt').value); } catch {} }
function setBusy(value) {
  busy = value;
  document.querySelectorAll('#workspace button, #workspace select, #workspace input').forEach(el => { el.disabled = value; });
  $('prompt').readOnly = value;
  $('record').disabled = value || !speechReady || !navigator.mediaDevices?.getUserMedia || !window.MediaRecorder;
  $('retry').disabled = value;
  if (value && (recording || speechAbort)) { $('cancel').disabled = false; $('stop').disabled = false; }
}
async function api(path, body, options = {}) {
  const response = await fetch(path, { credentials: 'same-origin', ...options,
    ...(body === undefined ? {} : { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }) });
  let value;
  try { value = await response.json(); } catch { throw new Error('The server did not return a usable response.'); }
  if (!response.ok) {
    if (response.status === 401) { $('login').hidden = false; $('workspace').hidden = true; }
    const error = new Error(value.error?.message || 'Request failed.');
    // A timeout or dropped response can conceal a committed operation.
    error.definitive = response.status < 500;
    throw error;
  }
  return value;
}
async function refresh(preferred) {
  const result = await api('/api/projects'); projects = result.projects; speechReady = result.speech_ready;
  $('speech-hint').hidden = speechReady;
  $('projects').replaceChildren(...projects.map(p => { const option = document.createElement('option'); option.value = p.name; option.textContent = p.name; return option; }));
  selected = projects.find(p => p.name === preferred) || projects[0];
  if (selected) $('projects').value = selected.name;
  $('project-controls').hidden = !selected;
  $('revision').textContent = selected ? `Version ${selected.candidate_revision} · ${selected.state}` : '';
  $('login').hidden = true; $('workspace').hidden = false; $('pending').hidden = !pending;
  setBusy(false);
}
function showResult(result) {
  if (!result.ok) { message(result.error?.message || 'Operation failed.'); return; }
  const data = result.data || {}, lines = [];
  if (data.message) lines.push(data.message);
  if (result.project) lines.push(`Version ${result.project.candidate_revision} · ${result.project.state}`);
  if (data.execution?.output?.total_cents !== undefined) lines.push(`Selected lunches total €${(data.execution.output.total_cents / 100).toFixed(2)}.`);
  if (data.comparison) lines.push(`Improved: ${data.comparison.improved?.join(', ') || 'none'}. Regressed: ${data.comparison.regressed?.join(', ') || 'none'}.`);
  $('summary').textContent = lines.join('\n\n'); $('details').textContent = JSON.stringify(result, null, 2);
  $('counts').replaceChildren();
  if (data.report?.counts) for (const [key, count] of Object.entries(data.report.counts)) {
    const label = document.createElement('span'); label.className = `count ${key === 'failed' && count ? 'fail' : ''}`;
    label.textContent = `${count} ${key}`; $('counts').append(label);
  }
  $('evidence').hidden = false;
  if (['test', 'improve'].includes(result.run?.kind)) latestRun = result.run.id;
  else if (data.latest_test_run) latestRun = data.latest_test_run;
  message('');
}
function savePending(value) {
  pending = value; $('pending').hidden = !value;
  try { value ? localStorage.setItem('paja-pending', JSON.stringify(value)) : localStorage.removeItem('paja-pending'); } catch {}
}
async function dispatch(request) {
  setBusy(true); message('Working…'); savePending(request);
  try {
    const result = await api(request.path, request.body); savePending(null); showResult(result);
    await refresh(request.name);
  } catch (error) {
    if (error.definitive) savePending(null);
    message(error.message);
  } finally { setBusy(false); }
}
$('login-form').addEventListener('submit', async event => {
  event.preventDefault();
  try { await api('/api/login', { password: $('password').value }); $('password').value = ''; await refresh(); message(''); }
  catch (error) { message(error.message); }
});
$('logout').onclick = async () => { cancelSpeech(); await api('/api/logout', {}); $('workspace').hidden = true; $('login').hidden = false; };
$('prompt').addEventListener('input', saveDraft);
$('example').onclick = () => { $('prompt').value = 'Choose two lunches for a total of eight euros'; saveDraft(); };
$('projects').onchange = () => { selected = projects.find(p => p.name === $('projects').value); latestRun = null; $('revision').textContent = `Version ${selected.candidate_revision} · ${selected.state}`; };
$('retry').onclick = () => pending && dispatch(pending);
$('create-form').onsubmit = event => {
  event.preventDefault(); if (busy) return;
  if (pending) return message('Reconcile the saved request first.');
  dispatch({ path: '/api/projects', name: $('project-name').value, body: { name: $('project-name').value, prompt: $('prompt').value, request_id: crypto.randomUUID() } });
};
document.querySelectorAll('[data-operation]').forEach(button => button.onclick = async () => {
  if (busy || !selected) return;
  if (pending) return message('Reconcile the saved request first.');
  const operation = button.dataset.operation, name = selected.name;
  const body = { project: selected.id, revision: selected.candidate_revision, request_id: crypto.randomUUID() };
  if (operation === 'run') body.prompt = $('prompt').value;
  if (operation === 'improve' || operation === 'results') {
    setBusy(true);
    try { const status = await api(`/api/projects/${name}/status`, { project: selected.id }); latestRun = status.data?.latest_test_run; }
    catch (error) { message(error.message); setBusy(false); return; }
    setBusy(false);
    if (!latestRun) return message('Test this version first.');
    body[operation === 'improve' ? 'from' : 'run'] = latestRun;
  }
  await dispatch({ path: `/api/projects/${name}/${operation}`, body, name });
});

function finishUI() {
  $('recording').hidden = true; $('stop').hidden = true; $('cancel').hidden = true; $('record').hidden = false;
  setBusy(false);
}
function cancelSpeech() {
  generation++; speechAbort?.abort(); speechAbort = undefined;
  if (recording) {
    clearInterval(recording.timer); clearTimeout(recording.limit);
    recording.recorder.onstop = null;
    if (recording.recorder.state !== 'inactive') recording.recorder.stop();
    recording.stream.getTracks().forEach(track => track.stop()); recording = null;
  }
  finishUI(); message('Cancelled. Your draft is unchanged.');
}
async function wav(blob) {
  const context = new AudioContext();
  let decoded;
  try { decoded = await context.decodeAudioData(await blob.arrayBuffer()); } finally { await context.close(); }
  if (!decoded.duration || decoded.duration > 30.2) throw new Error('Recording exceeded 30 seconds. Please try again.');
  const length = Math.min(480000, Math.ceil(decoded.duration * 16000));
  const offline = new OfflineAudioContext(1, length, 16000);
  const source = offline.createBufferSource(); source.buffer = decoded; source.connect(offline.destination); source.start();
  const mono = (await offline.startRendering()).getChannelData(0);
  const buffer = new ArrayBuffer(44 + mono.length * 2), view = new DataView(buffer);
  const text = (at, value) => [...value].forEach((char, i) => view.setUint8(at + i, char.charCodeAt(0)));
  text(0, 'RIFF'); view.setUint32(4, buffer.byteLength - 8, true); text(8, 'WAVE'); text(12, 'fmt ');
  view.setUint32(16, 16, true); view.setUint16(20, 1, true); view.setUint16(22, 1, true);
  view.setUint32(24, 16000, true); view.setUint32(28, 32000, true); view.setUint16(32, 2, true); view.setUint16(34, 16, true);
  text(36, 'data'); view.setUint32(40, mono.length * 2, true);
  mono.forEach((sample, i) => view.setInt16(44 + i * 2, Math.max(-1, Math.min(1, sample)) * (sample < 0 ? 32768 : 32767), true));
  return buffer;
}
$('record').onclick = async () => {
  if (busy) return;
  const ticket = ++generation, language = $('language').value;
  let permissionTimer;
  const timeout = new Promise((_, reject) => { permissionTimer = setTimeout(() => reject(new Error('Microphone permission timed out. Check your browser settings.')), 20000); });
  const permission = navigator.mediaDevices.getUserMedia({ audio: true });
  permission.then(stream => { if (ticket !== generation) stream.getTracks().forEach(track => track.stop()); }).catch(() => {});
  setBusy(true); $('cancel').hidden = false; $('cancel').disabled = false; message('Allow microphone access to start recording.');
  try {
    const stream = await Promise.race([permission, timeout]); clearTimeout(permissionTimer);
    if (ticket !== generation) { stream.getTracks().forEach(track => track.stop()); return; }
    let recorder;
    try { recorder = new MediaRecorder(stream); } catch (error) { stream.getTracks().forEach(track => track.stop()); throw error; }
    const chunks = [], start = Date.now();
    recording = { recorder, stream };
    recorder.ondataavailable = event => { if (event.data.size) chunks.push(event.data); };
    recorder.onerror = () => { cancelSpeech(); message('Recording failed. Your draft is unchanged.'); };
    recorder.onstop = async () => {
      stream.getTracks().forEach(track => track.stop());
      if (recording) { clearInterval(recording.timer); clearTimeout(recording.limit); recording = null; }
      if (ticket !== generation) return;
      $('recording').hidden = true; $('stop').hidden = true; message('Transcribing on your private server…');
      speechAbort = new AbortController();
      try {
        const audio = await wav(new Blob(chunks, { type: recorder.mimeType }));
        if (ticket !== generation) return;
        const result = await api(`/api/voice?language=${language}`, undefined, { method: 'POST', headers: { 'Content-Type': 'audio/wav' }, body: audio, signal: speechAbort.signal });
        if (ticket !== generation) return;
        $('prompt').value = result.text; saveDraft(); message('Review the transcript before using it.');
      } catch (error) { if (ticket === generation) message(error.message); }
      finally { if (ticket === generation) { speechAbort = undefined; finishUI(); } }
    };
    recorder.start();
    recording.timer = setInterval(() => { $('elapsed').textContent = `${Math.floor((Date.now() - start) / 1000)} / 30 sec`; }, 250);
    recording.limit = setTimeout(() => { if (recorder.state === 'recording') recorder.stop(); }, 29900);
    $('elapsed').textContent = '0 / 30 sec'; $('recording').hidden = false; $('record').hidden = true; $('stop').hidden = false; $('stop').disabled = false; message('');
  } catch (error) { if (ticket === generation) { cancelSpeech(); message(error.message); } }
  finally { clearTimeout(permissionTimer); }
};
$('stop').onclick = () => { if (recording?.recorder.state === 'recording') recording.recorder.stop(); };
$('cancel').onclick = cancelSpeech;
document.addEventListener('keydown', event => { if (event.key === 'Escape' && (recording || speechAbort || busy && !$('cancel').hidden)) cancelSpeech(); });
document.addEventListener('visibilitychange', () => { if (document.hidden && (recording || speechAbort || busy && !$('cancel').hidden)) cancelSpeech(); });
window.addEventListener('pagehide', () => { if (recording || speechAbort) cancelSpeech(); });
refresh().catch(error => { if (!error.message.includes('Sign in')) message(error.message); $('login').hidden = false; });
