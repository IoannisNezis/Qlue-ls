# Security Policy

## Overview

**Qlue-ls** is a language server that runs entirely on the user's machine:

- As a native CLI binary
- Or compiled to WebAssembly (WASM) and executed in the browser

Qlue-ls does **not** operate any hosted service and does **not** transmit user data to external servers. All analysis happens locally in the user's environment.

Because Qlue-ls analyzes user-provided source files and may be integrated into editors, its security model assumes it may process **untrusted project content**.

---

## Supported Versions

Security fixes are provided for:

| Version        | Supported |
|----------------|-----------|
| Latest release | ✅        |
| `main` branch  | ✅        |
| Older releases | ❌        |

Please upgrade to the latest version before reporting vulnerabilities affecting outdated versions.

---

## Threat Model

Qlue-ls:

- Parses and analyzes user-controlled source files
- Runs with the privileges of the invoking user
- May be embedded in editors or browser environments
- May operate on large or intentionally malformed inputs

### In Scope

The following are considered security issues:

- Arbitrary code execution triggered by malicious input
- Command injection
- Arbitrary file system access outside expected project boundaries
- Path traversal vulnerabilities
- Unsafe deserialization
- Memory safety vulnerabilities (especially in native builds)
- Denial-of-service issues (e.g. parser crashes, unbounded memory growth, CPU exhaustion from crafted input)
- WASM sandbox escapes or unintended host access

### Out of Scope

The following are **not** considered security vulnerabilities:

- Incorrect diagnostics
- Specification non-compliance
- Editor integration bugs
- Crashes caused by unsupported platforms
- Performance issues without a clear security impact

If you are unsure whether something qualifies, please report it privately.

---

## Reporting a Vulnerability

Please **do not open a public GitHub issue** for security vulnerabilities.

Instead, use one of the following:

### Preferred: GitHub Security Advisory (Private Disclosure)

Report via:
https://github.com/IoannisNezis/Qlue-ls/security/advisories

### Alternative: Direct Contact

If you prefer private email disclosure, contact:

Ioannis Nezis  
(please add your preferred contact email here)

---

## What to Include

To help resolve the issue efficiently, please include:

- A clear description of the vulnerability
- Affected version(s)
- Reproduction steps
- A minimal proof of concept
- Expected vs actual behavior
- Potential impact

If the issue involves memory corruption or unsafe behavior, include:

- Platform (OS, architecture)
- Build mode (debug/release)
- Whether it occurs in native or WASM build

---

## Response Timeline

We aim to:

- Acknowledge reports within 72 hours
- Provide an initial assessment within 7 days
- Release a fix as soon as reasonably possible depending on severity

Confirmed vulnerabilities will be:

- Patched
- Documented in a security advisory
- Credited to the reporter (unless anonymity is requested)

---

## Security Best Practices for Users

Because Qlue-ls runs locally:

- Do not run it as root/administrator unless necessary
- Avoid analyzing untrusted repositories with elevated privileges
- Keep Qlue-ls up to date
- In browser/WASM environments, rely on the browser sandbox and avoid granting unnecessary host capabilities

---

## Dependency Policy

Qlue-ls may depend on third-party crates or libraries. If a vulnerability is discovered in a dependency:

- It will be patched by updating to a secure version as soon as possible
- A security advisory will be published if required

---

## Responsible Disclosure

We follow responsible disclosure practices.  
Details of vulnerabilities will not be made public until a fix is available.

Thank you for helping improve the security of Qlue-ls.
