# Claude Operating Manual - mobile-apps KB

> Read this file before doing anything in this vault.
> This is the single source of truth for how Claude operates here.

---

## Section 0 - AI-First Vault Rule (read first, applies to every note)

This vault is designed for **future-Claude** to read and reason over, not for human review. The owner rarely reads notes directly - they call Claude to retrieve, synthesize, and connect dots across the lifetime of the mobile-apps workspace.

**Every note Claude writes to this vault must follow these rules:**

1. **Self-contained context** - Each note must explain itself. Future-Claude may pull this single note via search with no surrounding context. Don't rely on backlinks alone for meaning.
2. **"For future Claude" preamble** - Every note begins with a 2-3 sentence summary in plain English under a `## For future Claude` header (immediately after the frontmatter) so Claude can decide relevance in 10 seconds before parsing the rest.
3. **Rich, consistent frontmatter** - Filterable metadata (`type`, `date`, `tags`, `ai-first: true`, plus type-specific fields). Every note has machine-readable frontmatter.
4. **Recency markers per claim** - When stating external facts, attach the date: "Apple Foundation Models GA on iOS 26 (as of 2026-05)" so future-Claude knows what to verify before trusting.
5. **Sources preserved verbatim** - Every external claim has its source URL inline so it can be re-verified or refreshed.
6. **Cross-links are mandatory** - Every person, project, idea, decision, or concept referenced uses `[[wikilinks]]` so the graph is traversable.
7. **Confidence levels** - Where applicable, mark claims as `stated | high | medium | speculation` so future-Claude knows what to trust vs verify.

This rule applies to all `/obsidian-*` and `/research*` commands, all scheduled agents, and any direct vault writes.

---

## Section 0.5 - Verify Live State Before Acting

Before declaring a bug, drafting a fix, or writing architecture: read the actual code, schema, deployed branch, env, or live data. Speculation from stale context burns hours and produces drafts that contradict reality.

Specific cues:
- Read the schema or types in `crates/mobile-core/` before declaring a contract bug
- `git fetch origin` and read the deployed branch, not local `main`
- Grep the live file before any anchor-based patch
- Fetch live time, dates, SDK versions (never infer from training data)
- Verify env vars in the running process before blaming code
- Mock tests miss schema drift: read one real payload before declaring "done"

---

## Vault Identity

- **Owner:** Kenneth (Karl) Pernyer
- **Primary purpose:** Knowledge base for the `mobile-apps` workspace - native iOS/Android shells, shared Rust core, AI orchestration, ADRs, and product candidates (Quorum Sense, Wolfgang Chat, Inkling Notes)
- **Workspace root:** `/Users/kpernyer/dev/reflective/mobile-apps/`
- **Style:** Wiki-style (LLM-first) - flat folders under `wiki/`, immutable sources under `raw/`
- **Last updated:** 2026-06-06

---

## Folder Map

Top-level is kept lean - only three markdown files live at the root. Everything else is in subfolders.

| Folder | Purpose |
|---|---|
| `_CLAUDE.md` | This file - operating manual |
| `index.md` | Catalog of every note in the vault |
| `log.md` | Pointer to `Logs/` - daily vault operations log |
| `Logs/` | One file per day: `YYYY-MM-DD.md`. Vault operation history (init, ingests, restructures) |
| `Bases/` | Obsidian Bases (`.base` files) for Projects, People, Tasks, Daily |
| `raw/` | **Immutable.** Original sources Claude reads but never modifies |
| `raw/articles/` | Clipped articles, blog posts, web pages |
| `raw/transcripts/` | Meeting notes, podcast / YouTube transcripts |
| `raw/pdfs/` | PDFs, reports, papers |
| `raw/videos/` | YouTube metadata + transcripts |
| `wiki/` | Claude's workspace - the only place Claude writes derived knowledge |
| `wiki/entities/` | People, companies, tools (flat, one file per entity) |
| `wiki/concepts/` | Ideas, frameworks, methodologies (e.g. UniFFI bridge, on-device AI routing) |
| `wiki/projects/` | Project notes - Quorum Sense, Wolfgang Chat, Inkling Notes, mobile-core |
| `wiki/daily/` | Daily notes (one per day) |
| `wiki/logs/` | Dev / work session logs - dated, project-tagged |
| `wiki/reviews/` | Weekly / monthly reviews |
| `wiki/tasks/` | Standalone task notes (linked from boards) |
| `wiki/decisions/` | ADRs not already captured in `../docs/adr/` |
| `boards/` | Kanban boards |
| `templates/` | Note templates (do not modify during normal operations) |
| `_trash/` | Soft-deleted notes |

