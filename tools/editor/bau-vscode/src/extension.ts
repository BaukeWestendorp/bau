import * as vscode from 'vscode';
import { LanguageClientOptions } from 'vscode-languageclient';
import { LanguageClient } from 'vscode-languageclient/node';

export function activate(context: vscode.ExtensionContext) {
    const serverOptions = {
        command: 'bau',
        args: ['language-server'],
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { language: 'bau' },
        ],
    };

    const client = new LanguageClient('bau-language-server', serverOptions, clientOptions);
    client.start();
}