---
name: 📚 Documentation Request
description: Report missing, unclear, or incorrect documentation
title: "[Docs] "
labels: ["documentation", "triage"]
assignees: []
body:
  - type: markdown
    attributes:
      value: |
        Help us improve figo's documentation. Let us know what's missing or unclear.

  - type: dropdown
    id: type
    attributes:
      label: Documentation Type
      options:
        - Missing documentation
        - Unclear documentation
        - Incorrect documentation
        - Outdated documentation
        - Other
    validations:
      required: true

  - type: input
    id: location
    attributes:
      label: Location
      description: Where is the documentation issue? (file, URL, or section name)
      placeholder: "README.md / CLI Usage / figo-spec.md"

  - type: textarea
    id: description
    attributes:
      label: Description
      description: Describe the issue in detail. What is missing, unclear, or incorrect?
      placeholder: "The packet diagram JSON schema does not document the `bits` field..."
    validations:
      required: true

  - type: textarea
    id: suggestion
    attributes:
      label: Suggested Improvement
      description: If you have a suggestion for how to fix it, please share.

  - type: textarea
    id: context
    attributes:
      label: Additional Context
      description: Add any other context, screenshots, or examples.
