---
name: repo-architecture
description: Enforces strict separation between public-facing web directories (like docs/ for GitHub Pages) and internal repository files.
---

# REPOSITORY ARCHITECTURE & SECURITY RULES

You MUST strictly separate public-facing files from internal development files. 
When a repository uses a directory like `docs/` or `public/` to serve a static website (e.g., GitHub Pages), you are FORBIDDEN from placing internal files inside it.

## 1. Web / LP Directories (`website/`, `public/`, `lp/`)
These directories must ONLY contain files that are meant to be served publicly to end-users via the web.
- **ALLOWED**: `index.html`, `style.css`, `script.js`, `assets/`, optimized images.
- **STRICTLY BANNED**: Internal markdown documentation (`USER_GUIDE.md`, `ARCHITECTURE.md`), AI configuration files, lockfiles, scripts, `.agents/`, `.github/`, source code for the backend.
- **THE `docs/` BAN FOR WEB**: Do NOT hijack the `docs/` folder to host a Landing Page or Web App just because GitHub Pages legacy settings support it. `docs/` is strictly for real documentation (Markdown files, Sphinx, Docusaurus). For marketing pages, use `website/` or `public/`.

## 2. Internal Directories (Project Root)
Files meant for developers, AI agents, or repository management must reside at the project root or in dedicated internal folders, NEVER inside the public web directory.
- **Agent Skills**: MUST be at `<root>/.agents/skills/`. Never place them in `docs/.agents/`.
- **Documentation**: Developer docs like `README.md`, `USER_GUIDE.md`, or architecture notes belong at the repository root.
- **Config & State**: Files like `skills-lock.json`, `package.json`, or environment variables must stay at the root.

## Enforcement
Before creating or moving any file, ask yourself: "Will this directory be exposed to the public internet via a static site host?" If yes, and the file is internal, you are committing a critical security and architectural failure.
