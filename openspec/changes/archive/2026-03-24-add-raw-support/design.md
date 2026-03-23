# Add RAW Support — Design

## Approach

Use `rawler` crate (pure Rust, LGPL-2.1) for RAW decoding. It's from the dnglab project and supports many RAW formats including DNG, CR2, CR3, NEF, ARW, RAF, ORF, RW2.

No C dependencies needed — simpler CI and cross-platform build.

## Integration Points

RAW decoding is needed in three places:
1. **Thumbnail** (`thumbnail.rs`): decode RAW → resize to 320px
2. **Quick preview** (`preview.rs`): decode RAW → resize to 2048px
3. **Full preview** (`preview.rs`): decode RAW at full resolution

## Strategy

Create a shared `decode_image` function that:
1. Checks file extension
2. RAW format → use `rawler` to decode to RGB buffer → convert to `image::DynamicImage`
3. Standard format → use `image::open` as before

Both thumbnail.rs and preview.rs call this shared function instead of `image::open` directly.

## Crate

`rawler` — decodes RAW to raw pixel data. We convert the output to `image::DynamicImage` for the existing resize/encode pipeline.
