---
name: ✨ Feature Request
description: Suggest a new feature or enhancement for figo
title: "[Feature] "
labels: ["enhancement", "triage"]
assignees: []
body:
  - type: markdown
    attributes:
      value: |
        Thanks for sharing your idea! Please help us understand the problem you're trying to solve and the proposed solution.

  - type: textarea
    id: problem
    attributes:
      label: Problem Statement
      description: What problem are you trying to solve? Why is it important?
      placeholder: "I often need to generate diagrams from YAML input, but figo only accepts JSON..."
    validations:
      required: true

  - type: textarea
    id: solution
    attributes:
      label: Proposed Solution
      description: Describe the feature or enhancement you have in mind. How would it work from both CLI and library perspectives?
      placeholder: "Add a `--format yaml` flag and a corresponding `from_yaml` builder method..."
    validations:
      required: true

  - type: textarea
    id: use_cases
    attributes:
      label: Use Cases
      description: Describe specific scenarios where this feature would be valuable.
      placeholder: |
        - Generating diagrams from existing YAML configuration files
        - Reducing JSON escaping noise in hand-written inputs
    validations:
      required: true

  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives Considered
      description: Have you considered any alternative approaches or workarounds?

  - type: checkboxes
    id: principles
    attributes:
      label: Design Principles
      description: figo follows KISS and YAGNI principles. Please confirm you have read them.
      options:
        - label: I have read the project's design principles and believe this feature aligns with them.
          required: true

  - type: textarea
    id: context
    attributes:
      label: Additional Context
      description: Add links to similar features in other tools, mockups, or examples.
