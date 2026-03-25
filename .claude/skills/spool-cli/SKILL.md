---
name: spool-cli
description: Use when the user mentions photos, images, or pictures and wants to annotate, tag, read, write, or edit metadata — including dates, timestamps, GPS location, camera/lens info, keywords, ratings, or any EXIF/XMP/IPTC fields. Also use when the user asks to read date imprints from film photos.
metadata:
  pattern: [tool-wrapper, pipeline]
  tools: [spool-cli, bash, read]
---

# Spool CLI — Photo Metadata Editor

You have access to the `spool` CLI for reading and writing photo metadata (EXIF, XMP, IPTC).

## Finding the Binary

Try `spool` first (if in PATH). If not found, try in order:
- `~/.local/bin/spool`
- **macOS**: `/Applications/Spool.app/Contents/MacOS/spool`
- **Linux**: `/usr/bin/spool` (deb) or alongside the AppImage
- **Dev build**: `cargo run -p spool-cli --`

## Commands

```
spool list <directory> [--recursive]
spool get <file> [field] [--json]
spool set <file>... --field <field> --value <value>
spool set <file>... --json '{"Field":"value",...}'
spool preview <file> [--rotate <90|180|270>]
```

Multiple files can be specified for `set` — the same fields are written to all files:
```bash
spool set IMG_001.jpg IMG_002.jpg IMG_003.jpg --json '{"CameraMake":"Nikon","CameraModel":"FM2"}'
spool set IMG_001.jpg IMG_002.jpg --field DateTaken --value "2024-12-25"
```

`spool list` returns one file path per line, sorted by filename.

## Trigger Examples

Should activate:
- "Tag these photos with dates and GPS"
- "What camera was this taken with?"
- "Set metadata for this roll to Nikon FM2"
- "Read the date imprint on this film scan"
- "Set rating to 5 for all photos in this folder"

Should NOT activate:
- "Fix the white balance on this photo" (image editing, not metadata)
- "Convert these photos to PNG" (format conversion)
- "Use AI to identify faces in these photos" (content analysis)

## Query vs Annotation — choose the right mode

Not every request needs the full SOP. Decide which mode to use:

**Query mode** — user just wants to see info, no changes:
- "What metadata does this photo have?"
- "What camera was this taken with?"
- Just run `spool get <file> [--json]` and report the results. Done.

**Annotation mode** — user wants to write/change metadata:
- "Tag these photos with dates and GPS"
- "Set the camera info for this roll"
- Follow the full SOP below.

If unsure, start with a quick `spool get` to show current state, then ask if they want to make changes.

## Handling Existing Metadata

Before writing ANY field, check what's already there with `spool get <file> --json`.

- **Field is empty** → write freely
- **Field already has a value** → show the existing value and ask before overwriting
- **Field has the same value you're about to write** → skip, no action needed

Never silently overwrite existing metadata. The user may have annotated these photos before.

## Large Batch Strategy

When dealing with many photos, don't try to visually inspect every single one — that will exhaust the context window.

**Up to ~20 photos**: Read each one individually, full visual inspection.

**20–100 photos**: Sample strategy:
1. `spool list` to get the full file list
2. `spool get --json` on a few files to check existing metadata
3. Visually read the first 3-5 and last 2-3 photos to establish date range
4. Use frame numbers, filenames, and film strip grouping to infer the rest
5. Present the full plan (with inferred entries marked as "likely") for confirmation

**100+ photos**: Ask the user how to proceed:
- "There are 156 photos. Want me to sample a few to determine the pattern, or do you have the details (camera, dates, location) ready?"

## Film Photography Hints

Use these clues to infer metadata more accurately:

**Roll structure:**
- Standard rolls: 24 or 36 exposures
- Files from the same roll often share sequential frame numbers
- Same roll = same camera, same lens, same ISO, usually same day or trip
- If filenames contain strip/frame info (e.g., `Negative1-04-01`), frames on the same strip are consecutive shots

**Date imprint patterns by era/brand:**
- Japanese cameras (Nikon, Canon, etc.): `'98 10 12`, `'24 12 25`
- Some cameras: `OCT.12.1998`, `12/25/2024`
- European format: `25.12.2024`
- Always bottom-right corner, orange/red text, sometimes rotated with the photo

**Film stock identification:**
- Film border text: `KODAK 5063`, `FUJI SUPERIA`, `ILFORD HP5`
- Edge markings and DX codes can identify film stock
- Color negative (C-41): orange base, Kodak/Fuji/etc.
- B&W: clear or grey base, Ilford/Kodak Tri-X/etc.
- Slide (E-6): positive image on film, Fuji Velvia/Provia, Kodak Ektachrome

**What you can infer from visual clues:**
- Same clothing on subjects → likely same day
- Lighting changes (indoor → outdoor) → same day, different time
- Same background/location → same outing
- Baby/child growth between rolls → rough date ordering

## Standard Operating Procedure (Annotation Mode)

For ANY metadata annotation task, follow these steps:

### Step 1: Gather — collect all information

Load `references/fields.md` to know the available fields and their formats.

Gather metadata from all available sources:
- **User-provided info**: what the user told you (camera, location, date, etc.)
- **Visual inspection**: read photos with the Read tool to spot date imprints, visible landmarks, film borders, frame numbers
  - If a photo appears rotated or upside down, use `spool preview <file>` to generate a correctly oriented temp copy, then read that instead. The preview auto-corrects EXIF orientation. Use `--rotate 90/180/270` for manual rotation if needed. **Preview is view-only — it never modifies the original file.**
