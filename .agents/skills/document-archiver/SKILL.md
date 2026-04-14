---
name: document-archiver
description: Archives grown documents with immutable chain references. Use when project documents exceed token thresholds, at milestones, or on user command. Never overwrites existing archives — always creates new files with chain links.
---

# Document Archiver

## Overview

Safely archive oversized project documents and create fresh versions with only active context. Archives are **immutable** — once created, they are never modified, deleted, or overwritten. Each archive links to its predecessors, forming a complete historical chain that preserves the project's DNA.

## When to Use

- A project document (spec.md, designs.md, etc.) exceeds token threshold (~5K tokens)
- User commands: `/archive [document]` or `/archive all`
- Major milestone completion
- AI session context is degrading due to document size

**When NOT to use:** Documents are still manageable in size. Active sections that need frequent editing.

## Core Principles

### 1. Immutability (절대 금지)

```
❌ NEVER:
  - Overwrite an existing archive file
  - Modify an archive file's content
  - Delete an archive file
  - Rename an archive file

✅ ALWAYS:
  - Create a new file for each archive operation
  - Use date-based naming with collision suffixes
  - Link archives in a chain within the source document
```

### 2. Chain References (체인 참조)

Every archive file and the source document must maintain chain links so the complete history is always traceable by AI or humans.

### 3. DNA Preservation (DNA 계승)

The new version of the document must inherit core decisions, principles, and constraints — not start from blank.

## Archive Lifecycle

```
[TRIGGER] → [ANALYZE] → [CREATE ARCHIVE] → [REWRITE SOURCE] → [VERIFY]
```

## Step-by-Step Procedure

### Step 1: Trigger Analysis

When archiving is triggered, first scan the document:

```markdown
[ARCHIVE ANALYSIS]
- Document: {filename}
- Current size: {approximate tokens or lines}
- Completed sections: [list]
- Active/ongoing sections: [list]
- Historical records (old bugs, completed features): [list]
- Core DNA (principles, architecture decisions, constraints): [list]
```

### Step 2: Generate Archive Filename

```
Format: {original_name}_archive_{YYMMDD}.md
Collision: {original_name}_archive_{YYMMDD}_v2.md
           {original_name}_archive_{YYMMDD}_v3.md

Rules:
  - Check if filename already exists in .archive/
  - If exists, increment _v2, _v3, etc.
  - Never reuse a filename
```

### Step 3: Create Archive File

**Location:** `.archive/` folder (create if not exists)

**Archive file header:**
```markdown
# {Original Document Name} — Archive

> 📂 Archived: {YYYY-MM-DD}
> 🔗 Previous archive: `.archive/{previous_archive_filename}.md` (or "First archive" if none)
> 📄 Source document: `{source_filename.md}` (continues with v{N})
> 🧬 DNA inherited in source: [brief list of core principles carried forward]

---

[Full content of archived sections below — unchanged, complete]
```

**What to archive (move out of source):**
- ✅ Completed features with full implementation details
- ✅ Resolved bugs with root cause analysis
- ✅ Historical design decisions (superseded patterns)
- ✅ Past version UI layouts
- ✅ Completed milestones
- ✅ Lessons learned that are no longer actively relevant

**What to keep in source document:**
- ✅ Core principles and architecture (DNA)
- ✅ Active/ongoing work
- ✅ Current version specifications
- ✅ Open decisions
- ✅ Current troubleshooting
- ✅ Brief summary of archived content with link

### Step 4: Rewrite Source Document

**Source document header (with chain):**
```markdown
# {Document Name} (v{N})

> 📦 Archive Chain:
> - Latest → `.archive/{latest_archive_filename}.md`
> - History → `.archive/{older_archive_1}.md` → `.archive/{older_archive_2}.md` → ...

> 📝 Archived content moved to chain above. This document contains active context only.

---

[Active content continues...]
```

**For each archived section in source, replace with:**
```markdown
## {Section Title}

> 📦 Moved to archive: `.archive/{filename}.md`
>
> Brief 1-2 line summary of what was here.

```

### Step 5: Verify Chain Integrity

