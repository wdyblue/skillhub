#!/usr/bin/env python3

from __future__ import annotations

import json
import os
import shutil
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple


DEFAULT_SKILLS_ALL_ROOT = Path.home() / ".skills-all-sync" / "data"
SKILLS_ALL_ROOT = Path(
    os.environ.get("SKILLS_ALL_ROOT", str(DEFAULT_SKILLS_ALL_ROOT))
).expanduser()
REGISTRY_ROOT = SKILLS_ALL_ROOT / "registry"
STORE_ROOT = SKILLS_ALL_ROOT / "store" / "skills"
VIEWS_ROOT = SKILLS_ALL_ROOT / "views"
QUARANTINE_ROOT = SKILLS_ALL_ROOT / "quarantine"
BACKUPS_ROOT = SKILLS_ALL_ROOT / "backups"
STORE_STAGING_ROOT = SKILLS_ALL_ROOT / ".store-staging"
QUARANTINE_STAGING_ROOT = SKILLS_ALL_ROOT / ".quarantine-staging"

VIEW_NAMES = [
    "codex",
    "workbuddy",
    "workbuddy-connectors",
    "hermes-external",
    "toclaw",
]

DEFAULT_ROOT_MIGRATIONS = [
    {"name": "codex", "path": "~/.codex/skills", "view": "codex"},
    {"name": "workbuddy", "path": "~/.workbuddy/skills", "view": "workbuddy"},
    {
        "name": "workbuddy-connectors",
        "path": "~/.workbuddy/connectors/skills",
        "view": "workbuddy-connectors",
    },
    {"name": "hermes-external", "path": "~/.agents/skills", "view": "hermes-external"},
    {"name": "toclaw", "path": "~/.toclaw/workspace/skills", "view": "toclaw"},
]

SOURCE_PRIORITY = {
    "workbuddy_connectors": 0,
    "codex_user": 1,
    "workbuddy_user": 2,
    "hermes_system": 3,
    "toclaw_workspace": 4,
    "skills_all_root": 5,
    "skills_manager": 6,
    "eagle_claude": 7,
    "claude_user": 8,
    "agents_external": 9,
    "hermes_blues": 10,
}

VIEW_SOURCE_PRIORITY = {
    "codex": [
        "codex_user",
        "workbuddy_user",
        "skills_all_root",
        "hermes_system",
        "toclaw_workspace",
        "skills_manager",
        "claude_user",
        "eagle_claude",
    ],
    "workbuddy": [
        "workbuddy_user",
        "codex_user",
        "skills_all_root",
        "hermes_system",
        "toclaw_workspace",
        "skills_manager",
        "claude_user",
        "eagle_claude",
    ],
    "workbuddy-connectors": [
        "workbuddy_connectors",
    ],
    "hermes-external": [
        "skills_all_root",
        "toclaw_workspace",
        "skills_manager",
        "codex_user",
        "workbuddy_user",
        "claude_user",
        "eagle_claude",
    ],
    "toclaw": [
        "toclaw_workspace",
        "skills_all_root",
        "codex_user",
        "workbuddy_user",
        "skills_manager",
        "claude_user",
        "eagle_claude",
    ],
}


def load_root_migrations() -> List[Dict[str, str]]:
    raw = os.environ.get("SKILLS_ROOT_MIGRATIONS_JSON")
    if not raw:
        return DEFAULT_ROOT_MIGRATIONS
    parsed = json.loads(raw)
    normalized = []
    for item in parsed:
        normalized.append(
            {
                "name": str(item["name"]),
                "path": str(item["path"]),
                "view": str(item["view"]),
            }
        )
    return normalized


ROOT_MIGRATIONS = load_root_migrations()


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: object) -> None:
    path.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def slugify(value: str) -> str:
    value = value.strip().replace(" ", "-")
    cleaned = []
    for char in value:
        if char.isalnum() or char in {"-", "_"} or "\u4e00" <= char <= "\u9fff":
            cleaned.append(char)
        else:
            cleaned.append("-")
    slug = "".join(cleaned)
    while "--" in slug:
        slug = slug.replace("--", "-")
    return (slug.strip("-") or "unnamed-skill").lower()


def short_id(value: str) -> str:
    return value[:8]


def clear_directory_contents(path: Path) -> None:
    if not path.exists():
        path.mkdir(parents=True, exist_ok=True)
        return
    for child in path.iterdir():
        if child.is_symlink() or child.is_file():
            child.unlink()
        else:
            shutil.rmtree(child)


