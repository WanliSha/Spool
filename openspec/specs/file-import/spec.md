# File Import Spec

## Approach
File-explorer style — no import database, no project concept. Open files directly from their original location, edit EXIF, done.

## Import Methods
- Drag and drop files
- Drag and drop folder
- File picker (select files or folder)

## Folder Behaviour
- Default: load only the top-level folder (no recursion)
- Option in settings: "Load subfolders when opening a folder" (off by default)

## File Handling
- Files are read from their original location, not copied
- No database or index is created
- No state is persisted after closing the app

## Supported Formats
All image formats supported by the `image` crate and `libraw-rs`:
- JPEG, TIFF, PNG, WebP, BMP
- RAW: CR2, CR3, NEF, ARW, RAF, DNG, ORF, RW2, etc.