- **Existing metadata**: run `spool get <file> --json` to see what's already there. Flag any fields that already have values.
- **Inference**: what you can reasonably deduce (see Film Photography Hints above)

DO NOT proceed to Step 2 until you have inspected all relevant photos and collected all available information.

### Step 2: Classify — sort every field by confidence

Assign each piece of information to one of three categories:

| Confidence | Meaning | Example |
|---|---|---|
| **confirmed** | User explicitly stated, or clearly readable | User said "Nikon FM2"; date imprint reads "2024.12.25" clearly |
| **likely** | Strong evidence but not 100% certain | Date imprint partially faded but reads "2024.12.2_" with 5 most probable; GPS for "Taipei 101" resolves to well-known coordinates; inferred from same roll as a confirmed photo |
| **uncertain** | Guessing, ambiguous, or conflicting info | Can't tell if last digit is 6 or 8; user said "somewhere in Taipei" without a specific spot |

### Step 3: Present — show the annotation plan

Present ALL gathered information in a single summary table, grouped by confidence:

```
## Annotation Plan

### Confirmed (will write)
File           | Field       | Value              | Source
IMG_001-003    | CameraMake  | Nikon              | user stated
IMG_001-003    | CameraModel | FM2                | user stated
IMG_001-003    | ISO         | 400                | user stated
IMG_001.jpg    | DateTaken   | 2024-12-25         | date imprint (clear)
IMG_002.jpg    | DateTaken   | 2024-12-26         | date imprint (clear)

### Likely (need confirmation)
File           | Field       | Value              | Reason
IMG_003.jpg    | DateTaken   | 2024-12-25         | imprint faded, best guess from "2024.12.2_"
IMG_001-003    | GPSLatitude | 25.0340            | "Taipei 101" — well-known landmark
IMG_001-003    | GPSLongitude| 121.5645           | "Taipei 101" — well-known landmark

### Uncertain (skipping unless you decide)
File           | Field       | Value              | Issue
IMG_004.jpg    | DateTaken   | ?                  | no date imprint found
IMG_005.jpg    | DateTaken   | 2024-12-26 or 28   | last digit unclear

### Existing values (will NOT overwrite unless you say so)
File           | Field       | Current Value      | New Value
IMG_006.jpg    | Author      | "John"             | "Arthur" (user stated)
```

DO NOT proceed to Step 4 until the full annotation plan is presented to the user.

### Step 4: Confirm — get user approval

- **Confirmed**: "I'll write these now unless you object."
- **Likely**: "These look right to me — OK to write?" (present as a group, user can approve all or pick)
- **Uncertain**: "I need your help with these." (ask specific questions)
- **Existing values**: "These files already have values. Overwrite?" (list each conflict)

DO NOT proceed to Step 5 until the user has explicitly approved the writes. Wait for confirmation.

### Step 5: Write — execute in batch

ONLY AFTER user confirmation:
1. Write all approved fields using `spool set` with `--json` for efficiency
2. Group files that share the same values into single commands
3. Report what was written

```bash
# Batch: same metadata for multiple files
spool set IMG_001.jpg IMG_002.jpg IMG_003.jpg --json '{"CameraMake":"Nikon","CameraModel":"FM2","ISO":"400"}'

# Per-file: different dates
spool set IMG_001.jpg --field DateTaken --value "2024-12-25"
spool set IMG_002.jpg --field DateTaken --value "2024-12-26"
```

### Step 6: Verify — confirm what was written

Run `spool get <file> --json` on a sample file to verify the write succeeded. Report back:

```
Done: 3 files updated: CameraMake, CameraModel, ISO
Done: 2 files updated: DateTaken
Skipped: 1 file skipped: IMG_004.jpg (no date — user to decide later)
```

DO NOT proceed to Step 7 unless verification succeeded for all written files.

### Step 7: Organize (Optional)

Only offer this step if the user has more than 5 files in the batch.

After annotation is complete, offer the user organization options:

**Option A: Move files into folders by a metadata field**

Ask: "Want me to organize these files into folders? I can sort by:"
- **Date**: `2024-12-25/`, `2024-12-26/`, ...
- **Camera**: `Nikon FM2/`, `Canon AE-1/`, ...
- **Location**: `Taipei/`, `Tokyo/`, ...
- **Film stock**: `Portra 400/`, `HP5/`, ... (if spool:FilmStock is set)
- **Custom**: any combination the user specifies

Example result:
```
photos/
├── 2024-12-25/
│   ├── IMG_001.jpg
│   └── IMG_003.jpg
├── 2024-12-26/
│   └── IMG_002.jpg
└── unsorted/
    └── IMG_004.jpg    ← no date, couldn't classify
```

Files that lack the sorting field go to an `unsorted/` folder.

**Option B: Decide what still needs annotation**

For files that were skipped or had uncertain fields, present a summary:

```
## Still needs attention

Needs date:
  - IMG_004.jpg (no date imprint found)
  - IMG_005.jpg (unclear: 2024-12-26 or 28)

Needs GPS:
  - IMG_006.jpg (no location provided)

Fully annotated:
  - IMG_001.jpg, IMG_002.jpg, IMG_003.jpg
```

Ask: "Want to work through the incomplete ones now, or save them for later?"

**Implementation notes:**
- Use `mkdir -p` and `mv` via Bash to move files
- Always confirm the target folder structure BEFORE moving anything
- If files would overwrite existing files, warn and ask
- After moving, run `spool get` to verify metadata survived the move (it should — metadata is embedded in the file)