def remove_path(path: Path) -> None:
    if not path.exists() and not path.is_symlink():
        return
    if path.is_symlink() or path.is_file():
        path.unlink()
    else:
        shutil.rmtree(path)


def unique_child_dir(parent: Path, base_name: str) -> Path:
    candidate = parent / base_name
    index = 2
    while candidate.exists() or candidate.is_symlink():
        candidate = parent / f"{base_name}-{index}"
        index += 1
    return candidate


def choose_representative(entities: List[Dict[str, object]]) -> Dict[str, object]:
    def rank(entity: Dict[str, object]) -> Tuple[int, int, int, str]:
        exposure_priorities = [
            SOURCE_PRIORITY.get(exposure["source_id"], 999)
            for exposure in entity["exposures"]
        ]
        best_source = min(exposure_priorities) if exposure_priorities else 999
        path_depth = min(
            exposure["relative_skill_dir"].count("/") for exposure in entity["exposures"]
        )
        return (
            best_source,
            -int(entity.get("file_count", 0)),
            -int(entity.get("total_bytes", 0)),
            entity["real_dir"],
        )

    return sorted(entities, key=rank)[0]


def choose_canonical_slug(
    representative: Dict[str, object],
    entities: List[Dict[str, object]],
    used_slugs: Set[str],
) -> str:
    candidates = [
        representative.get("skillhub_meta", {}).get("skillId")
        if representative.get("skillhub_meta")
        else None,
        representative.get("dir_name"),
        representative.get("display_name"),
        representative.get("canonical_slug_suggestion"),
    ]

    for candidate in candidates:
        if not candidate:
            continue
        slug = slugify(str(candidate))
        if slug not in used_slugs:
            used_slugs.add(slug)
            return slug

    base = slugify(str(representative.get("dir_name") or representative["entity_id"]))
    slug = f"{base}-{short_id(representative['entity_id'])}"
    while slug in used_slugs:
        slug = f"{base}-{short_id(representative['entity_id'])}-{len(used_slugs)}"
    used_slugs.add(slug)
    return slug


def load_canonical_groups() -> List[Dict[str, object]]:
    entities = load_json(REGISTRY_ROOT / "skills-index.json")
    by_digest: Dict[str, List[Dict[str, object]]] = defaultdict(list)
    for entity in entities:
        by_digest[entity["content_digest"]].append(entity)

    groups = []
    used_slugs: Set[str] = set()
    for digest, grouped_entities in sorted(by_digest.items()):
        representative = choose_representative(grouped_entities)
        canonical_slug = choose_canonical_slug(representative, grouped_entities, used_slugs)
        groups.append(
            {
                "canonical_slug": canonical_slug,
                "content_digest": digest,
                "representative_entity_id": representative["entity_id"],
                "representative_real_dir": representative["real_dir"],
                "display_name": representative["display_name"],
                "dir_name": representative["dir_name"],
                "has_skillhub_meta": representative["has_skillhub_meta"],
                "skillhub_meta": representative["skillhub_meta"],
                "members": grouped_entities,
            }
        )
    return groups


def is_under(path: Path, parent: Path) -> bool:
    try:
        path.resolve().relative_to(parent.resolve())
        return True
    except ValueError:
        return False
    except FileNotFoundError:
        return False


def choose_copy_source(group: Dict[str, object]) -> Path:
    non_store_paths = []
    store_paths = []

    for member in group["members"]:
        real_dir = Path(member["real_dir"])
        if real_dir.exists():
            if is_under(real_dir, STORE_ROOT):
                store_paths.append(real_dir)
            else:
                non_store_paths.append(real_dir)

        for exposure in member["exposures"]:
            skill_dir = Path(exposure["skill_dir"])
            if not skill_dir.exists():
                continue
            if is_under(skill_dir, STORE_ROOT):
                store_paths.append(skill_dir)
            else:
                non_store_paths.append(skill_dir)

    if non_store_paths:
        return sorted(non_store_paths)[0]
    if store_paths:
        return sorted(store_paths)[0]
    raise FileNotFoundError(f"No copy source found for canonical skill {group['canonical_slug']}")