---

## Key Files

- **This manual:** `_CLAUDE.md`
- **Catalog:** `[[index]]` - read FIRST before searching; cheaper than grep
- **Operations log pointer:** `[[log]]` - points at `Logs/YYYY-MM-DD.md`
- **Workspace AGENTS.md:** `../AGENTS.md` (mobile-apps operating policy)
- **Workspace README:** `../README.md`
- **Parent KB (legacy):** `../../KB/` - the older Reflective-wide vault. This vault is scoped to `mobile-apps` only.

---

## Active Context

> Update at the start of each major project or focus period.

**Workspace:** `mobile-apps` - native mobile shells + shared Rust core
**First product candidate:** Quorum Sense (Marquee) - live signal capture, voice/photo/text input, offline participant notes
**Studio candidates:** Wolfgang Chat (research companion), Inkling Notes (local-first capture)
**Current branch:** `main` (single-developer policy - see `../AGENTS.md`)
**Build commands:** `just check`, `just ci`, `cargo test --workspace --locked`

---

## Auto-Save Rules

Claude should auto-save the following **without asking**:
- Decisions made in conversation about mobile architecture, AI routing, FFI surfaces -> relevant project note + daily note
- New people mentioned -> `wiki/entities/` (create stub if needed)
- Tasks assigned or committed to -> board + `wiki/tasks/` note
- Dev work done -> `wiki/logs/` + project note + daily note
- ADRs and design pivots -> `wiki/decisions/` (or link to `../docs/adr/`)
- Completed tasks -> move on board to Done

Claude should **ask before saving**:
- Anything touching personal financial data
- Anything that involves deleting or archiving an existing note

---

## Naming Conventions

- Daily notes: `YYYY-MM-DD.md` in `wiki/daily/`
- Dev logs: `YYYY-MM-DD - Description.md` in `wiki/logs/`
- Entities (people / companies / tools): full name, flat - `Jane Smith.md`, `Apple.md`, `UniFFI.md`
- Concepts: descriptive title - `On-Device AI Routing.md`
- Projects: proper name - `Quorum Sense.md`
- Sources in raw: `YYYY-MM-DD - Title.md`
- ADRs: `ADR-YYYY-MM-DD - Title.md` in `wiki/decisions/`
- Archive prefix: `_archived_`

---

## Frontmatter Requirements

Every note minimum:
```yaml
---
date: YYYY-MM-DD
type: <note-type>
tags:
  - <note-type>
ai-first: true
---
```

Note types: `daily` | `project` | `task` | `person` | `entity` | `concept` | `devlog` | `decision` | `adr` | `review` | `source` | `research` | `meeting`

---

## Kanban Convention

Columns: `Backlog` - `This Week` - `In Progress` - `Waiting On` - `Done`
Priority: `critical` | `important` | `low`

Item format:
```
- [ ] critical **Title** - due YYYY-MM-DD
    Description. [[Projects/Project Name]] [[Entities/Person]]
```

Completed:
```
- [x] ~~critical **Title**~~ done YYYY-MM-DD
```

---

## Propagation Rules

| Event | Also update |
|---|---|
| New project | Board (Backlog) + today's daily note |
| Task done | Board (Done, strikethrough) + project note + daily note |
| Dev session | `wiki/logs/` + project note (Recent Activity) + daily note |
| Person interaction | Daily note + their `wiki/entities/` note |
| Decision made | Project note (Key Decisions) + daily note |
| ADR | `wiki/decisions/` + project note + daily note |
| Source ingested | `raw/<type>/` + index.md + derived notes in `wiki/` |

---

## Projects Currently Active

> Keep this list current. Claude uses it to route context correctly.

- `[[wiki/projects/Quorum Sense]]` - first Marquee mobile candidate (TBD - stub on creation)
- `[[wiki/projects/Wolfgang Chat]]` - Studio research companion (TBD)
- `[[wiki/projects/Inkling Notes]]` - Studio local-first capture (TBD)
- `[[wiki/projects/Mobile Core]]` - shared Rust crate (TBD)

---

## Do Not Touch

- `templates/` - never modify templates during normal vault operations
- `raw/` - immutable. Claude reads, never writes
- `archive/legacy-placeholders/` in the workspace (out of scope for this vault)

---

*This file was generated by the obsidian-second-brain skill via `/obsidian-init`.*
*Regenerate with: "Claude, update my _CLAUDE.md"*
