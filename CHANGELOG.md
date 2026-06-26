# Changelog

## [0.3.2](https://github.com/matsumotory/claude-desktop-switcher/compare/v0.3.1...v0.3.2) (2026-06-26)


### Bug Fixes

* ci build and rust warnings ([22ec03b](https://github.com/matsumotory/claude-desktop-switcher/commit/22ec03bee857c2ab0d4366619c75a8a46cff766d))
* gh pr merge requires repository arg ([8bdef4b](https://github.com/matsumotory/claude-desktop-switcher/commit/8bdef4b3465e99839f4e92bd254cbda49aa4235a))
* make mock keychain use static global store for integration tests ([f3164d9](https://github.com/matsumotory/claude-desktop-switcher/commit/f3164d9c276fa5d88601ec1b493e480ef7d8c4bf))

## [0.3.1](https://github.com/matsumotory/claude-desktop-switcher/compare/v0.3.0...v0.3.1) (2026-06-26)


### Bug Fixes

* build issues (RGBA icon and type annotations) ([db83113](https://github.com/matsumotory/claude-desktop-switcher/commit/db831131ba6b930dcf8bd0f65685507112a0ed35))
* exclude desktop crate from cargo test, fix CLI Arc type annotation ([266ae2b](https://github.com/matsumotory/claude-desktop-switcher/commit/266ae2b2ce0d42aa217690e0e404fda0f3891f15))
* expose mock module via test-utils feature for integration tests ([369e2ed](https://github.com/matsumotory/claude-desktop-switcher/commit/369e2ed55f806d815886fe7b570f935def985811))

## [0.3.0](https://github.com/matsumotory/claude-desktop-switcher/compare/v0.2.2...v0.3.0) (2026-06-26)


### Features

* auto-merge release-please PRs ([af1f9fc](https://github.com/matsumotory/claude-desktop-switcher/commit/af1f9fceeac09eaef7bafdfeacc1bc0b4c08ce5f))


### Bug Fixes

* add missing crate::error::Result import to profile/mod.rs ([c27494b](https://github.com/matsumotory/claude-desktop-switcher/commit/c27494bd692212229d6ed212a69cb80a46453252))
* add notify::Error to CswError ([82d5457](https://github.com/matsumotory/claude-desktop-switcher/commit/82d54575774cebe4d171662a4fb1a5aa1b6f94b1))
* make PlatformProvider Send + Sync ([226ec4b](https://github.com/matsumotory/claude-desktop-switcher/commit/226ec4b3d8be567e1cbcd52362fc8f36865b3987))
* remove invalid devUrl from tauri.conf.json ([cd9b7ba](https://github.com/matsumotory/claude-desktop-switcher/commit/cd9b7baef000202d93a25076d54304725f8c3026))
* revert tauri-action version to v0 to fix CI ([d61dad3](https://github.com/matsumotory/claude-desktop-switcher/commit/d61dad3017d0aa885498e610971b1c384feba83e))

## [0.2.2](https://github.com/matsumotory/claude-desktop-switcher/compare/v0.2.1...v0.2.2) (2026-06-26)


### Bug Fixes

* remove npm install from workflows since there is no package.json ([9871cbe](https://github.com/matsumotory/claude-desktop-switcher/commit/9871cbeea8189d2ff789881217c2aa5e56bccde7))
* resolve notify API and compilation errors, correct CI tauri-action version, and add rigorous SPECIFICATION ([a355377](https://github.com/matsumotory/claude-desktop-switcher/commit/a355377ce6b543075fd8260560eecc82af312f3d))

## [0.2.1](https://github.com/matsumotory/claude-desktop-switcher/compare/v0.2.0...v0.2.1) (2026-06-26)


### Bug Fixes

* update tauri-action to v0 and implement testing architecture ([462a487](https://github.com/matsumotory/claude-desktop-switcher/commit/462a4871600f6dd6a932dd5916f3daf92c88981c))

## [0.2.0](https://github.com/matsumotory/claude-desktop-switcher/compare/v0.1.0...v0.2.0) (2026-06-26)


### Features

* complete file watcher implementation and update README to remove phasing ([2216eec](https://github.com/matsumotory/claude-desktop-switcher/commit/2216eec5616bcabdea0cab0cbffaf1204db10687))
* implement 2026 agentic structure, sync EN LP to Bento, recreate docs/, ensure pixel-perfect assets, and fix Release Please permissions ([3d8f93a](https://github.com/matsumotory/claude-desktop-switcher/commit/3d8f93a4a07e76b68def5de051eec0a348f43968))
* implement core switcher logic, profile manager, keychain backup/restore, and cli skeleton ([d9617ab](https://github.com/matsumotory/claude-desktop-switcher/commit/d9617ab5505aba521deab53eea784cac41623708))
* implement Phase 2 Tauri v2 desktop menu bar app with settings UI ([c353da0](https://github.com/matsumotory/claude-desktop-switcher/commit/c353da098309188cd2b778b4a7e46cc1befc6d7f))
* implement release-please for automated semantic versioning, add tauri v2 AI skill ([119b61c](https://github.com/matsumotory/claude-desktop-switcher/commit/119b61c036e20f69b66a7a7ec3d434cb465d92a9))
* inject CLAUDE_CONFIG_DIR into Claude Desktop env for CLI inheritance ([b826de6](https://github.com/matsumotory/claude-desktop-switcher/commit/b826de6fa62ba02fca0f32657ef150e0f1bcce54))
* redesign LP with TasteSkill and add GitHub Pages workflow ([e2654e4](https://github.com/matsumotory/claude-desktop-switcher/commit/e2654e4d145589874cdab77b37811c63ae00a6a4))


### Bug Fixes

* modernize CI to Node 24 and Tauri v2, add bilingual LP with user guide ([cf20eb7](https://github.com/matsumotory/claude-desktop-switcher/commit/cf20eb7ffc8ae0c603a108e14c40b18b032255b4))
* resolve release-please workspace version parsing issue ([58796ee](https://github.com/matsumotory/claude-desktop-switcher/commit/58796eef3a318655f8d87882a77a4d773e94a4cc))
* Update GitHub Pages action path to website/ and add MIT License ([34d6c09](https://github.com/matsumotory/claude-desktop-switcher/commit/34d6c09eab16cdd60acd093b1e695e3686a8beb7))
