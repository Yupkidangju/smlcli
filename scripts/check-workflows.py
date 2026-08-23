#!/usr/bin/env python3
"""Static release-control checks for GitHub workflow drift."""

import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = [ROOT / ".github/workflows/ci.yml", ROOT / ".github/workflows/release.yml"]


def main() -> None:
    errors = []
    for workflow in WORKFLOWS:
        text = workflow.read_text(encoding="utf-8")
        for line_number, line in enumerate(text.splitlines(), start=1):
            match = re.search(r"\buses:\s*([^\s#]+)", line)
            if match and not re.fullmatch(r"[\w.-]+/[\w.-]+@[0-9a-f]{40}", match.group(1)):
                errors.append(f"{workflow.name}:{line_number}: mutable action {match.group(1)}")
            stripped = line.strip()
            if stripped.startswith("run: cargo ") and any(
                command in stripped for command in (" build ", " check ", " clippy ", " test ", " fetch ")
            ) and "--locked" not in stripped:
                errors.append(f"{workflow.name}:{line_number}: cargo command missing --locked")

    release = WORKFLOWS[1].read_text(encoding="utf-8")
    for target in ("x86_64-unknown-linux-musl", "x86_64-pc-windows-msvc"):
        if target not in release:
            errors.append(f"release.yml: missing canonical target {target}")
    for required in (
        "attestations: write",
        "id-token: write",
        "artifact-metadata: write",
        "cargo deny check",
        "cargo audit",
    ):
        if required not in release:
            errors.append(f"release.yml: missing gate {required}")
    for required in (
        "branches: [main]",
        "workflow_dispatch:",
        "Smoke test release binary",
        "verify-release-artifacts.py dist",
        "id: attest-provenance",
        "id: attest-sbom",
        "sbom-path: dist/${{ matrix.artifact }}.spdx.json",
        "steps.attest-provenance.outputs.attestation-url",
        "steps.attest-sbom.outputs.attestation-url",
        "gh attestation verify",
        "--predicate-type \"https://spdx.dev/Document/v2.3\"",
        "--source-digest \"$GITHUB_SHA\"",
        "--signer-workflow \"$GITHUB_REPOSITORY/.github/workflows/release.yml\"",
        "if: startsWith(github.ref, 'refs/tags/v')",
    ):
        if required not in release:
            errors.append(f"release.yml: missing hosted verification control {required}")
    for banned in ("ubuntu-latest", "windows-latest", "rust-toolchain@stable", "action-gh-release"):
        if banned in release:
            errors.append(f"release.yml: mutable or noncanonical control remains: {banned}")

    if errors:
        raise SystemExit("\n".join(errors))
    print("workflow controls verified")


if __name__ == "__main__":
    main()
