# Templates

Templates can be built in or imported from ZIP archives.

## Built-in Templates

- Next.js App Router
- Next.js + Tailwind
- Vite React
- Static HTML/CSS/JS
- PHP

## ZIP Templates

ZIP files can be selected with the native picker, pasted as a path or dropped into the import zone.

Requirements:

- file extension must be `.zip`;
- archive must be 100 MB or smaller;
- archive may contain up to 2000 files;
- uncompressed content may be up to 250 MB;
- archive entries must stay inside the target folder;
- absolute paths and `..` traversal are rejected;
- `template.json` must exist at the ZIP root;
- `files/` must exist at the ZIP root and contains the project files;
- `template.json` must use a supported project type and package manager.

Example:

```txt
template.zip
  template.json
  files/
    package.json
    src/
    public/
```

```json
{
  "name": "Next.js App Router Starter",
  "type": "next",
  "version": "1.0.0",
  "author": "Farid Leonov",
  "description": "Clean starter for Local Dev Studio",
  "defaultPort": 3000,
  "packageManager": "pnpm",
  "requiresInstall": true
}
```
