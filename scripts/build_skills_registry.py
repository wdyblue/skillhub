#!/usr/bin/env python3

from __future__ import annotations

import hashlib
import json
import os
import re
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List


DEFAULT_SKILLS_ALL_ROOT = Path.home() / ".skills-all-sync" / "data"
SKILLS_ALL_ROOT = Path(
    os.environ.get("SKILLS_ALL_ROOT", str(DEFAULT_SKILLS_ALL_ROOT))
).expanduser()

DEFAULT_SOURCE_ROOTS = [
    {
        "source_id": "skills_all_root",
        "kind": "aggregate",
        "path": str(SKILLS_ALL_ROOT),
        "agent": "none",
    },
    {
        "source_id": "codex_user",
        "kind": "agent-root",
        "path": "~/.codex/skills",
        "agent": "codex",
    },
    {
        "source_id": "workbuddy_user",
        "kind": "agent-root",
        "path": "~/.workbuddy/skills",
        "agent": "workbuddy",
    },
    {
        "source_id": "hermes_system",
        "kind": "agent-root",
        "path": "~/.hermes/profiles/system/skills",
        "agent": "hermes_system",
    },
    {
        "source_id": "hermes_blues",
        "kind": "agent-root",
        "path": "~/.hermes/profiles/blues/skills",
        "agent": "hermes_profile",
    },
    {
        "source_id": "agents_external",
        "kind": "agent-root",
        "path": "~/.agents/skills",
        "agent": "hermes_external",
    },
    {
        "source_id": "skills_manager",
        "kind": "manager-root",
        "path": "~/.skills-manager/skills",
        "agent": "skills_manager",
    },
    {
        "source_id": "toclaw_workspace",
        "kind": "agent-root",
        "path": "~/.toclaw/workspace/skills",
        "agent": "toclaw",
    },
    {
        "source_id": "workbuddy_connectors",
        "kind": "connector-root",
        "path": "~/.workbuddy/connectors/skills",
        "agent": "workbuddy_connectors",
    },
    {
        "source_id": "claude_user",
        "kind": "agent-root",
        "path": "~/.claude/skills",
        "agent": "claude",
    },
    {
        "source_id": "eagle_claude",
        "kind": "agent-root",
        "path": "~/Eagle Agent/.claude/skills",
        "agent": "claude",
    },
]

LAYOUT_DIRS = [
    "store/skills",
    "views/codex",
    "views/workbuddy",
    "views/workbuddy-connectors",
    "views/hermes-external",
    "views/toclaw",
    "views/shared-by-category/meeting",
    "views/shared-by-category/design",
    "views/shared-by-category/office",
    "views/shared-by-category/research",
    "registry",
    "quarantine",
]

CONTROL_TOP_LEVEL_DIRS = {"store", "views", "registry", "quarantine"}


@dataclass
class Exposure:
    source_id: str
    source_kind: str
    agent: str
    root_path: str
    skill_dir: str
    skill_md: str
    relative_skill_dir: str
    resolved_skill_dir: str


def load_source_roots() -> List[Dict[str, str]]:
    raw = os.environ.get("SKILLS_SOURCE_ROOTS_JSON")
    if not raw:
        return DEFAULT_SOURCE_ROOTS
    parsed = json.loads(raw)
    normalized = []
    for item in parsed:
        normalized.append(
            {
                "source_id": str(item["source_id"]),
                "kind": str(item.get("kind", "agent-root")),
                "path": str(item["path"]),
                "agent": str(item.get("agent", "unknown")),
            }
        )
    return normalized


SOURCE_ROOTS = load_source_roots()


def sha1_text(value: str) -> str:
    return hashlib.sha1(value.encode("utf-8")).hexdigest()


def slugify(value: str) -> str:
    value = value.strip()
    value = value.replace(" ", "-")
    value = re.sub(r"[^0-9A-Za-z_\-\u4e00-\u9fff]+", "-", value)
    value = re.sub(r"-{2,}", "-", value).strip("-")
    return value or "unnamed-skill"


def load_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="ignore")


def parse_skill_md(skill_md: Path) -> Dict[str, str]:
    text = load_text(skill_md)
    lines = text.splitlines()
    meta: Dict[str, str] = {}

    if lines[:1] == ["---"]:
        for line in lines[1:200]:
            if line.strip() == "---":
                break
            match = re.match(r"^([A-Za-z0-9_\-]+):\s*(.+?)\s*$", line)
            if match:
                meta[match.group(1)] = match.group(2).strip().strip("\"'")

    for line in lines[:200]:
        for key in ("skill_name", "name", "title", "description"):
            if key in meta:
                continue
            match = re.match(rf"^{key}:\s*(.+?)\s*$", line)
            if match:
                meta[key] = match.group(1).strip().strip("\"'")

    if "title" not in meta:
        for line in lines[:50]:
            if line.startswith("# "):
                meta["title"] = line[2:].strip()
                break

    return meta


