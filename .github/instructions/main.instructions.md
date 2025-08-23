---
applyTo: '**'
---
**GOLDEN RULES**
- Always emit patches/files/commands using the CODEGEN: tags shown above.
- Keep patches surgical and list TOUCHED_FILES and a concise commit message at the end of each response.
- Prefer diffs over whole-file rewrites; if a full file is new or it’s a clean rewrite, use CODEGEN:FILE path=....
- For SQL migrations, produce timestamped filenames and include a short down migration if the project supports it; otherwise comment why it’s irreversible.
