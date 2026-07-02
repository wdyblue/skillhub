# Skills Storage Architecture

## Goal

Build one canonical skills repository that all local agents can share without
breaking each agent's own skill discovery logic.

The design must satisfy these rules:

- Each real skill should exist in one canonical location whenever possible.
- Agents should read from agent-specific views, not directly from the canonical store.
- Agent-specific naming and directory constraints must be preserved in views.
- Deduplication, quarantine, and compatibility decisions must be tracked centrally.

## Canonical Layout

```text
/Users/yiyao1234/Documents/skills all/
  store/
    skills/
      <canonical-skill-id>/
  views/
    codex/
    workbuddy/
    workbuddy-connectors/
    hermes-external/
    toclaw/
    shared-by-category/
      meeting/
      design/
      office/
      research/
  registry/
    skills-index.json
    duplicates.json
    compatibility.json
    sources.json
  quarantine/
```

## Directory Responsibilities

### `store/skills/`

This is the canonical entity layer.

- Real files live here.
- Each skill should have one canonical directory.
- Do not organize this layer primarily by topic category.
- Do not expose this layer directly to agents.
- Keep source metadata in registry files, not by nesting the path deeply.

Recommended canonical directory name:

- Prefer a stable slug when the skill is generic and cross-agent.
- Preserve `skill_20530...` as canonical name if the marketplace identity matters.

### `views/`

This is the exposure layer.

- Agents read from here.
- Each view can use symlinks to one or more canonical skills in `store/skills/`.
- The same canonical skill can appear in multiple views.
- Category folders belong here if needed.

### `registry/`

This is the control layer.

- `skills-index.json`: one record per canonical skill.
- `duplicates.json`: duplicate groups, preferred survivor, quarantined copies.
- `compatibility.json`: which agents can read which skills.
- `sources.json`: original install source and original path history.

### `quarantine/`

This is the isolation layer.

- Old copies.
- Conflicting variants.
- Incomplete skills.
- Skills with broken metadata.
- Skills pending manual review.

## Agent Read Model

Agents should not scan the canonical store directly.

They should scan only their own view:

- Codex reads `views/codex/`
- WorkBuddy reads `views/workbuddy/`
- WorkBuddy connectors read `views/workbuddy-connectors/`
- Hermes reads `views/hermes-external/`
- ToClaw reads `views/toclaw/`

Optional:

- `views/shared-by-category/` is for human browsing and manual operations, not as a primary agent input path.

## Compatibility Rules

### Codex

- Can read a flat or categorized skill directory as long as the structure is valid.
- Use `views/codex/` as the Codex-facing root.
- Do not rely on Codex reading the canonical store directly.

### WorkBuddy / CodeBuddy / SkillHub

- Preserve marketplace folder names when required.
- Preserve `_skillhub_meta.json` when present.
- Do not rename marketplace skills just for aesthetics in the WorkBuddy view.
- Use `views/workbuddy/` as the WorkBuddy-facing root.

### Hermes

- Hermes should continue reading an external directory rather than mixing all skills into its internal system paths.
- Point `~/.agents/skills` at `views/hermes-external/` or populate it from that view.
- Avoid duplicate names in Hermes external view when Hermes already has an internal skill of the same name.

### ToClaw

- Keep a dedicated flat view if ToClaw expects direct skill folders.
- Use `views/toclaw/` as the ToClaw-facing root.

## Naming Rules

### Canonical naming

- Use a stable canonical directory name in `store/skills/`.
- If the skill has a strong external identity, preserve it.
- If a marketplace skill is known primarily by `skill_20530...`, keep that as canonical unless a stronger invariant is proven.

### View naming

- View names may differ from canonical names if the agent needs a specific name.
- A single canonical skill may have multiple exposed names across views.

Examples:

- Canonical: `store/skills/skill_2053084099650080768/`
- WorkBuddy view: `views/workbuddy/skill_2053084099650080768 -> ../../store/skills/skill_2053084099650080768`
- Hermes view: `views/hermes-external/tencent-meeting -> ../../store/skills/skill_2053084099650080768`

Only do this rename in a view when compatibility is confirmed.

## Classification Rules

Topic classification should not control the canonical store.

Use category folders only in human-facing or agent-specific views:

- `views/shared-by-category/design/...`
- `views/shared-by-category/meeting/...`

This avoids duplicating entities and avoids reclassifying the same skill into multiple canonical locations.

## Install and Sync Rules

### New skill from any agent

1. Detect the newly installed skill.
2. Decide whether it is a new canonical skill or a duplicate of an existing one.
3. Move or sync the real files into `store/skills/<canonical-skill-id>/`.
4. Update registry records.
5. Regenerate relevant views.

### New skill created manually in Codex

1. Create the skill in canonical store first if possible.
2. Register compatibility for Codex, Hermes, WorkBuddy, or ToClaw.
3. Generate links into every compatible view.

### Marketplace skill

1. Preserve original metadata and identity.
2. Store one canonical copy.
3. Expose it to WorkBuddy with the original marketplace-facing name.
4. Expose it to other agents only after compatibility review.

## Deduplication Policy

- Exact duplicates: keep one canonical copy, remove or quarantine the others.
- Same name but different content: keep separate canonical entries until reviewed.
- Same capability but different ecosystems: do not merge automatically.
- Broken or partial copies: quarantine, do not expose in views.

## Current Local Strategy

The safest rollout path on this Mac is:

1. Build registry from current skill roots.
2. Rebuild `~/.agents/skills` as a clean Hermes external view.
3. Keep existing Codex and WorkBuddy paths untouched until the new views are verified.
4. Then migrate Codex, WorkBuddy, and ToClaw one by one to view-based reading.

## Sync Automation

Automation should run a two-step rebuild:

1. Rebuild the registry from all configured source roots.
2. Rebuild canonical store and agent views from that registry.

Local implementation:

- Sync entrypoint: `/Users/yiyao1234/Documents/skillhub/scripts/sync_skills_all.py`
- Registry rebuild: `/Users/yiyao1234/Documents/skillhub/scripts/build_skills_registry.py`
- View and root rebuild: `/Users/yiyao1234/Documents/skillhub/scripts/migrate_skills_architecture.py`

Recommended runtime model:

- Use `launchd` with `RunAtLoad`.
- Also run on a short interval so newly installed skills in any agent root are re-ingested into canonical store.
- Keep a lock file so overlapping sync jobs do not collide.

## Non-Goals

- Do not force every agent to read one identical top-level directory.
- Do not flatten all skills into one visible root for all agents.
- Do not rename marketplace skills globally.
- Do not use iCloud category layout as the canonical storage model.

## Practical Principle

The stable model is:

- One canonical entity layer.
- Multiple agent-facing view layers.
- One central registry that decides what is shared, what is quarantined, and what each agent is allowed to read.
