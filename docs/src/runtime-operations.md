# Runtime Operations

Berth manages server processes through a local runtime state model.

## Lifecycle

```bash
berth start github
berth stop github
berth restart github
```

Stop behavior is graceful-first: Berth sends a normal termination signal, waits briefly for exit,
and escalates to force termination only when needed.

## Status and Logs

```bash
berth status
berth status --health-check
berth logs github --tail 100
```

Status includes process state and, when available, PID and memory metadata.

The `--health-check` flag adds an MCP protocol health probe column: Berth sends a JSON-RPC `initialize` request to each running server and reports the protocol version and server name.

## Auto-Restart Policy

Config keys:

- `berth.auto-restart` (`true` / `false`)
- `berth.max-restarts` (positive integer)
- `berth.sandbox` (`basic` / `off`)
- `berth.sandbox-network` (`inherit` / `deny-all`)
- `berth.max-memory` (memory limit with K/M/G suffix, e.g. `512M`, `1G`)
- `berth.max-file-descriptors` (positive integer, e.g. `1024`)

When auto-restart is enabled, Berth launches a hidden tokio-backed supervisor process that
monitors crash exits and performs bounded restarts without requiring `berth status` polling.

When sandbox mode is enabled:

- Linux uses `landlock-restrict` for filesystem scope enforcement when available and `setpriv --no-new-privs` for additional hardening
- macOS uses `sandbox-exec` with a generated profile and declared write-path allowances
- Windows uses Job Objects for process isolation
- Other platforms fall back to standard process launch while preserving policy config

## Global Configuration

Berth supports a global config file at `~/.berth/berth.toml` that provides workspace-wide defaults:

```toml
[runtime]
auto_restart = true
max_restarts = 5

[security]
sandbox = "basic"
sandbox_network = "inherit"
```

Per-server config values override global defaults. A missing or malformed global config file is silently ignored.

Example:

```bash
berth config github --set berth.auto-restart=true
berth config github --set berth.max-restarts=3
berth config github --set berth.sandbox=basic
berth config github --set berth.sandbox-network=inherit
```
