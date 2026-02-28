# Claude Context

## leptos_router Quirks

- `leptos_router` has no `csr` feature — do not add `features = ["csr"]` in
  Cargo.toml or the build will fail
- The `A` component does not accept a `class` prop — use `use_location()` from
  `leptos_router::hooks` to detect the active route and apply classes to inner
  elements instead

## Deployment

GitHub Actions builds on push to main and deploys `dist/` to Cloudflare Pages.
Required GitHub secrets: `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`.
The first workflow run creates the Pages project; subsequent runs skip that step
silently via `continue-on-error: true`.
