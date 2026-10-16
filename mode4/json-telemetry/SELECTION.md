# Selection contract

## Input

```json
{
  "identity_key": "...",
  "definition_ids": [
    "entity-intelligence.organization.001.legal-name-and-legal-form",
    "entity-intelligence.universal.004.official-website-and-public-references"
  ],
  "include_related": false
}
```

## Output (dispatch-ready)

Only the nodes, relations, and properties that satisfy the listed definition_ids for the identity_key.

No full feed payloads.
No unrelated worker-touched data.

## Rules

- Identity resolution is a separate prior step.
- Definition IDs come from the frozen 240-field catalog.
- Relationship definitions are included only when `include_related` is true or the requirement kind is connection.
- Operator catalog maps definition_id → operator identity (existing binaries stay; selection is added on top).
