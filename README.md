# Footical

A Leptos CSR web app for subscribing to Football Mundial fixture calendars via iCal links.
Deployed as a static site on Cloudflare Pages.

## External Links

- https://footballmundial.com

## To Do

- [ ] Generate `mundial_league_groups.json`, `mundial_leagues.json`, and `mundial_teams.json`
  from PostgreSQL at build time via a `build.rs` script
- [ ] Generate per-team ICS calendar files from PostgreSQL at build time and upload to a CDN
