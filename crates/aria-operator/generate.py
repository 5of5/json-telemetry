#!/usr/bin/env python3
"""Generate the frozen operator catalog and 535 distinct crates.

Reads the Binary Repository v1 sheet dumps (or the already-frozen catalog)
and writes:
  catalog/operators.json
  crates/operators/<package>/{Cargo.toml,spec.json,src/lib.rs,src/main.rs}

Each crate is unique (package name, BINARY_ID, spec.json). Calculation is
shared: every binary links aria-operator, which runs telemetry::transform
(the Aria transformer) and projects the closed operator JSON (sheet 09).
"""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SHEET = Path("/tmp/binrepo")
OUT_CATALOG = Path(__file__).resolve().parent / "catalog" / "operators.json"
OUT_OPS = ROOT / "crates" / "operators"


def load_sheet(name: str) -> list:
    raw = json.loads((SHEET / f"{name}.json").read_text())
    return raw["values"]


def rows(name: str) -> list[dict]:
    vals = load_sheet(name)
    hdr = start = None
    for i, r in enumerate(vals):
        if r and r[0] == "binary_id":
            hdr, start = r, i + 1
            break
    if hdr is None:
        raise SystemExit(f"no header in {name}")
    out = []
    for r in vals[start:]:
        if not r or not str(r[0]).startswith("BIN."):
            continue
        d = dict(zip(hdr, r + [""] * (len(hdr) - len(r))))
        out.append(d)
    return out


def split_list(raw: str) -> list[str]:
    if not raw or raw.strip() in {"—", "-", "n/a", "internal"}:
        return []
    parts = re.split(r"[|,]", raw)
    out = []
    for p in parts:
        p = re.sub(r"\s*\(.*?\)\s*", " ", p).strip()
        if p and p not in {"—", "-"}:
            out.append(p)
    return out


def parse_limit(raw: str) -> int | None:
    if not raw:
        return None
    s = raw.strip()
    if s.isdigit():
        return int(s)
    return None


def package_name(crate: str) -> str:
    return crate.replace("_", "-")


def from_family(row: dict) -> dict:
    layer = row["layer"].strip()
    return {
        "binary_id": row["binary_id"],
        "operator": row["operator"],
        "layer": layer,
        "class": layer,
        "parent": row.get("parent") or "",
        "crate": row["crate"],
        "package": package_name(row["crate"]),
        "telemetry_fork": row.get("json_telemetry_fork") or "",
        "verify": row.get("VERIFY") == "T",
        "neo4j_pass": row.get("neo4j_pass") == "YES",
        "default_limit": parse_limit(row.get("default_limit") or ""),
        "node_types": split_list(row.get("node_types_emitted") or ""),
        "relationship_types": split_list(row.get("relationship_types_emitted") or ""),
        "anchor_tags": split_list(row.get("required_anchor_tags") or ""),
        "result_definition_ref": row.get("resultDefinitionRef") or "",
        "retrieval_step": row.get("retrieval_step") or "",
        "pass_through": layer in {"TRANSFORM", "HOST"},
        "property_key": None,
        "taxonomy": None,
        "wave": None,
    }


def slug_from_neo4j(obj: str, prefix: str) -> str:
    s = obj.strip()
    if s.lower().startswith(prefix):
        return s.split(":", 1)[-1]
    return s


def from_residual(row: dict) -> dict:
    cls = row["class"].strip()
    neo = row.get("neo4j_object") or ""
    node_types, rels, tags, prop = [], [], [], None
    if cls == "NODE":
        node_types = [slug_from_neo4j(neo, "label:")]
    elif cls == "REL":
        rels = [slug_from_neo4j(neo, "rel:")]
    elif cls == "TAG":
        tags = [slug_from_neo4j(neo, "tag:")]
        rels = ["TAGGED_AS"]
    elif cls == "PROP":
        prop = slug_from_neo4j(neo, "prop:")
    return {
        "binary_id": row["binary_id"],
        "operator": row["operator"],
        "layer": "RESIDUAL",
        "class": cls,
        "parent": row.get("parent_family") or "",
        "crate": row["crate"],
        "package": package_name(row["crate"]),
        "telemetry_fork": row.get("json_telemetry_fork") or "",
        "verify": row.get("VERIFY") == "T",
        "neo4j_pass": row.get("neo4j_pass") == "YES",
        "default_limit": None,
        "node_types": node_types,
        "relationship_types": rels,
        "anchor_tags": tags,
        "result_definition_ref": row.get("resultDefinitionRef") or "",
        "retrieval_step": "memory",
        "pass_through": False,
        "property_key": prop,
        "taxonomy": None,
        "wave": row.get("wave") or None,
    }


