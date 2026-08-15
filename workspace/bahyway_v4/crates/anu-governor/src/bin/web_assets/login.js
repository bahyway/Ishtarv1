// Reads the selected vault file as raw bytes, base64-encodes it client
// side, and posts it + the passphrase to /api/login. The passphrase never
// touches anything but this one POST body over TLS; the actual Argon2id
// derivation + ChaCha20-Poly1305 decrypt + Ed25519 seal verification all
// happen server-side against the real kupru crate (anu_governor::web_auth).

function arrayBufferToBase64(buf) {
  let binary = '';
  const bytes = new Uint8Array(buf);
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
}

async function attemptLogin() {
  const fileInput = document.getElementById('vault-file');
  const pathInput = document.getElementById('vault-path');
  const passInput = document.getElementById('passphrase');
  const status = document.getElementById('login-status');
  const btn = document.getElementById('btn-login');

  const file = fileInput.files[0];
  const vaultPath = pathInput ? pathInput.value.trim() : '';
  if (!file && !vaultPath) {
    status.textContent = 'select a vault file, or enter a path on this machine, first';
    return;
  }

  btn.disabled = true;
  status.textContent = 'unlocking (Argon2id is deliberately slow — this can take a few seconds)…';
  try {
    // vault_path (server reads the file itself) takes priority when set --
    // it's the fallback for when the browser's native file picker won't
    // open at all (a real desktop/portal issue, not a page bug; see
    // anu_governor_web.rs's /api/login handler for why this is safe: Shala
    // only ever serves 127.0.0.1, with the invoking user's own filesystem
    // permissions either way).
    let body;
    if (vaultPath) {
      body = JSON.stringify({ vault_path: vaultPath, passphrase: passInput.value });
    } else {
      const buf = await file.arrayBuffer();
      const vault_b64 = arrayBufferToBase64(buf);
      body = JSON.stringify({ vault_b64, passphrase: passInput.value });
    }
    const res = await fetch('/api/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body,
    });
    if (res.ok) {
      window.location.href = '/';
      return;
    }
    const msg = await res.text();
    if (res.status === 429) {
      status.textContent = JSON.parse(msg).error || 'too many attempts — try again later';
    } else {
      try {
        status.textContent = JSON.parse(msg).error || msg;
      } catch {
        status.textContent = msg;
      }
    }
  } catch (e) {
    status.textContent = String(e.message || e);
  } finally {
    btn.disabled = false;
    passInput.value = '';
  }
}

document.getElementById('btn-login').addEventListener('click', attemptLogin);
document.getElementById('passphrase').addEventListener('keydown', (e) => {
  if (e.key === 'Enter') attemptLogin();
});
