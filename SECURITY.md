# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in agcu, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Email the maintainers directly (see Cargo.toml for contact)
3. Include steps to reproduce and potential impact

We will respond within 48 hours and issue a fix as soon as possible.

## Scope

agcu requires Accessibility permissions to function. This is an OS-level
security boundary — the user must explicitly grant access. agcu does not:

- Transmit data over the network
- Store credentials or sensitive data (refs cache contains only element names/roles)
- Execute arbitrary code from remote sources
- Bypass OS security boundaries

## Permissions

agcu uses the following OS permissions:

| Permission    | Why                                               | Platform |
| ------------- | ------------------------------------------------- | -------- |
| Accessibility | Read/write the accessibility tree, simulate input | macOS    |
| AT-SPI2       | Read/write the accessibility tree                 | Linux    |
| UI Automation | Read/write the accessibility tree, simulate input | Windows  |