def from_deep(row: dict) -> dict:
    tag = slug_from_neo4j(row.get("neo4j_object") or "", "tag:")
    return {
        "binary_id": row["binary_id"],
        "operator": row["operator"],
        "layer": "DEEP_TAG",
        "class": "TAG",
        "parent": row.get("parent_family") or "",
        "crate": row["crate"],
        "package": package_name(row["crate"]),
        "telemetry_fork": row.get("json_telemetry_fork") or "",
        "verify": row.get("VERIFY") == "T",
        "neo4j_pass": row.get("neo4j_pass") == "YES",
        "default_limit": None,
        "node_types": [],
        "relationship_types": ["TAGGED_AS"],
        "anchor_tags": [tag] if tag else split_list(row["operator"].replace("TAG.", "")),
        "result_definition_ref": row.get("resultDefinitionRef") or "",
        "retrieval_step": "memory",
        "pass_through": False,
        "property_key": None,
        "taxonomy": row.get("taxonomy") or None,
        "wave": row.get("wave") or None,
    }


CARGO_TOML = """# Generated. Unique operator crate; AriA is linked via aria-operator.
[package]
name = {name!r}
description = {desc!r}
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
rust-version.workspace = true
publish = false

[lib]
path = "src/lib.rs"

[[bin]]
name = {name!r}
path = "src/main.rs"

[dependencies]
aria-operator = {{ path = "../../aria-operator" }}

[lints]
workspace = true
"""

LIB_RS = """//! Unique operator {binary_id} ({operator}).
//!
//! Links the Aria transformer through `aria-operator` and emits closed
//! operator JSON (Binary Repository v1 / sheet 09) with the shared
//! `aria-telemetry-query-v1` spine under `telemetry`.

/// Catalog identity.
pub const BINARY_ID: &str = {binary_id};
/// Operator name on the closed envelope.
pub const OPERATOR: &str = {operator};
/// This crate's frozen spec (sheet row).
pub const SPEC: &str = include_str!("../spec.json");

/// Run this operator on a host JSON payload.
pub fn run(payload: &[u8]) -> Result<aria_operator::OperatorEnvelope, aria_operator::OperatorError> {{
    aria_operator::run_spec(SPEC, payload, &aria_operator::RunOpts::default())
}}
"""

MAIN_RS = """fn main() {{
    std::process::exit(aria_operator::bin_main({pkg}::SPEC));
}}
"""


def rust_ident(package: str) -> str:
    return package.replace("-", "_")


def write_crate(spec: dict) -> None:
    pkg = spec["package"]
    dest = OUT_OPS / pkg
    dest.mkdir(parents=True, exist_ok=True)
    (dest / "src").mkdir(exist_ok=True)
    (dest / "spec.json").write_text(json.dumps(spec, indent=2, sort_keys=True) + "\n")
    desc = f"Aria operator binary {spec['binary_id']} ({spec['operator']})"
    (dest / "Cargo.toml").write_text(
        CARGO_TOML.format(name=pkg, desc=desc)
    )
    ident = rust_ident(pkg)
    (dest / "src" / "lib.rs").write_text(
        LIB_RS.format(
            binary_id=json.dumps(spec["binary_id"]),
            operator=json.dumps(spec["operator"]),
        )
    )
    (dest / "src" / "main.rs").write_text(MAIN_RS.format(pkg=ident))


def main() -> None:
    specs = [from_family(r) for r in rows("01_BINARY_CATALOG")]
    specs += [from_residual(r) for r in rows("11_RESIDUAL_BINARIES")]
    specs += [from_deep(r) for r in rows("14_DEEP_TAG_BINARIES")]
    specs.sort(key=lambda s: s["binary_id"])
    crates = [s["crate"] for s in specs]
    packages = [s["package"] for s in specs]
    ids = [s["binary_id"] for s in specs]
    assert len(specs) == 535, len(specs)
    assert len(set(crates)) == 535, "crate names not unique"
    assert len(set(packages)) == 535, "package names not unique"
    assert len(set(ids)) == 535, "binary_id not unique"

    OUT_CATALOG.parent.mkdir(parents=True, exist_ok=True)
    OUT_CATALOG.write_text(json.dumps(specs, indent=2, sort_keys=True) + "\n")

    if OUT_OPS.exists():
        for child in OUT_OPS.iterdir():
            if child.is_dir() and (child / "Cargo.toml").exists():
                # regenerated below
                pass
    OUT_OPS.mkdir(exist_ok=True)
    for spec in specs:
        write_crate(spec)

    readme = OUT_OPS / "README.md"
    readme.write_text(
        "# Operator binaries\n\n"
        f"{len(specs)} distinct crates. Each crate is one Binary Repository v1 "
        "operator. AriA (`telemetry::transform`) is linked into every binary "
        "via `aria-operator`. The closed operator JSON is unique; the nested "
        "`telemetry` object is the shared `aria-telemetry-query-v1` spine.\n\n"
        "Regenerate: `python3 crates/aria-operator/generate.py`\n"
    )
    print(f"wrote {len(specs)} crates -> {OUT_OPS}")
    print(f"catalog -> {OUT_CATALOG}")


if __name__ == "__main__":
    main()
