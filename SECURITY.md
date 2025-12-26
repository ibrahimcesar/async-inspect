# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

As async-inspect is currently in pre-1.0 development, only the latest minor version receives security updates. We recommend always using the latest version.

## Security Model

async-inspect is a **debugging and observability tool** designed for development and production monitoring. It does not:

- Handle authentication or authorization
- Process untrusted user input directly
- Make network requests
- Execute arbitrary code

### Production Considerations

When using async-inspect in production:

1. **Sampling**: Enable sampling to reduce overhead and data exposure
   ```rust
   Config::global().set_sampling_rate(100); // Track 1 in 100 tasks
   ```

2. **Ring Buffer**: Use bounded memory to prevent unbounded growth
   ```rust
   Timeline::with_ring_buffer(10_000);
   ```

3. **Dashboard Access**: The web dashboard (if enabled) binds to localhost by default. If exposing externally, implement appropriate access controls.

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

**For security vulnerabilities, please DO NOT open a public GitHub issue.**

Instead, please report security issues via one of these methods:

1. **GitHub Security Advisories** (Preferred)
   - Go to the [Security tab](https://github.com/ibrahimcesar/async-inspect/security/advisories)
   - Click "Report a vulnerability"
   - Provide details about the vulnerability

2. **Email**
   - Send details to: [async-inspect@ibrahimcesar.com](mailto:async-inspect@ibrahimcesar.com)
   - Use subject line: `[SECURITY] async-inspect vulnerability report`
   - Include PGP encryption if desired (key available on request)

### What to Include

Please provide:

- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Potential impact
- Any suggested fixes (optional)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Assessment**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: Within 24-48 hours
  - High: Within 7 days
  - Medium: Within 30 days
  - Low: Next scheduled release

### Disclosure Policy

- We follow [coordinated disclosure](https://en.wikipedia.org/wiki/Coordinated_vulnerability_disclosure)
- We will credit reporters in the security advisory (unless anonymity is requested)
- We aim to release fixes before public disclosure
- We will notify you when the fix is released

## Security Best Practices for Users

### Development

```rust
// Full tracking for debugging
Config::global().debug_mode();
```

### Production

```rust
use async_inspect::config::Config;

// Minimal overhead configuration
Config::global().production_mode();

// Or manually configure:
Config::global().set_sampling_rate(100);
Config::global().set_track_awaits(false);
Config::global().enable_adaptive_sampling();
```

### Sensitive Data

async-inspect captures task names and metadata. Avoid including sensitive information in:

- Task names
- Custom event data
- Log messages that flow through the timeline

```rust
// Avoid this:
inspector.register_task(format!("process_user_{}", user_email)); // Leaks PII

// Prefer this:
inspector.register_task(format!("process_user_{}", user_id)); // Use opaque IDs
```

## Dependencies

We regularly audit dependencies using:

```bash
cargo audit
```

Current dependency security status is tracked in our CI pipeline. Known advisories are documented in release notes.

### Optional Features

Security-sensitive optional dependencies:

| Feature | Dependencies | Notes |
|---------|--------------|-------|
| `dashboard` | `axum`, `tower-http` | HTTP server |
| `lsp` | `tower-lsp` | IPC/TCP server |

Disable features you don't need:
```toml
[dependencies]
async-inspect = { version = "0.1", default-features = false, features = ["tokio", "cli"] }
```

## Security Changelog

### v0.1.0
- Initial release
- No known security vulnerabilities

---

Thank you for helping keep async-inspect and its users safe!
