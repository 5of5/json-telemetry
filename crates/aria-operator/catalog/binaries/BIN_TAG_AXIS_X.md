# BIN.TAG.AXIS_X

**Operator:** `TAG.AXIS_X` · **layer:** RESIDUAL · **class:** TAG · **parent:** MARKET_MAP
**Crate:** `aria-res-tag-axis-x` · **verify:** True · **result:** `residual.tag.axis_x`

## Why

Listed Binary Repository v1 identity. Independent calculation; no borrowed scores.

## Function

Tag operator. Does not name a new entity. Tags or reads an existing node. Role-tags (BUYER/COMPETITOR/…) require the specific tag on the record.

## Use

`work --binary BIN.TAG.AXIS_X` or `work --json` with `ops: ["BIN.TAG.AXIS_X"]`

Production callback: working vertical or nothing. Forget is not delete — original payload stays in `source`.

## Declared neighborhood

- node types: —
- relationships: TAGGED_AS
- anchor tags: AXIS_X
- property key: —

## Maps that consume this

- (not a primary binary on sheet 05, or is itself a map mixer)

## Important

- No Trust / Use / Goal fields.
- Does not rewrite other binaries' verticals.
- Empty declared types → omitted from `aria-work-v1` results.
