import * as vscode from 'vscode';

let _currentPanel: ParticleViewPanel | undefined;

/**
 * Hosts the Three.js / WebGL2 particle viewport in a VSCodium webview panel.
 * On bare-metal Fedora 44 with a real GPU, this upgrades to WebGPU automatically.
 */
export class ParticleViewPanel implements vscode.Disposable {
    public static readonly viewType = 'dubsar.particleView';

    private readonly _panel: vscode.WebviewPanel;
    private readonly _disposables: vscode.Disposable[] = [];

    public static createOrShow(extensionUri: vscode.Uri): void {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        if (_currentPanel) {
            _currentPanel._panel.reveal(column ?? vscode.ViewColumn.Two);
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            ParticleViewPanel.viewType,
            'DubSar 3D Viewport',
            column ?? vscode.ViewColumn.Two,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [vscode.Uri.joinPath(extensionUri, 'static')],
            }
        );

        _currentPanel = new ParticleViewPanel(panel, extensionUri);
    }

    public static updateProjection(projName: string): void {
        _currentPanel?._panel.webview.postMessage({ type: 'setProjection', projection: projName });
    }

    public static updateParticleData(b64Png: string): void {
        _currentPanel?._panel.webview.postMessage({ type: 'renderPng', data: b64Png });
    }

    private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
        this._panel = panel;
        this._panel.webview.html = this._buildHtml(extensionUri);

        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        this._panel.webview.onDidReceiveMessage(
            (msg: { type: string; data: unknown }) => {
                if (msg.type === 'ready') {
                    this._panel.webview.postMessage({ type: 'hello', version: '0.1.0' });
                }
            },
            null,
            this._disposables
        );
    }

    private _buildHtml(extensionUri: vscode.Uri): string {
        const nonce = getNonce();
        return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy"
        content="default-src 'none'; script-src 'nonce-${nonce}' https://cdn.jsdelivr.net;
                 style-src 'unsafe-inline'; img-src data: blob:;">
  <title>DubSar 3D Viewport</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { background: #0c0c12; color: #e0e0e0; font-family: monospace; display: flex; flex-direction: column; height: 100vh; }
    #toolbar { padding: 6px 12px; background: #1a1a2e; border-bottom: 1px solid #333; display: flex; gap: 12px; align-items: center; font-size: 12px; }
    #toolbar select { background: #0f3460; color: #e0e0e0; border: 1px solid #444; padding: 2px 6px; border-radius: 3px; }
    #canvas-wrapper { flex: 1; position: relative; }
    #canvas { width: 100%; height: 100%; display: block; }
    #stats { position: absolute; top: 8px; left: 8px; font-size: 11px; color: #88ff88; opacity: 0.8; line-height: 1.5; }
    #png-output { max-width: 100%; max-height: 100%; object-fit: contain; display: none; }
  </style>
</head>
<body>
  <div id="toolbar">
    <span style="color:#88aaff;font-weight:bold;">DubSar 3D Viewport</span>
    <label>Projection:
      <select id="proj-select">
        <option value="default">Spatial (pos.xyz)</option>
        <option value="pos_mom_z">Position + Momentum Z</option>
        <option value="phase_space">Phase Space</option>
      </select>
    </label>
    <span id="backend-label" style="color:#aaa;">backend: software</span>
  </div>

  <div id="canvas-wrapper">
    <!-- PNG output mode (software renderer frames) -->
    <img id="png-output" alt="Simulation frame" />

    <!-- WebGL/WebGPU canvas for live rendering -->
    <canvas id="canvas"></canvas>

    <div id="stats">
      <div>Tribe A: <span id="stat-a">—</span></div>
      <div>Tribe B: <span id="stat-b">—</span></div>
      <div>Free:    <span id="stat-free">—</span></div>
      <div>Step:    <span id="stat-step">0</span></div>
    </div>
  </div>

  <script nonce="${nonce}" src="https://cdn.jsdelivr.net/npm/three@0.165.0/build/three.min.js"></script>
  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();

    // ── Three.js scene ───────────────────────────────────────────────────────
    const canvas = document.getElementById('canvas');
    const pngOut = document.getElementById('png-output');

    const renderer = new THREE.WebGLRenderer({ canvas, antialias: false });
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.setClearColor(0x0c0c12);

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(60, canvas.clientWidth / canvas.clientHeight, 0.001, 1000);
    camera.position.set(0, 0, 3);
    scene.add(new THREE.AmbientLight(0xffffff, 0.5));

    // Point cloud (placeholder — updated from kernel)
    const geo = new THREE.BufferGeometry();
    const positions = new Float32Array(300000 * 3); // 300K placeholder points
    for (let i = 0; i < positions.length; i++) {
      positions[i] = (Math.random() - 0.5) * 2;
    }
    geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    const mat = new THREE.PointsMaterial({ size: 0.004, color: 0x44ff88, transparent: true, opacity: 0.7 });
    const points = new THREE.Points(geo, mat);
    scene.add(points);

    function resize() {
      const w = canvas.clientWidth, h = canvas.clientHeight;
      renderer.setSize(w, h, false);
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
    }
    new ResizeObserver(resize).observe(canvas);
    resize();

    let angle = 0;
    function animate() {
      requestAnimationFrame(animate);
      angle += 0.003;
      camera.position.x = Math.sin(angle) * 3;
      camera.position.z = Math.cos(angle) * 3;
      camera.lookAt(0, 0, 0);
      renderer.render(scene, camera);
    }
    animate();

    // ── VSCode message handler ───────────────────────────────────────────────
    window.addEventListener('message', e => {
      const msg = e.data;
      if (msg.type === 'renderPng') {
        // Software renderer frame — show as image
        pngOut.src = 'data:image/png;base64,' + msg.data;
        pngOut.style.display = 'block';
        canvas.style.display = 'none';
      } else if (msg.type === 'updateStats') {
        document.getElementById('stat-a').textContent = msg.tribe_a;
        document.getElementById('stat-b').textContent = msg.tribe_b;
        document.getElementById('stat-free').textContent = msg.free;
        document.getElementById('stat-step').textContent = msg.step;
      } else if (msg.type === 'setProjection') {
        // Will be used once GPU path sends buffer data
        document.getElementById('proj-select').value = msg.projection;
      } else if (msg.type === 'hello') {
        document.getElementById('backend-label').textContent = 'backend: software (VM)';
      }
    });

    document.getElementById('proj-select').addEventListener('change', e => {
      vscode.postMessage({ type: 'projectionChanged', value: e.target.value });
    });

    // Signal ready
    vscode.postMessage({ type: 'ready' });
  </script>
</body>
</html>`;
    }

    dispose(): void {
        _currentPanel = undefined;
        this._panel.dispose();
        this._disposables.forEach(d => d.dispose());
    }
}

function getNonce(): string {
    let text = '';
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    for (let i = 0; i < 32; i++) text += chars.charAt(Math.floor(Math.random() * chars.length));
    return text;
}