```markdown
[CHAIN VERIFICATION]
  - [ ] Archive file created (not modified existing)
  - [ ] Archive filename unique (no collision)
  - [ ] Archive header links to previous archive (or marks "first")
  - [ ] Source document header links to latest archive
  - [ ] Source document lists full chain history
  - [ ] Archived sections replaced with summaries + links
  - [ ] Core DNA (principles, decisions) inherited in source
  - [ ] No content lost — everything exists in either source or archive
```

## Safety Checks

### Pre-Archive Safety Gate

```
[SAFETY GATE — MUST PASS BEFORE ARCHIVING]
  1. Read source document in full
  2. Identify ALL sections to archive
  3. Check .archive/ directory for existing files
  4. Generate unique filename (verify no collision)
  5. Confirm: no existing archive will be touched
  6. Present archive plan to user (unless auto-triggered)
  7. Proceed only after confirmation
```

### Immutability Enforcement

```
[IMMUTABLE ARCHIVE PROTOCOL]
  - Archive files are READ-ONLY after creation
  - If archive content needs correction:
    → Create a NEW archive with corrected content
    → Link to the old archive as "Superseded by"
    → Never edit the old file
  - If user requests "update the archive":
    → REFUSE and explain immutability
    → Offer to create new archive instead
```

## Token Threshold Guidelines

| Document Size | Action |
|---------------|--------|
| < 3K tokens | No action needed |
| 3K–5K tokens | Monitor; warn user at session end |
| 5K–8K tokens | Recommend archive; ask user |
| > 8K tokens | Auto-trigger archive warning; archive on confirmation |
| > 12K tokens | Strongly recommend immediate archive |

## Chain Example: 3 Archives of designs.md

```
.archive/
├── designs_archive_250201.md          (1st archive)
├── designs_archive_250315.md          (2nd)
├── designs_archive_250410.md          (3rd)
└── designs_archive_250410_v2.md       (4th, same day collision)

designs.md source header:
---
# designs.md (v4)

> 📦 Archive Chain:
> - Latest → `.archive/designs_archive_250410_v2.md`
> - History → `.archive/designs_archive_250410.md` → `.archive/designs_archive_250315.md` → `.archive/designs_archive_250201.md`
---
```

## DNA Inheritance Template

When creating the new source document, carry forward:

```markdown
## 🧬 Inherited DNA (from previous versions)

> These principles and decisions are inherited from archive history and remain active:

### Core Principles
- [List key architectural principles from archives]
- [List design philosophies that still apply]

### Active Decisions
- [List decisions from archives that still govern current work]

### Known Constraints
- [List technical/business constraints inherited from history]

### Anti-Patterns (금지 패턴)
- [List patterns that were tried and rejected, from archive history]
```

## Common Scenarios

### Scenario 1: Single Archive

```
spec.md grows to 7K tokens
→ Archive completed features
→ spec.md v2 created with active features only
→ .archive/spec_archive_250410.md created
→ Chain: spec.md → spec_archive_250410.md
```

### Scenario 2: Multiple Archives (Chain)

```
spec.md v2 grows again to 6K tokens
→ Archive newly completed features
→ spec.md v3 created
→ .archive/spec_archive_250501.md created
→ Chain: spec.md → spec_archive_250501.md → spec_archive_250410.md
```

### Scenario 3: Same-Day Collision

```
Two archive operations on same day for same document
→ First: designs_archive_250410.md
→ Second: designs_archive_250410_v2.md
→ Chain updated with both entries
```

### Scenario 4: User Requests "Update Archive"

```
User: "Update the last archive with the new changes"
→ REFUSE: "Archives are immutable. I'll create a new archive instead."
→ Create new archive with current content
→ Update chain in source document
```

## Red Flags

- [ ] Archive filename collides with existing file
- [ ] Source document doesn't link to archive chain
- [ ] Archive file doesn't link to previous archive
- [ ] Content exists in neither source nor archive (data loss)
- [ ] Core DNA not inherited in source document
- [ ] Archived sections deleted without summary + link replacement

## Verification

After archiving, confirm:

- [ ] New archive file created with unique filename
- [ ] No existing archive file was modified
- [ ] Archive header contains chain link to previous archive
- [ ] Source document header contains full chain history
- [ ] Each archived section replaced with summary + archive link
- [ ] DNA (principles, decisions, constraints) inherited in source
- [ ] Zero content loss — everything traceable
- [ ] Chain is traversable from source → latest → oldest archive