def copy_canonical_store(groups: List[Dict[str, object]]) -> List[Dict[str, object]]:
    clear_directory_contents(STORE_STAGING_ROOT)
    staging_skills_root = STORE_STAGING_ROOT / "skills"
    staging_skills_root.mkdir(parents=True, exist_ok=True)
    created = []
    for group in groups:
        source_dir = choose_copy_source(group)
        target_dir = staging_skills_root / group["canonical_slug"]
        shutil.copytree(source_dir, target_dir, symlinks=True)
        created.append(
            {
                "canonical_slug": group["canonical_slug"],
                "source_dir": str(source_dir),
                "target_dir": str(target_dir),
            }
        )
    remove_path(STORE_ROOT)
    staging_skills_root.rename(STORE_ROOT)
    remove_path(STORE_STAGING_ROOT)
    return created


def quarantine_sources(group: Dict[str, object]) -> List[Path]:
    representative_real_dir = Path(group["representative_real_dir"]).resolve()
    candidates: Dict[str, Path] = {}

    for member in group["members"]:
        real_dir = Path(member["real_dir"])
        if real_dir.exists():
            resolved = real_dir.resolve()
            if resolved != representative_real_dir and not is_under(resolved, STORE_ROOT) and not is_under(resolved, VIEWS_ROOT) and not is_under(resolved, QUARANTINE_ROOT):
                candidates[str(resolved)] = resolved

        for exposure in member["exposures"]:
            skill_dir = Path(exposure["skill_dir"]).expanduser()
            if not skill_dir.exists():
                continue
            resolved = skill_dir.resolve()
            if resolved != representative_real_dir and not is_under(resolved, STORE_ROOT) and not is_under(resolved, VIEWS_ROOT) and not is_under(resolved, QUARANTINE_ROOT):
                candidates[str(resolved)] = resolved

    return sorted(candidates.values())


def rebuild_quarantine(groups: List[Dict[str, object]]) -> List[Dict[str, str]]:
    clear_directory_contents(QUARANTINE_STAGING_ROOT)
    exact_root = QUARANTINE_STAGING_ROOT / "exact-duplicates"
    exact_root.mkdir(parents=True, exist_ok=True)

    entries = []
    for group in groups:
        sources = quarantine_sources(group)
        if not sources:
            continue
        group_root = exact_root / group["canonical_slug"]
        group_root.mkdir(parents=True, exist_ok=True)
        for source_dir in sources:
            base_name = slugify(source_dir.name or group["canonical_slug"])
            target_dir = unique_child_dir(
                group_root,
                f"{base_name}-{short_id(group['content_digest'])}",
            )
            shutil.copytree(source_dir, target_dir, symlinks=True)
            entries.append(
                {
                    "canonical_slug": group["canonical_slug"],
                    "source_dir": str(source_dir),
                    "target_dir": str(target_dir),
                    "reason": "exact-duplicate-snapshot",
                }
            )

    remove_path(QUARANTINE_ROOT)
    QUARANTINE_STAGING_ROOT.rename(QUARANTINE_ROOT)
    remove_path(QUARANTINE_STAGING_ROOT)
    return entries


def all_exposures(group: Dict[str, object]) -> List[Dict[str, object]]:
    exposures: List[Dict[str, object]] = []
    for member in group["members"]:
        exposures.extend(member["exposures"])
    return exposures


def exposure_candidates(group: Dict[str, object], view_name: str) -> List[Dict[str, object]]:
    exposures = all_exposures(group)
    allowed_sources = VIEW_SOURCE_PRIORITY[view_name]
    ranked = []
    for exposure in exposures:
        if exposure["source_id"] not in allowed_sources:
            continue
        ranked.append(
            (
                allowed_sources.index(exposure["source_id"]),
                exposure["relative_skill_dir"].count("/"),
                exposure["relative_skill_dir"],
                exposure,
            )
        )
    return [item[3] for item in sorted(ranked)]


def hermes_internal_names(groups: List[Dict[str, object]]) -> Set[str]:
    names: Set[str] = set()
    for group in groups:
        for exposure in all_exposures(group):
            if exposure["source_id"] != "hermes_system":
                continue
            for member in group["members"]:
                names.add(slugify(str(member["dir_name"])).lower())
                names.add(slugify(str(member["display_name"])).lower())
    return names


def relative_target(link_path: Path, target_dir: Path) -> str:
    return os.path.relpath(target_dir, start=link_path.parent)


def create_view_link(link_path: Path, target_dir: Path) -> None:
    link_path.parent.mkdir(parents=True, exist_ok=True)
    if link_path.exists() or link_path.is_symlink():
        if link_path.is_symlink() or link_path.is_file():
            link_path.unlink()
        else:
            shutil.rmtree(link_path)
    os.symlink(relative_target(link_path, target_dir), link_path)


