# github-action-nix-proxy

A very simple HTTP/1.1 forward proxy for Nix talking to S3 over kgateway.

It acts as a substituter for any service running on the host.

## Why
Nix does not reliably support setting query parameters to all requests and does not
necessarily read environment variables at all.

Append a query parameter to all requests `x-use-new-ingress=1` to make
sure we hit the new ingress. This ingress supports mapping `Authorization: Basic token`
to `Authorization Bearer` and validate the incoming token from a GitHub Action.

## How
Set the substituters = ["http://127.0.0.1:8300"] and it will behave just like a binary cache.

## Running
The proxy is configured using evironment variables.

- `NIXFWD_PORT` sets the proxy port
- `NIXFWD_TARGET` sets the proxy target (e.g. S3 bucket)


## Development
- `direnv` allow to initialise the development environemtn
- `bacon` to run the code, check `bacon.toml` for commands

In `./.cargo/config.toml` a minimum publish age is added.
This will be stabilized in a later version of Rust.

To check for security issues and other problems with the dependency graph run
- `cargo audit`
- `cargo deny check` (licenses not currently handled properly)