def parse_skillhub_meta(skill_dir: Path) -> Dict[str, object]:
    meta_path = skill_dir / "_skillhub_meta.json"
    if not meta_path.exists():
        return {}
    try:
        return json.loads(load_text(meta_path))
    except json.JSONDecodeError:
        return {"_invalid_json": True}


def hash_file(path: Path, digest: hashlib._Hash) -> None:
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(1024 * 1024)
            if not chunk:
                break
            digest.update(chunk)


def fingerprint_dir(skill_dir: Path) -> Dict[str, object]:
    digest = hashlib.sha256()
    file_count = 0
    total_bytes = 0

    paths: List[Path] = []
    for current_root, dirnames, filenames in os.walk(skill_dir):
        dirnames.sort()
        filenames.sort()
        for filename in filenames:
            paths.append(Path(current_root) / filename)

    for path in sorted(paths):
        rel_path = path.relative_to(skill_dir).as_posix()
        digest.update(rel_path.encode("utf-8"))
        if path.is_symlink():
            target = os.readlink(path)
            digest.update(b"SYMLINK")
            digest.update(target.encode("utf-8"))
            file_count += 1
            continue

        try:
            stat_result = path.stat()
        except FileNotFoundError:
            continue

        digest.update(str(stat_result.st_size).encode("utf-8"))
        hash_file(path, digest)
        file_count += 1
        total_bytes += stat_result.st_size

    return {
        "content_digest": digest.hexdigest(),
        "file_count": file_count,
        "total_bytes": total_bytes,
    }


def ensure_layout(root: Path) -> None:
    for relative_dir in LAYOUT_DIRS:
        (root / relative_dir).mkdir(parents=True, exist_ok=True)


def iter_skill_dirs(source_root: Path, source_id: str) -> Iterable[Path]:
    for current_root, dirnames, filenames in os.walk(source_root, followlinks=True):
        current_path = Path(current_root)
        if current_path == source_root and source_id == "skills_all_root":
            dirnames[:] = [name for name in dirnames if name not in CONTROL_TOP_LEVEL_DIRS]
        dirnames.sort()
        filenames.sort()
        if "SKILL.md" in filenames:
            yield current_path


