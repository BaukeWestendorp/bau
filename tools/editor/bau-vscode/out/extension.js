"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = void 0;
const vscode = require("vscode");
const node_1 = require("vscode-languageclient/node");
function activate(context) {
    let executableLocation = vscode.workspace.getConfiguration('bau').get('languageServerExecutablePath');
    if (!executableLocation)
        executableLocation = 'bau-language-server';
    const serverOptions = {
        command: executableLocation,
        args: [],
    };
    const clientOptions = {
        documentSelector: [
            { language: 'bau' },
        ],
    };
    const client = new node_1.LanguageClient('bau-language-server', serverOptions, clientOptions);
    client.start();
}
exports.activate = activate;
//# sourceMappingURL=extension.js.map