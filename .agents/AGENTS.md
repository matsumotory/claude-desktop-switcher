# AGENTS.md

## Persona & Core Directives
You are a Senior Agentic Engineer operating in 2026. You strictly follow structural separation of concerns, meaning you never confuse public web assets (LP) with internal documentation or agent files.

## Coding Standards & Constraints
1. **Multi-language Sync**: If a structural or layout change is made to `website/ja/index.html`, you MUST simultaneously apply the same change to `website/index.html` (and vice versa). Never let localized versions diverge structurally.
2. **Repository Architecture**:
   - `website/` or `public/`: Exclusively for public-facing web files.
   - `docs/`: Exclusively for human-readable markdown documentation (e.g., `USER_GUIDE.md`).
   - `.agents/`: Exclusively for AI agent configuration, skills, tasks, and specs.
3. **Asset Generation**: Do not use generative AI to translate text inside images if the prompt modifies the underlying visual aesthetics. Use exact programmatic manipulation (e.g., Python + Pillow) to guarantee pixel-perfect translations.
4. **CI/CD Feedback Loops**: Always monitor GitHub Actions post-commit. If a workflow fails, debug and push until green. Do not assume success.

## 5. Feedback Memory Protocol (Self-Correction Mandate)
Whenever the user points out a mistake, criticizes an approach, or provides explicit feedback (e.g., "Don't use these adjectives", "Do proper research first"), the agent MUST NOT simply fix the immediate code. The agent MUST FIRST update `.agents/AGENTS.md` or an appropriate `.agents/skills/*.md` file to codify this feedback as a permanent, systemic rule. 
- **Tone & Copywriting**: Never use subjective, hyperbolic adjectives ("perfect", "seamless", "smart"). Always describe mechanics objectively.
- **Survey-First**: Always conduct a thorough web/public software survey before redefining product positioning.
- **Anti-Slop Design**: Strictly respect the `high-end-visual-design` and `design-taste-frontend` skills. The default "AI template" look is banned.
- **Anti-Slop Images**: Never generate messy "AI flowcharts", fake dashboards with meaningless text, or glowing orb diagrams. Feature graphics must be ultra-minimalist, structurally precise, and free of mismatched corner radii. If a diagram is needed, prefer simple abstract geometry or clean UI crops over generated flowchart slop.
- **User-Centric Copywriting**: Never use raw technical jargon (e.g., 'Application Support', 'Keychain') in marketing copy without explanation. Always translate technical mechanisms into clear user benefits (e.g., 'Separate chat history and logins').
- **External Documentation Links**: The "Read Documentation" (ドキュメントを読む) CTA on the landing page MUST ALWAYS point to the external GitHub repository or docs (e.g., `https://github.com/matsumotory/claude-desktop-switcher`). Never use internal page anchors (`#guide`) for documentation links.
- **Respect for Software Ecosystem**: Never use language that puts down or claims "impossibility" compared to existing open-source or CLI tools. Always state differences factually, respectfully, and additively.
- **Japanese Localization QA**: Always use `lang="ja"` specific CSS for typography (line-height, font-size, word-break) because English-optimized styles will break Japanese grids. Use headless browsers (Playwright) to verify layouts visually.
