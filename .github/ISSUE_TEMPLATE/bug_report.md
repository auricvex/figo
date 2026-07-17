---
name: 🐛 Bug Report
description: Report a bug to help us improve figo
title: "[Bug] "
labels: ["bug", "triage"]
assignees: []
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to report a bug! Please fill out the sections below so we can reproduce and fix the issue quickly.

  - type: textarea
    id: summary
    attributes:
      label: Bug Summary
      description: A clear and concise description of what the bug is.
      placeholder: "figo box crashes when width is set to 0..."
    validations:
      required: true

  - type: textarea
    id: reproduction
    attributes:
      label: Steps to Reproduce
      description: Provide a minimal set of steps that consistently reproduces the bug.
      placeholder: |
        1. Run `figo box '{"width":0,"content":"x"}'`
        2. Observe the panic
    validations:
      required: true

  - type: textarea
    id: input
    attributes:
      label: Input / Command
      description: Paste the exact JSON input or CLI command you used.
      render: json
      placeholder: |
        {
          "width": 0,
          "content": "x"
        }

  - type: textarea
    id: expected
    attributes:
      label: Expected Behavior
      description: Describe what you expected to happen.
      placeholder: "figo should print an empty box or a graceful error..."
    validations:
      required: true

  - type: textarea
    id: actual
    attributes:
      label: Actual Behavior
      description: Paste the actual output, stack trace, or error message.
      render: text
    validations:
      required: true

  - type: dropdown
    id: os
    attributes:
      label: Operating System
      options:
        - macOS
        - Linux
        - Windows
        - Other
    validations:
      required: true

  - type: input
    id: version
    attributes:
      label: figo Version
      description: Output of `figo --version`
      placeholder: "0.1.0"

  - type: input
    id: rust
    attributes:
      label: Rust Version
      description: Output of `rustc --version`
      placeholder: "rustc 1.85.0"

  - type: textarea
    id: context
    attributes:
      label: Additional Context
      description: Add any other context, screenshots, or workarounds here.