def choose_view_path(
    group: Dict[str, object],
    view_name: str,
    occupied: Set[str],
    hermes_blocked_names: Set[str],
    occupied_leaf_names: Set[str],
) -> Optional[str]:
    candidates = exposure_candidates(group, view_name)
    canonical_slug = group["canonical_slug"]

    if view_name == "workbuddy-connectors":
        if not candidates:
            return None
        rel = candidates[0]["relative_skill_dir"]
        if rel in occupied:
            rel = f"{rel}-{short_id(group['content_digest'])}"
        occupied.add(rel)
        return rel

    if view_name == "hermes-external":
        dir_name_norm = slugify(str(group["dir_name"])).lower()
        display_name_norm = slugify(str(group["display_name"])).lower()
        if dir_name_norm in hermes_blocked_names or display_name_norm in hermes_blocked_names:
            return None

        ranked_rel_paths = []
        for exposure in candidates:
            rel = exposure["relative_skill_dir"]
            if rel.startswith(".system/"):
                rel = canonical_slug
            ranked_rel_paths.append(rel)
        if not ranked_rel_paths:
            ranked_rel_paths.append(canonical_slug)

        for rel in ranked_rel_paths:
            leaf_name = Path(rel).name.lower()
            if leaf_name in occupied_leaf_names:
                continue
            if rel in occupied:
                rel = f"{rel}-{short_id(group['content_digest'])}"
            occupied.add(rel)
            occupied_leaf_names.add(Path(rel).name.lower())
            return rel
        return None

    if candidates:
        rel = candidates[0]["relative_skill_dir"]
        if view_name == "hermes-external" and rel.startswith(".system/"):
            rel = canonical_slug
    else:
        rel = canonical_slug

    if rel.startswith(".system/") and view_name in {"codex", "workbuddy"}:
        pass
    elif rel.startswith(".system/"):
        rel = canonical_slug

    if rel in occupied:
        rel = f"{rel}-{short_id(group['content_digest'])}"
    occupied.add(rel)
    return rel


def build_views(groups: List[Dict[str, object]]) -> Dict[str, List[Dict[str, str]]]:
    for view_name in VIEW_NAMES:
        clear_directory_contents(VIEWS_ROOT / view_name)

    hermes_blocked = hermes_internal_names(groups)
    view_links: Dict[str, List[Dict[str, str]]] = {view: [] for view in VIEW_NAMES}
    occupied_paths: Dict[str, Set[str]] = {view: set() for view in VIEW_NAMES}
    occupied_leaf_names: Dict[str, Set[str]] = {view: set() for view in VIEW_NAMES}

    for group in groups:
        target_dir = STORE_ROOT / group["canonical_slug"]
        for view_name in VIEW_NAMES:
            rel_path = choose_view_path(
                group,
                view_name,
                occupied_paths[view_name],
                hermes_blocked,
                occupied_leaf_names[view_name],
            )
            if not rel_path:
                continue
            link_path = VIEWS_ROOT / view_name / rel_path
            create_view_link(link_path, target_dir)
            view_links[view_name].append(
                {
                    "canonical_slug": group["canonical_slug"],
                    "link_path": str(link_path),
                    "target_dir": str(target_dir),
                    "relative_path": rel_path,
                }
            )

    return view_links


def backup_existing_root(path: Path, backup_root: Path) -> Optional[str]:
    if not path.exists() and not path.is_symlink():
        return None
    backup_path = backup_root / path.name
    counter = 1
    while backup_path.exists() or backup_path.is_symlink():
        backup_path = backup_root / f"{path.name}-{counter}"
        counter += 1
    backup_path.parent.mkdir(parents=True, exist_ok=True)
    shutil.move(str(path), str(backup_path))
    return str(backup_path)


def repoint_roots(timestamp: str) -> List[Dict[str, Optional[str]]]:
    results = []
    backup_session_root = BACKUPS_ROOT / f"roots-{timestamp}"
    backup_session_root.mkdir(parents=True, exist_ok=True)

    for item in ROOT_MIGRATIONS:
        target_root = Path(item["path"]).expanduser()
        target_root.parent.mkdir(parents=True, exist_ok=True)
        view_path = VIEWS_ROOT / item["view"]
        status = "repointed"

        if target_root.is_symlink():
            try:
                if target_root.resolve() == view_path.resolve():
                    results.append(
                        {
                            "name": item["name"],
                            "root_path": str(target_root),
                            "backup_path": None,
                            "view_path": str(view_path),
                            "status": "already-pointed",
                        }
                    )
                    continue
            except FileNotFoundError:
                pass

        backup_path = backup_existing_root(target_root, backup_session_root / item["name"])
        os.symlink(str(view_path), str(target_root))
        results.append(
            {
                "name": item["name"],
                "root_path": str(target_root),
                "backup_path": backup_path,
                "view_path": str(view_path),
                "status": status,
            }
        )
    return results


