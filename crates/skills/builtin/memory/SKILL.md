---
name: memory
description: "Two-layer memory system: long-term facts (MEMORY.md) and searchable history log (HISTORY.md)."
always: true
---

# Memory

## Structure

- `SOUL.md` -- Bot personality and communication style. **Managed by consolidation.** Do NOT edit.
- `USER.md` -- User profile and preferences. **Managed by consolidation.** Do NOT edit.
- `memory/MEMORY.md` -- Long-term facts (project context, important events). **Managed by consolidation.** Do NOT edit.
- `memory/HISTORY.md` -- Append-only plain text log, not loaded into context. Each entry starts with `[YYYY-MM-DD HH:MM]`. Use `exec` with `grep` to search it.

## Search Past Events

`memory/HISTORY.md` is a plain text file. Each line follows the format:

```
[YYYY-MM-DD HH:MM] ROLE: summary of events/decisions/topics
```

Use the `exec` tool with `grep` to search:

- Search by keyword: `grep -i "keyword" memory/HISTORY.md`
- Search by date: `grep "^\[2026-02-27" memory/HISTORY.md`
- Search by date range (month): `grep "^\[2026-02" memory/HISTORY.md`
- Count matches: `grep -c "keyword" memory/HISTORY.md`
- Show context: `grep -i -B1 -A1 "keyword" memory/HISTORY.md`

## Important

- **Do NOT edit SOUL.md, USER.md, or memory/MEMORY.md.** They are automatically managed by the consolidation process.
- If you notice outdated information, it will be corrected during the next consolidation cycle.
