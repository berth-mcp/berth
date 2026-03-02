# Client Integration

Berth can write MCP client configuration entries for installed servers.

## Supported Clients

- Claude Desktop (`claude-desktop`)
- Cursor (`cursor`)
- Windsurf (`windsurf`)
- Continue (`continue`)
- VS Code (`vscode`)

## Commands

```bash
berth link claude-desktop
berth link cursor
berth link windsurf
berth link continue
berth link vscode

berth unlink claude-desktop
```

## Watch Mode

Use `--watch` to keep the client config in sync as servers are installed or removed:

```bash
berth link claude-desktop --watch
```

Berth polls the servers directory every 2 seconds and re-generates the client config when changes are detected. Press Ctrl+C to stop.

## Behavior

- validates required server config before linking
- writes/updates the client MCP config file
- creates a backup before modifying existing client config
- applies env permission filtering to linked server entries
