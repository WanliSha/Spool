# Licensing Spec

## Decision

Spool uses a **dual-licensing** model:

| License | Audience | Terms |
|---|---|---|
| **GPLv3** | Open source users | Free. Derivative works must also be GPLv3. |
| **Commercial** | Companies wanting closed-source use | Paid. Contact maintainer for terms. |

## Rationale

- GPLv3 protects the codebase from being closed-source forked without contributing back
- Commercial license preserves monetization and acquisition options
- The project owner retains full copyright, enabling re-licensing at any time
- Tauri (MIT) and Leaflet (BSD-2) impose no license restrictions on Spool

## Repository Files

### `LICENSE`
- Full GPLv3 text
- Standard copy from https://www.gnu.org/licenses/gpl-3.0.txt

### `LICENSE-COMMERCIAL.md`
- Brief description of what the commercial license allows:
  - Use in proprietary/closed-source products
  - Distribute without GPL obligations
- Contact email for inquiries
- No formal contract needed until a buyer appears

### `README.md` License Section
```
## License

Spool is dual-licensed:

- **Open Source**: [GPLv3](./LICENSE)
- **Commercial**: For closed-source or proprietary use, see [Commercial License](./LICENSE-COMMERCIAL.md)
```

## CLA (Contributor License Agreement)

### Why
External contributions must be covered by a CLA so the project owner can issue commercial licenses for all code in the repository.

### When
Must be set up **before merging the first external PR**.

### How
- Use [CLA Assistant](https://cla-assistant.io/) (free GitHub integration)
- Automatically prompts contributors to sign when they open a PR
- Core clause: contributor grants the project maintainer an irrevocable, worldwide, royalty-free license to their contribution, including the right to sublicense

## Timeline

| Phase | Action |
|---|---|
| **Now** | Add `LICENSE` (GPLv3) and `LICENSE-COMMERCIAL.md` to repo |
| **Before first external PR** | Set up CLA via cla-assistant.io |
| **When a company inquires** | Engage a lawyer to draft a formal commercial license agreement |
