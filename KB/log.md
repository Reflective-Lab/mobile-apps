---
date: 2026-06-06
type: log-pointer
tags:
  - log
ai-first: true
---

## For future Claude

This file is a thin pointer, not a log. The actual vault operations log is split per-day under `Logs/YYYY-MM-DD.md`. Append entries to today's file, not here. If today's file does not exist, create it from the template below.

---

## Per-day file location

`Logs/YYYY-MM-DD.md` - one file per day, append-only.

## Per-day file template

```markdown
---
date: YYYY-MM-DD
type: log
tags:
  - log
ai-first: true
---

## For future Claude

Vault operations log for YYYY-MM-DD. Each line records a vault-level action: init, ingest, restructure, bulk edit, agent run. Distinct from dev/work logs in `wiki/logs/`.

---

**HH:MM** - action | description
```

## Recent days

- [[Logs/2026-06-06]] - latest