def write_reports(
    groups: List[Dict[str, object]],
    copied: List[Dict[str, object]],
    quarantined: List[Dict[str, str]],
    view_links: Dict[str, List[Dict[str, str]]],
    repointed_roots: List[Dict[str, Optional[str]]],
    timestamp: str,
) -> None:
    canonical_map = []
    for group in groups:
        canonical_map.append(
            {
                "canonical_slug": group["canonical_slug"],
                "content_digest": group["content_digest"],
                "display_name": group["display_name"],
                "dir_name": group["dir_name"],
                "representative_real_dir": group["representative_real_dir"],
                "member_entity_ids": [member["entity_id"] for member in group["members"]],
                "member_real_dirs": [member["real_dir"] for member in group["members"]],
            }
        )

    report = {
        "generated_at": datetime.now().astimezone().isoformat(),
        "canonical_skills": len(groups),
        "copied_entities": len(copied),
        "quarantine_entries": len(quarantined),
        "view_entry_counts": {view: len(entries) for view, entries in view_links.items()},
        "repointed_roots": repointed_roots,
    }

    write_json(REGISTRY_ROOT / "canonical-map.json", canonical_map)
    write_json(REGISTRY_ROOT / "migration-report.json", report)
    write_json(
        REGISTRY_ROOT / "view-links.json",
        {view: entries for view, entries in view_links.items()},
    )
    write_json(
        REGISTRY_ROOT / "quarantine-report.json",
        {
            "generated_at": report["generated_at"],
            "entries": quarantined,
            "quarantine_entries": len(quarantined),
            "quarantine_root": str(QUARANTINE_ROOT),
        },
    )
    (REGISTRY_ROOT / "migration-report.md").write_text(
        "\n".join(
            [
                "# Skills Migration Report",
                "",
                f"- Generated at: `{report['generated_at']}`",
                f"- Canonical skills: `{report['canonical_skills']}`",
                f"- Copied canonical entities: `{report['copied_entities']}`",
                f"- Quarantine entries: `{report['quarantine_entries']}`",
                "",
                "## View Entry Counts",
                "",
                *[
                    f"- `{view}`: `{count}`"
                    for view, count in report["view_entry_counts"].items()
                ],
                "",
                "## Repointed Roots",
                "",
                *[
                    f"- `{item['name']}`: `{item['root_path']}` -> `{item['view_path']}`"
                    for item in repointed_roots
                ],
                "",
                f"- Backup session: `{(BACKUPS_ROOT / f'roots-{timestamp}').as_posix()}`",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    (REGISTRY_ROOT / "quarantine-report.md").write_text(
        "\n".join(
            [
                "# Skills Quarantine Report",
                "",
                f"- Generated at: `{report['generated_at']}`",
                f"- Quarantine root: `{QUARANTINE_ROOT.as_posix()}`",
                f"- Entries: `{len(quarantined)}`",
                "",
                "## Entries",
                "",
                *[
                    f"- `{item['canonical_slug']}`: `{item['source_dir']}` -> `{item['target_dir']}`"
                    for item in quarantined
                ],
            ]
        )
        + "\n",
        encoding="utf-8",
    )


def ensure_layout() -> None:
    STORE_ROOT.mkdir(parents=True, exist_ok=True)
    QUARANTINE_ROOT.mkdir(parents=True, exist_ok=True)
    BACKUPS_ROOT.mkdir(parents=True, exist_ok=True)
    for view_name in VIEW_NAMES:
        (VIEWS_ROOT / view_name).mkdir(parents=True, exist_ok=True)


def main() -> int:
    ensure_layout()
    groups = load_canonical_groups()
    copied = copy_canonical_store(groups)
    quarantined = rebuild_quarantine(groups)
    view_links = build_views(groups)
    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    repointed = repoint_roots(timestamp)
    write_reports(groups, copied, quarantined, view_links, repointed, timestamp)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
