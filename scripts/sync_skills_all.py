#!/usr/bin/env python3

from __future__ import annotations

import fcntl
import os
import subprocess
import sys
import time
from pathlib import Path


SCRIPT_ROOT = Path(__file__).resolve().parent
DEFAULT_SKILLS_ALL_ROOT = Path.home() / ".skills-all-sync" / "data"
SKILLS_ALL_ROOT = Path(
    os.environ.get("SKILLS_ALL_ROOT", str(DEFAULT_SKILLS_ALL_ROOT))
).expanduser()
REGISTRY_ROOT = SKILLS_ALL_ROOT / "registry"
LOCK_PATH = REGISTRY_ROOT / ".sync.lock"
LOG_PATH = REGISTRY_ROOT / "sync.log"

BUILD_SCRIPT = SCRIPT_ROOT / "build_skills_registry.py"
MIGRATE_SCRIPT = SCRIPT_ROOT / "migrate_skills_architecture.py"


def log(message: str) -> None:
    timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
    line = f"[{timestamp}] {message}\n"
    LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    with LOG_PATH.open("a", encoding="utf-8") as handle:
        handle.write(line)


def run_script(script: Path) -> None:
    subprocess.run([sys.executable, str(script)], check=True, env=os.environ.copy())


def main() -> int:
    LOCK_PATH.parent.mkdir(parents=True, exist_ok=True)
    with LOCK_PATH.open("w", encoding="utf-8") as handle:
        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            log("sync skipped because another sync is already running")
            return 0

        try:
            log("sync started")
            run_script(BUILD_SCRIPT)
            log("registry rebuilt")
            run_script(MIGRATE_SCRIPT)
            log("migration rebuilt successfully")
            return 0
        except subprocess.CalledProcessError as exc:
            log(f"sync failed: {exc}")
            return exc.returncode or 1
        except Exception as exc:  # pragma: no cover
            log(f"sync failed with unexpected error: {exc}")
            return 1
        finally:
            fcntl.flock(handle.fileno(), fcntl.LOCK_UN)


if __name__ == "__main__":
    raise SystemExit(main())
