#!/usr/bin/env python3
"""Copy one release binary and emit a portable SHA-256 checksum file."""

import argparse
import hashlib
import shutil
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    parser.add_argument("--artifact", required=True)
    args = parser.parse_args()

    source = Path(args.binary)
    artifact = Path(args.artifact)
    artifact.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, artifact)
    digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
    artifact.with_suffix(artifact.suffix + ".sha256").write_text(
        f"{digest} *{artifact.name}\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
