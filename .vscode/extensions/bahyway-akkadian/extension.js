const vscode = require('vscode');

// ═════════════════════════════════════════════════════════════════
//  AAOL Notebook (.akknb) — container serializer ONLY.
//  Maps the .akknb JSON file <-> notebook cells. The AAOL code inside
//  each cell is highlighted by this extension's own "akkadian" grammar
//  (crates/aaol/src/token.rs vocabulary) and, later, analyzed by the
//  pure-Rust AAOL analyzer. No language logic lives here — this is
//  file-format plumbing, the lawful kind (the notebook cell is a
//  sealed request, never a computation — see ADR-019, GL-CSD-001).
//
//  .akknb on disk (human-readable, git-friendly, seal-able):
//  {
//    "akknb": "1.0",
//    "sealedBy": null,
//    "cells": [
//      { "kind": "markdown", "value": "# Title" },
//      { "kind": "akkadian", "value": "tribe example { }" }
//    ]
//  }
// ═════════════════════════════════════════════════════════════════

const enc = new TextEncoder();
const dec = new TextDecoder();

const serializer = {
  async deserializeNotebook(content) {
    let raw = { cells: [] };
    try {
      const text = dec.decode(content);
      if (text.trim().length) raw = JSON.parse(text);
    } catch (e) {
      // corrupt or empty -> start with one akkadian cell rather than throw
      raw = { cells: [{ kind: 'akkadian', value: '' }] };
    }
    const cells = (raw.cells || []).map(c => {
      if (c.kind === 'markdown') {
        return new vscode.NotebookCellData(
          vscode.NotebookCellKind.Markup, c.value || '', 'markdown');
      }
      // default: an AAOL code cell, real language id "akkadian"
      // (this extension's own registered id -- not "akk", which was
      // never sealed as this ecosystem's real language id).
      return new vscode.NotebookCellData(
        vscode.NotebookCellKind.Code, c.value || '', 'akkadian');
    });
    return new vscode.NotebookData(cells);
  },

  async serializeNotebook(data) {
    const cells = data.cells.map(c => ({
      kind: c.kind === vscode.NotebookCellKind.Markup ? 'markdown' : 'akkadian',
      value: c.value,
    }));
    const out = { akknb: '1.0', sealedBy: null, cells };
    return enc.encode(JSON.stringify(out, null, 2));
  },
};

function activate(context) {
  context.subscriptions.push(
    vscode.workspace.registerNotebookSerializer('akknb', serializer,
      { transientOutputs: true })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('akkadian.newNotebook', async () => {
      const data = new vscode.NotebookData([
        new vscode.NotebookCellData(vscode.NotebookCellKind.Markup,
          '# New AAOL Notebook 𒁾\n\nDescribe the analysis below.', 'markdown'),
        new vscode.NotebookCellData(vscode.NotebookCellKind.Code,
          '# AAOL cells hold real .akk/.ak source\ntribe example { }\n', 'akkadian'),
      ]);
      const nb = await vscode.workspace.openNotebookDocument('akknb', data);
      await vscode.window.showNotebookDocument(nb);
    })
  );
}
function deactivate() {}
module.exports = { activate, deactivate };
