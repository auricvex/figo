# Security Policy

## Supported Versions

The following versions of `figo` are currently supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in `figo`, please report it privately via
[GitHub Security Advisories](https://github.com/auricvex/figo/security/advisories/new)
rather than opening a public issue.

We will acknowledge receipt of your report within 72 hours and will work with you
to understand and resolve the issue as quickly as possible.

Please include as much detail as possible in your report, such as:

- A description of the vulnerability
- Steps to reproduce the issue
- Affected versions
- Any potential impact
- Suggested fixes (if any)

## Disclosure Policy

Once a vulnerability has been fixed, we will publish a GitHub Security Advisory
and a corresponding release. We ask that you do not publicly disclose the
details of the vulnerability until we have had a reasonable opportunity to
release a fix.

## Security Best Practices

When using `figo`, keep the following in mind:

- `figo` processes JSON input. Do not pass untrusted input to the CLI without
  validation.
- The clipboard output feature (`--clipboard`) interacts with the system
  clipboard. Ensure you trust the source of any input you copy to the clipboard.
- Report any suspicious behavior or potential security issues promptly.
