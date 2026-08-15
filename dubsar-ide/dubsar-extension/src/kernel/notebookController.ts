import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as path from 'path';

/**
 * Connects VSCodium's notebook API to the dubsar-kernel process.
 * Each .dubsar notebook gets its own kernel instance via the Jupyter ZMQ protocol.
 */
export class DubSarNotebookController implements vscode.Disposable {
    private readonly _controller: vscode.NotebookController;
    private readonly _disposables: vscode.Disposable[] = [];

    constructor(private readonly context: vscode.ExtensionContext) {
        this._controller = vscode.notebooks.createNotebookController(
            'dubsar-kernel-controller',
            'dubsar',
            'DubSar (Rust + Vulkan)'
        );

        this._controller.supportedLanguages = ['rust', 'dubsar'];
        this._controller.supportsExecutionOrder = true;
        this._controller.description = '7D particle simulation kernel';
        this._controller.detail = 'Software backend (Hyper-V VM) · GPU backend (Fedora 44 bare-metal)';

        this._controller.executeHandler = this._execute.bind(this);
        this._disposables.push(this._controller);
    }

    private async _execute(
        cells: vscode.NotebookCell[],
        _notebook: vscode.NotebookDocument,
        _controller: vscode.NotebookController
    ): Promise<void> {
        for (const cell of cells) {
            await this._executeCell(cell);
        }
    }

    private async _executeCell(cell: vscode.NotebookCell): Promise<void> {
        const execution = this._controller.createNotebookCellExecution(cell);
        execution.start(Date.now());

        const code = cell.document.getText().trim();

        try {
            const output = await this._runWithKernel(code);
            await execution.replaceOutput([output]);
            execution.end(true, Date.now());
        } catch (err) {
            const errMsg = err instanceof Error ? err.message : String(err);
            await execution.replaceOutput([
                new vscode.NotebookCellOutput([
                    vscode.NotebookCellOutputItem.text(
                        `DubSar kernel error: ${errMsg}`,
                        'text/plain'
                    ),
                ]),
            ]);
            execution.end(false, Date.now());
        }
    }

    private async _runWithKernel(code: string): Promise<vscode.NotebookCellOutput> {
        const config = vscode.workspace.getConfiguration('dubsar');
        const kernelPath = config.get<string>('kernelPath', 'dubsar-kernel');

        return new Promise((resolve, reject) => {
            const proc = cp.spawn(kernelPath, ['--eval'], { stdio: ['pipe', 'pipe', 'pipe'] });

            let stdout = '';
            let stderr = '';

            proc.stdin.write(code + '\n');
            proc.stdin.end();

            proc.stdout.on('data', (d: Buffer) => { stdout += d.toString(); });
            proc.stderr.on('data', (d: Buffer) => { stderr += d.toString(); });

            proc.on('close', (code_) => {
                if (code_ !== 0 && stdout === '') {
                    reject(new Error(stderr || `kernel exited with code ${code_}`));
                    return;
                }

                // Try to parse as JSON (stats output), otherwise treat as plain text
                try {
                    const json = JSON.parse(stdout.trim());
                    resolve(new vscode.NotebookCellOutput([
                        vscode.NotebookCellOutputItem.json(json, 'application/json'),
                        vscode.NotebookCellOutputItem.text(JSON.stringify(json, null, 2), 'text/plain'),
                    ]));
                } catch {
                    // Check for base64 PNG prefix
                    const lines = stdout.trim().split('\n');
                    const pngLine = lines.find(l => l.startsWith('PNG:'));
                    if (pngLine) {
                        const b64 = pngLine.slice(4);
                        resolve(new vscode.NotebookCellOutput([
                            vscode.NotebookCellOutputItem.create(
                                Buffer.from(b64, 'base64'),
                                'image/png'
                            ),
                        ]));
                    } else {
                        resolve(new vscode.NotebookCellOutput([
                            vscode.NotebookCellOutputItem.text(stdout || stderr, 'text/plain'),
                        ]));
                    }
                }
            });

            proc.on('error', reject);
        });
    }

    dispose(): void {
        this._disposables.forEach(d => d.dispose());
    }
}
