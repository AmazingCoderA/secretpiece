# Architecture Decision Records

Each significant decision gets a numbered, immutable ADR. Changing a decision
means writing a new ADR that supersedes the old one.

Naming: `NNNN-short-title.md` (for example `0001-container-format.md`).

Template:

```markdown
# NNNN. Title

- Status: proposed | accepted | superseded by NNNN
- Date: YYYY-MM-DD
- Deciders: names

## Context

What problem are we solving? What constraints matter (security, portability, UX)?

## Decision

What we decided, precisely.

## Consequences

Positive, negative, and open questions. Security impact must be explicit.
```

Planned ADRs:

- 0001 — container file instead of hidden partition;
- 0002 — GUI stack: egui + eframe on desktop, Compose on Android;
- 0003 — SPV1 on-disk layout and generation rules;
- 0004 — key hierarchy and slot model;
- 0005 — deniable vs protected hidden volume;
- 0006 — metadata encoding and index limits.
