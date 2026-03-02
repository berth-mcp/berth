# Berth Documentation

> The safe runtime & package manager for MCP servers.

Berth is a single-binary CLI that lets you discover, install, configure, run, and secure [MCP](https://modelcontextprotocol.io) servers — replacing manual JSON config editing with a clean developer experience and a real security layer.

## Quick Start

```bash
cargo install berth

berth search github
berth install github
berth config github --set token=$GITHUB_TOKEN
berth start github
berth link claude-desktop
```

## What's Covered

| Section | Topics |
|---------|--------|
| [Installation](installation.md) | Binary installer, Homebrew, building from source |
| [CLI Reference](cli-reference.md) | All commands, flags, and registry API endpoints |
| [Security Model](security-model.md) | Permissions, audit trail, sandbox, org policy, encrypted secrets |
| [Runtime Operations](runtime-operations.md) | Start/stop/restart, health checks, auto-restart, resource limits, global config |
| [Client Integration](client-integration.md) | Claude Desktop, Cursor, Windsurf, Continue, VS Code, watch mode |
| [Team Workflows](team-workflows.md) | Config export/import, org policy, analytics |

## Install Methods

```bash
# Cargo
cargo install berth

# npm
npx @berth/cli search github

# Shell script
curl -fsSL https://raw.githubusercontent.com/berth-mcp/berth/main/install.sh | sh

# Homebrew (source)
brew install --HEAD ./Formula/berth.rb
```

## Links

- [GitHub Repository](https://github.com/berth-mcp/berth)
- [crates.io](https://crates.io/crates/berth)
- [npm](https://www.npmjs.com/package/@berth/cli)
- [MCP Protocol](https://modelcontextprotocol.io)
