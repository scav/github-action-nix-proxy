# github-action-nix-proxy

A very simple HTTP/1.1 forward proxy for Nix talking to S3 over kgateway.

It acts as a substituter for any service running on the host.

## Why
Nix does not reliably support setting query parameters to all requests and does not
necessarily read environment variables at all.

This proxy attaches `Authorization: Bearer $token` to the request. It also respects all
query parameters and paths added by nix.

## How
Set the substituters = ["http://127.0.0.1:8300"] and it will behave just like a binary cache.

## Running
The proxy is configured using environment variables.

- `NIXFWD_PORT` sets the proxy port
- `NIXFWD_URL` sets the proxy target
- `NIXFWD_HOST` sets the HOST Header
- `NIXFWD_AUTHORIZATION_TOKEN` sets the AUTHORIZATION token (defaults to Bearer)
- `NIXFWD_AUTHORIZATION_TOKEN_NAME` sets the AUTHORIZATION token (defaults to Bearer)

In a GitHub Action this can run as a background service after nix has been
installed in the runner:
```yaml
  - name: Run nix-proxy
    id: nix-proxy
    run: |
      nix run
    background: true
```

## Development
- `direnv` allow to initialise the development environment
- `bacon` to run the code, check `bacon.toml` for commands

In `./.cargo/config.toml` a minimum publish age is added.
This will be stabilized in a later version of Rust.

To check for security issues and other problems with the dependency graph run
- `cargo audit`
- `cargo deny check` (licenses not currently handled properly)
