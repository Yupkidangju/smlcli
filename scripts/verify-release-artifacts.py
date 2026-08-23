#!/usr/bin/env python3
"""Verify every *.sha256 file next to a downloaded release artifact."""

import argparse
import hashlib
import json
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("directory")
    args = parser.parse_args()
    root = Path(args.directory)
    checksum_files = sorted(root.glob("*.sha256"))
    if not checksum_files:
        raise SystemExit("no checksum files found")
    for checksum_file in checksum_files:
        digest, flagged_name = checksum_file.read_text(encoding="utf-8").strip().split(maxsplit=1)
        artifact = root / flagged_name.lstrip(" *")
        actual = hashlib.sha256(artifact.read_bytes()).hexdigest()
        if actual != digest:
            raise SystemExit(f"checksum mismatch: {artifact.name}")
        sbom = Path(f"{artifact}.spdx.json")
        if not sbom.is_file():
            raise SystemExit(f"missing SPDX SBOM: {sbom.name}")
        try:
            document = json.loads(sbom.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            raise SystemExit(f"invalid SPDX SBOM {sbom.name}: {error}") from error
        if document.get("spdxVersion") != "SPDX-2.3":
            raise SystemExit(f"unexpected SPDX version: {sbom.name}")
        if document.get("SPDXID") != "SPDXRef-DOCUMENT":
            raise SystemExit(f"missing SPDX document identifier: {sbom.name}")
        packages = document.get("packages")
        if not isinstance(packages, list) or not any(
            package.get("name") == "smlcli" for package in packages if isinstance(package, dict)
        ):
            raise SystemExit(f"root package missing from SPDX SBOM: {sbom.name}")
        print(f"verified {artifact.name}: {actual}")
        print(f"verified {sbom.name}: SPDX-2.3")


if __name__ == "__main__":
    main()