def build_registry() -> Dict[str, object]:
    entities: Dict[str, Dict[str, object]] = {}
    source_counts: Dict[str, int] = defaultdict(int)
    missing_roots: List[Dict[str, str]] = []

    for source in SOURCE_ROOTS:
        source_root = Path(source["path"]).expanduser()
        if not source_root.exists():
            missing_roots.append(
                {
                    "source_id": source["source_id"],
                    "path": str(source_root),
                }
            )
            continue

        for skill_dir in iter_skill_dirs(source_root, source["source_id"]):
            skill_md = skill_dir / "SKILL.md"
            real_dir = skill_dir.resolve()
            entity_id = sha1_text(str(real_dir))

            exposure = Exposure(
                source_id=source["source_id"],
                source_kind=source["kind"],
                agent=source["agent"],
                root_path=str(source_root),
                skill_dir=str(skill_dir),
                skill_md=str(skill_md),
                relative_skill_dir=skill_dir.relative_to(source_root).as_posix(),
                resolved_skill_dir=str(real_dir),
            )

            if entity_id not in entities:
                parsed_meta = parse_skill_md(real_dir / "SKILL.md")
                skillhub_meta = parse_skillhub_meta(real_dir)
                fingerprint = fingerprint_dir(real_dir)
                basename = real_dir.name
                suggested_slug = skillhub_meta.get("skillId") or parsed_meta.get("skill_name") or basename

                entities[entity_id] = {
                    "entity_id": entity_id,
                    "canonical_slug_suggestion": slugify(str(suggested_slug)),
                    "real_dir": str(real_dir),
                    "skill_md": str(real_dir / "SKILL.md"),
                    "dir_name": basename,
                    "display_name": parsed_meta.get("skill_name")
                    or parsed_meta.get("name")
                    or parsed_meta.get("title")
                    or basename,
                    "description": parsed_meta.get("description", ""),
                    "content_digest": fingerprint["content_digest"],
                    "file_count": fingerprint["file_count"],
                    "total_bytes": fingerprint["total_bytes"],
                    "has_skillhub_meta": bool(skillhub_meta),
                    "skillhub_meta": {
                        "skillId": skillhub_meta.get("skillId"),
                        "name": skillhub_meta.get("name"),
                        "source": skillhub_meta.get("source"),
                        "iconSource": skillhub_meta.get("iconSource"),
                    }
                    if skillhub_meta
                    else None,
                    "observed_sources": [],
                    "observed_agents": [],
                    "exposures": [],
                }

            entities[entity_id]["observed_sources"].append(source["source_id"])
            entities[entity_id]["observed_agents"].append(source["agent"])
            entities[entity_id]["exposures"].append(exposure.__dict__)
            source_counts[source["source_id"]] += 1

    for entity in entities.values():
        entity["observed_sources"] = sorted(set(entity["observed_sources"]))
        entity["observed_agents"] = sorted(set(entity["observed_agents"]))

    duplicate_groups = []
    by_digest: Dict[str, List[Dict[str, object]]] = defaultdict(list)
    for entity in entities.values():
        by_digest[entity["content_digest"]].append(entity)
    for digest, grouped_entities in sorted(by_digest.items()):
        if len(grouped_entities) < 2:
            continue
        duplicate_groups.append(
            {
                "group_type": "exact-content-duplicate",
                "content_digest": digest,
                "entity_ids": sorted(entity["entity_id"] for entity in grouped_entities),
                "real_dirs": sorted(entity["real_dir"] for entity in grouped_entities),
                "display_names": sorted({entity["display_name"] for entity in grouped_entities}),
            }
        )

    variant_groups = []
    by_name: Dict[str, List[Dict[str, object]]] = defaultdict(list)
    for entity in entities.values():
        by_name[slugify(entity["dir_name"]).lower()].append(entity)
    for normalized_name, grouped_entities in sorted(by_name.items()):
        digests = {entity["content_digest"] for entity in grouped_entities}
        if len(grouped_entities) < 2 or len(digests) < 2:
            continue
        variant_groups.append(
            {
                "group_type": "same-name-different-content",
                "normalized_name": normalized_name,
                "entity_ids": sorted(entity["entity_id"] for entity in grouped_entities),
                "real_dirs": sorted(entity["real_dir"] for entity in grouped_entities),
            }
        )

    compatibility = []
    for entity in sorted(entities.values(), key=lambda item: item["display_name"].lower()):
        visible_in = {agent: False for agent in [
            "codex",
            "workbuddy",
            "hermes_system",
            "hermes_profile",
            "hermes_external",
            "toclaw",
            "claude",
            "skills_manager",
            "workbuddy_connectors",
            "none",
        ]}
        for agent in entity["observed_agents"]:
            visible_in[agent] = True
        compatibility.append(
            {
                "entity_id": entity["entity_id"],
                "canonical_slug_suggestion": entity["canonical_slug_suggestion"],
                "display_name": entity["display_name"],
                "visible_in": visible_in,
                "has_skillhub_meta": entity["has_skillhub_meta"],
            }
        )

    summary = {
        "generated_at": __import__("datetime").datetime.now().astimezone().isoformat(),
        "skills_all_root": str(SKILLS_ALL_ROOT),
        "source_roots_scanned": len(SOURCE_ROOTS) - len(missing_roots),
        "missing_source_roots": missing_roots,
        "discovered_entities": len(entities),
        "exact_duplicate_groups": len(duplicate_groups),
        "same_name_variant_groups": len(variant_groups),
        "source_entry_counts": dict(sorted(source_counts.items())),
    }

    return {
        "summary": summary,
        "entities": sorted(entities.values(), key=lambda item: item["display_name"].lower()),
        "duplicate_groups": duplicate_groups,
        "variant_groups": variant_groups,
        "compatibility": compatibility,
        "sources": {
            "roots": [
                {
                    **source,
                    "path": str(Path(source["path"]).expanduser()),
                    "exists": Path(source["path"]).expanduser().exists(),
                }
                for source in SOURCE_ROOTS
            ],
            "missing_roots": missing_roots,
            "entry_counts": dict(sorted(source_counts.items())),
        },
    }


def write_json(path: Path, payload: object) -> None:
    path.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def write_summary_md(path: Path, registry: Dict[str, object]) -> None:
    summary = registry["summary"]
    lines = [
        "# Skills Registry Scan Summary",
        "",
        f"- Generated at: `{summary['generated_at']}`",
        f"- Skills all root: `{summary['skills_all_root']}`",
        f"- Source roots scanned: `{summary['source_roots_scanned']}`",
        f"- Discovered entities: `{summary['discovered_entities']}`",
        f"- Exact duplicate groups: `{summary['exact_duplicate_groups']}`",
        f"- Same-name variant groups: `{summary['same_name_variant_groups']}`",
        "",
        "## Source Entry Counts",
        "",
    ]
    for source_id, count in summary["source_entry_counts"].items():
        lines.append(f"- `{source_id}`: `{count}`")

    missing = summary["missing_source_roots"]
    if missing:
        lines.extend(["", "## Missing Roots", ""])
        for item in missing:
            lines.append(f"- `{item['source_id']}`: `{item['path']}`")

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    ensure_layout(SKILLS_ALL_ROOT)
    registry = build_registry()

    registry_dir = SKILLS_ALL_ROOT / "registry"
    write_json(registry_dir / "skills-index.json", registry["entities"])
    write_json(
        registry_dir / "duplicates.json",
        {
            "exact_duplicate_groups": registry["duplicate_groups"],
            "same_name_variant_groups": registry["variant_groups"],
        },
    )
    write_json(registry_dir / "compatibility.json", registry["compatibility"])
    write_json(registry_dir / "sources.json", registry["sources"])
    write_json(registry_dir / "scan-summary.json", registry["summary"])
    write_summary_md(registry_dir / "scan-summary.md", registry)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
