#!/usr/bin/env python3
"""Generate a deterministic SPDX 2.3 JSON SBOM from locked Cargo metadata."""

import argparse
import datetime
import hashlib
import json
import os
import subprocess
from pathlib import Path


def spdx_id(package_id: str) -> str:
    digest = hashlib.sha256(package_id.encode("utf-8")).hexdigest()[:16]
    return f"SPDXRef-Package-{digest}"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    metadata = json.loads(
        subprocess.check_output(
            [
                "cargo",
                "metadata",
                "--locked",
                "--format-version",
                "1",
                "--filter-platform",
                args.target,
            ],
            text=True,
        )
    )
    packages_by_id = {package["id"]: package for package in metadata["packages"]}
    root = next(package for package in metadata["packages"] if package["name"] == "smlcli")
    epoch = int(os.environ.get("SOURCE_DATE_EPOCH", "0"))
    created = datetime.datetime.fromtimestamp(epoch, datetime.timezone.utc).strftime(
        "%Y-%m-%dT%H:%M:%SZ"
    )

    packages = []
    for package_id in sorted(packages_by_id):
        package = packages_by_id[package_id]
        source = package.get("source") or "NOASSERTION"
        packages.append(
            {
                "SPDXID": spdx_id(package_id),
                "name": package["name"],
                "versionInfo": package["version"],
                "downloadLocation": source,
                "filesAnalyzed": False,
                "licenseConcluded": package.get("license") or "NOASSERTION",
                "licenseDeclared": package.get("license") or "NOASSERTION",
                "copyrightText": "NOASSERTION",
                "externalRefs": [
                    {
                        "referenceCategory": "PACKAGE-MANAGER",
                        "referenceType": "purl",
                        "referenceLocator": (
                            f"pkg:cargo/{package['name']}@{package['version']}"
                        ),
                    }
                ],
            }
        )

    relationships = [
        {
            "spdxElementId": "SPDXRef-DOCUMENT",
            "relationshipType": "DESCRIBES",
            "relatedSpdxElement": spdx_id(root["id"]),
        }
    ]
    resolve = metadata.get("resolve") or {"nodes": []}
    for node in sorted(resolve["nodes"], key=lambda item: item["id"]):
        for dependency in sorted(node.get("dependencies", [])):
            if node["id"] in packages_by_id and dependency in packages_by_id:
                relationships.append(
                    {
                        "spdxElementId": spdx_id(node["id"]),
                        "relationshipType": "DEPENDS_ON",
                        "relatedSpdxElement": spdx_id(dependency),
                    }
                )

    document = {
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": f"smlcli-{root['version']}-{args.target}",
        "documentNamespace": (
            "https://spdx.org/spdxdocs/"
            f"smlcli-{root['version']}-{args.target}-{spdx_id(root['id']).lower()}"
        ),
        "creationInfo": {"created": created, "creators": ["Tool: smlcli-generate-sbom"]},
        "packages": packages,
        "relationships": relationships,
    }
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(document, ensure_ascii=False, sort_keys=True, indent=2) + "\n")


if __name__ == "__main__":
    main()
