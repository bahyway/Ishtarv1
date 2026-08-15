import * as vscode from 'vscode';
import * as path from 'path';
import * as cp from 'child_process';
import { DubSarNotebookController } from './kernel/notebookController';
import { ParticleViewPanel } from './webview/particleView';

let controller: DubSarNotebookController | undefined;

export function activate(context: vscode.ExtensionContext) {
    console.log('DubSar IDE activated');

    // Notebook controller — wires .dubsar files to the Rust kernel
    controller = new DubSarNotebookController(context);
    context.subscriptions.push(controller);

    // Command: open a new storage fragmentation notebook
    context.subscriptions.push(
        vscode.commands.registerCommand('dubsar.openStorageSim', async () => {
            const uri = await vscode.window.showSaveDialog({
                defaultUri: vscode.Uri.file(path.join(vscode.workspace.rootPath ?? '', 'storage_fragmentation.dubsar')),
                filters: { 'DubSar Notebooks': ['dubsar'] },
            });
            if (!uri) return;

            const content = buildStarterNotebook();
            await vscode.workspace.fs.writeFile(uri, Buffer.from(JSON.stringify(content, null, 2)));
            await vscode.commands.executeCommand('vscode.openWith', uri, 'jupyter-notebook');
        })
    );

    // Command: install the kernel spec
    context.subscriptions.push(
        vscode.commands.registerCommand('dubsar.installKernel', async () => {
            const config = vscode.workspace.getConfiguration('dubsar');
            const kernelPath = config.get<string>('kernelPath', 'dubsar-kernel');

            const term = vscode.window.createTerminal('DubSar Kernel Install');
            term.sendText(`${kernelPath} --install`);
            term.show();
        })
    );

    // Command: open 3D viewport panel
    context.subscriptions.push(
        vscode.commands.registerCommand('dubsar.render3d', () => {
            ParticleViewPanel.createOrShow(context.extensionUri);
        })
    );

    // Command: change projection
    context.subscriptions.push(
        vscode.commands.registerCommand('dubsar.setProjection', async () => {
            const choices = [
                { label: 'Default (pos.xyz)', value: 'default' },
                { label: 'Position + Momentum Z', value: 'pos_mom_z' },
                { label: 'Phase Space (pos.x / mom.x / scalar)', value: 'phase_space' },
            ];
            const pick = await vscode.window.showQuickPick(choices, {
                placeHolder: 'Select 7D→3D projection',
            });
            if (!pick) return;
            ParticleViewPanel.updateProjection(pick.value);
        })
    );
}

export function deactivate() {
    controller?.dispose();
}

function buildStarterNotebook(): object {
    return {
        nbformat: 4,
        nbformat_minor: 5,
        metadata: {
            kernelspec: {
                display_name: 'DubSar (Rust + Vulkan)',
                language: 'rust',
                name: 'dubsar',
            },
            language_info: { name: 'rust' },
        },
        cells: [
            {
                cell_type: 'markdown',
                id: 'intro',
                metadata: {},
                source: [
                    '# DubSar IDE — Storage Fragmentation Simulation\n',
                    '\n',
                    'This notebook demonstrates the **7D particle model** of storage fragmentation.\n',
                    '- **Tribe A** (green): tightly-packed, low fragmentation\n',
                    '- **Tribe B** (red): scattered, high fragmentation\n',
                    '- **Free blocks** (yellow): unassigned, chaotic\n',
                ],
            },
            {
                cell_type: 'code',
                id: 'init',
                metadata: {},
                source: ['%%dubsar init-storage 100000'],
                outputs: [],
                execution_count: null,
            },
            {
                cell_type: 'code',
                id: 'step',
                metadata: {},
                source: ['%%dubsar step 50'],
                outputs: [],
                execution_count: null,
            },
            {
                cell_type: 'code',
                id: 'render',
                metadata: {},
                source: ['%%dubsar render'],
                outputs: [],
                execution_count: null,
            },
            {
                cell_type: 'code',
                id: 'stats',
                metadata: {},
                source: ['%%dubsar stats'],
                outputs: [],
                execution_count: null,
            },
            {
                cell_type: 'markdown',
                id: 'gpu-note',
                metadata: {},
                source: [
                    '## Phase 2: GPU Path (bare-metal Fedora 44)\n',
                    '\n',
                    'On bare-metal with GPU passthrough, replace the software renderer:\n',
                    '```\n',
                    '%%glsl\n',
                    '#version 450\n',
                    'layout(local_size_x = 256) in;\n',
                    '// ... compute shader for 1B particles\n',
                    '```\n',
                ],
            },
        ],
    };
}
