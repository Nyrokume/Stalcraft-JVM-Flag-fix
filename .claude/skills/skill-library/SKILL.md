# Skill library — Stalcraft JVM Wrapper

DAILY skills load every session for this repo. LIBRARY skills stay searchable; invoke manually when needed.

## DAILY (Rust / Tauri / Windows desktop)

| Skill | Evidence |
|-------|----------|
| `rust-patterns` | `src-tauri/src/*.rs`, `Cargo.toml` |
| `rust-testing` | `#[cfg(test)]` in `jvm.rs`, `log.rs`, `system.rs` |
| `windows-desktop-e2e` | IFEO, Win32 APIs, Tauri Windows bundle |
| `verification-before-completion` | `cargo test`, `npm run build:prod` before release |
| `ponytail` | minimal diffs, YAGNI |

## LIBRARY (off-stack — search when needed)

| Skill | Evidence |
|-------|----------|
| `react-patterns`, `vue-patterns`, `nextjs-*` | vanilla JS in `src/main.js`, no framework |
| `django-patterns`, `springboot-*`, `laravel-*` | no Python/PHP/Java backend |
| `flutter-reviewer`, `kotlin-*` | no mobile code |
| `supabase-postgres-best-practices` | no database layer |

## Rules

- No TypeScript: skip `rules/typescript/*` as DAILY.
- No Python/Go hooks in repo: do not install language-specific ECC hooks.

## Triggers

- **IFEO / JVM / G1 / heap** → app code in `stalcraft-jvm-wrapper/src-tauri/`
- **Hardware detection** → `system.rs`
- **GUI** → `src/index.html`, `src/main.js`, `src/assets/styles.css`
