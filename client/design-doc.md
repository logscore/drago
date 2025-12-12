# Drago – Dynamic DNS Client

## Purpose

Drago is a single self-contained Rust binary that manages Dynamic DNS synchronization for hobbyists.  
It consists of:

- A CLI interface for user control.
- A daemon that runs in the background to periodically sync device IPs with the Drago API.

The code must prioritize:

- Security (no unsafe operations or privilege misuse)
- Simplicity (flat, transparent flow, explicit code)
- Observability (clear status, logs, PID, readable behavior)
- No network activity except explicit sync endpoints.

## Core Concept

One binary, two modes:

- CLI control layer — manages login, daemon start/stop/status.
- Daemon mode — performs the periodic IP fetch and sync to the remote API.

The CLI spawns the daemon by running itself with a daemon command argument.

No other process downloads, bundles, or reaches external binaries.

## Overall Architecture

### Binary entry points

drago login # prompt to securely store API key
drago start # start daemon (forks itself)
drago stop # stops daemon (by PID)
drago status # show daemon state
drago restart # stop + start
drago daemon # internal command: runs sync loop

### Modules

src/
├── main.rs # CLI entrypoint
├── daemon.rs # background loop logic
├── config.rs # config / key mgmt
├── api.rs # networking + sync
└── process.rs # PID mgmt and daemon control

## Core Operations

### CLI

- Use clap for subcommands.
- Each command routes to a dedicated function (no async).
- Write small helper utilities for reading config, PIDs, and logs.
- Use deterministic paths for config and logs.

File paths:
Linux/Mac:
PID file: /tmp/drago.pid
Logs: /tmp/drago.out, /tmp/drago.err
Config: ~/.config/drago/config.json

## Command Definitions

### login

1. Prompt user for API key (no echo input, use rpassword crate).
2. Create ~/.config/drago if missing.
3. Save { "api_key": "<KEY>" } JSON to config file (0600 perms).
4. Print confirmation message.

### start

1. Check if /tmp/drago.pid exists.
   - If yes and process is alive → print "Daemon already running" and exit 0.
2. Spawn a detached process:
   current_exe() arg("daemon")
3. Print "✅ Daemon started."

### stop

1. Read PID from /tmp/drago.pid.
2. Send SIGTERM (libc::kill(pid, 15)).
3. Remove PID file.
4. Print result.

### status

1. Check if PID file exists.
2. Validate process alive using ps -p PID.
3. Print state — "✅ running (PID xxx)" or "❌ not running."
4. Optionally read /tmp/drago.out for recent line with timestamp or "last sync".

### restart

Sequential stop() then start().

### daemon

Invoked internally by the CLI.  
Implements:

- Daemonize current process (use daemonize crate).
- Write PID file.
- Redirect stdout/stderr to log files.
- Enter background loop:
  1. Read API key from config.
  2. Fetch public IP from https://api.ipify.org.
  3. Send to ${DRAGO_API_URL:-https://api.drago.dev}/sync using HTTP POST.
  4. Log result to /tmp/drago.out.
  5. Sleep 300 seconds.
  6. Repeat indefinitely.
- Continue looping until process SIGTERM'd.

All network requests are blocking + retry-safe (no async required).

## Networking

Dependencies: reqwest (blocking) + serde_json.  
Endpoints:

POST /sync
Authorization: Bearer <api_key>
Content-Type: application/json
{
"ip": "<IPv4>",
"timestamp": "<ISO8601 UTC>"
}

Expect 200 OK → success.

On failure:

- Log message, retry after sleep.

## Config Handling

config.json structure:
{
"api_key": "YOUR_API_KEY"
}

Implementation:

- Use serde to read/write.
- 0600 file permission (std::fs::set_permissions).

## Logging

- All stdout/stderr redirected by daemonize to:
  - /tmp/drago.out for normal logs.
  - /tmp/drago.err for errors.
- Always log:
  - start time
  - IP fetch result
  - sync result (HTTP response)
  - timestamp of each iteration

## Security Controls

1. Never run as root except when setting up /etc/drago configs (optional future).
2. No hardcoded credentials.
3. All communication over HTTPS.
4. File permissions restricted to the current user.
5. Fail safe → in network errors, skip update, retry next iteration.

## Dependencies Required

[dependencies]
clap = { version = "4", features = ["derive"] }
reqwest = { version = "0.12", features = ["blocking", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
daemonize = "0.5"
rpassword = "7"
dirs = "5"
chrono = "0.4"
libc = "0.2"

## Process State Management

- /tmp/drago.pid written on daemon start.
- On exit/SIGTERM, remove the PID file.
- If stale PID found, CLI ignores and overwrites it.

PID validation method:

- kill(pid, 0) to check existence before killing.

## Success Criteria

The AI implementation is correct when:

1. Running drago start spawns a persistent daemon that updates /tmp/drago.out every 5 minutes.
2. Running drago status correctly reflects daemon liveness.
3. Running drago stop stops the daemon and cleans up state.
4. IP and timestamp are successfully posted to the API endpoint.
5. Config and logs are human-readable and secure.

## Non-Goals (MVP)

- No GUI or tray interface.
- No automatic service registration (launchd, systemd).
- No parallel tasks or async threads.
- No auto-updater (manual updates only).

## Implementation Summary:

The entire Drago product is a single Rust binary that:

- Provides a simple CLI surface.
- Manages its own background daemon via subcommand and PID.
- Sends IP syncs securely and periodically to a remote DNS API.
- Requires no internet fetches or runtime dependencies beyond its own code.
