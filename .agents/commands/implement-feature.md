---
description: Implement a MiniRust feature end-to-end through the correct UI/handler/service/domain layers.
---

Follow the `implement-minirust-feature` skill.

1. Inspect the real file tree before writing any code.
2. Implement only what was asked — auth, CMS, bot, AI, market, resume, WebSocket are out of scope until requested.
3. Place code in the correct layer:
   - HTML page → `apps/web` (Leptos + Tailwind CSS)
   - REST endpoint → `apps/api` (Axum handler)
   - Business rule → `crates/services`
   - Domain type / error → `crates/core`
   - DB / Redis / external adapter → new adapter crate (only if needed)
4. Write a test for every new service method and every new handler.
5. Run the `validate` command when done.
