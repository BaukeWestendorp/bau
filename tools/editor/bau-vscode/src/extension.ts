import * as vscode from 'vscode';
import { LanguageClientOptions } from 'vscode-languageclient';
import { LanguageClient } from 'vscode-languageclient/node';

export function activate(context: vscode.ExtensionContext) {
    if (!vscode.workspace.getConfiguration('bau').get<boolean>('disableLanguageServer')) {
        startLanguageServer(context);
    }
}

function startLanguageServer(context: vscode.ExtensionContext) {
    let executableLocation = vscode.workspace.getConfiguration('bau').get<string>('languageServerExecutablePath');
    if (!executableLocation) executableLocation = 'bau-language-server';

    const serverOptions = {
        command: executableLocation,
        args: [],
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { language: 'bau' },
        ],
    };

    const client = new LanguageClient('bau-language-server', serverOptions, clientOptions);
    client.start();
}