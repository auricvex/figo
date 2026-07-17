---
name: ❓ Question / Support
description: Ask a question or request help using figo
title: "[Question] "
labels: ["question", "triage"]
assignees: []
body:
  - type: markdown
    attributes:
      value: |
        Have a question about using figo? We're happy to help! Please provide as much detail as possible.

  - type: textarea
    id: question
    attributes:
      label: Question
      description: What would you like to know?
      placeholder: "How do I center-align text inside a box with the builder API?"
    validations:
      required: true

  - type: textarea
    id: context
    attributes:
      label: Context
      description: Provide any relevant context, such as the command or code you are using.
      render: rust

  - type: textarea
    id: tried
    attributes:
      label: What I've Tried
      description: Describe what you have already tried and what happened.

  - type: checkboxes
    id: checklist
    attributes:
      label: Checklist
      options:
        - label: I have read the README and CLI usage documentation.
          required: false
        - label: I have searched existing issues and discussions for similar questions.
          required: false
