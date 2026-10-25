LuLu Universal Attendance Management System

Attendance management. Licensed under GPL-3.0.

Supported devices:

1. ZKTeco (ADMS V2.1 and earlier)
2. Solution (Almost all)

Demo: https://lulu-demo.16pxlabs.com
User: demo
Password: demo12345

## Repository layout

    LuLu-BackEnd/     Rust API (Actix Web) + database migrations
    LuLu-FrontEnd/    Nuxt 4 SPA (Vue 3 + Pinia + Tailwind) - admin
    LuLu-Field/       Single-file employee PWA (served at /field-app)
    Readme.md         this file

## Requirements

- Rust edition 2024 (backend)
- Node.js 20+ and npm (frontend)
- A running PostgreSQL server

## Running the backend

    cd LuLu-BackEnd
    cp .env.example .env        # then fill in the values
    cargo run

Database migrations run automatically at server start (`sqlx::migrate!`), by filename
order. The server refuses to start if any migration fails.

Variables the code reads:

| Variable          | Required | Default                   | Notes                                                            |
| ----------------- | -------- | ------------------------- | ---------------------------------------------------------------- |
| `DATABASE_URL`    | yes      | -                         | PostgreSQL connection string                                     |
| `SECRET_KEY`      | yes      | -                         | Must be present; the server exits without it. See Security notes |
| `HOST`            | no       | `0.0.0.0`                 | Bind address                                                     |
| `PORT`            | no       | `4343`                    | Backend port                                                     |
| `ALLOWED_ORIGINS` | no       | loopback + private ranges | Comma-separated CORS allowlist; public origins need this set     |

Example `.env` (replace with your own environment's values):

    DATABASE_URL=postgres://USER:***@127.0.0.1:5432/lulu
    SECRET_KEY=replace-with-a-long-random-string
    HOST=0.0.0.0
    PORT=4343
    ALLOWED_ORIGINS=https://lulu.example.com

The backend refuses to start unless `SECRET_KEY` is set.

## Running the frontend

    cd LuLu-FrontEnd
    npm install
    npm run dev                # dev port 3232 by default

Production:

    npm run build
    node .output/server/index.mjs

The API address is derived from whatever host the browser used, so it resolves to
`http://<hostname>:4343/api`. That means `localhost`, a LAN IP, and a static domain all
work from the same build. Set `NUXT_PUBLIC_API_BASE` if the API lives on another origin.

## Publishing the demo build (static, no server)

The demo build swaps the real backend for a mock API that runs inside the browser, so it
can be published to static hosting with no Rust and no database:

    cd LuLu-FrontEnd
    npm run generate:demo

The result is in `.output/public`. The mock lives in `app/composables/mockApi.js`. That
build uses the demo credentials, so do not use it for real data.

## Security notes

Findings from a code audit, and which ones are now fixed:

- **Employee and admin accounts are separated.** `users.user_field_only` marks PWA
  accounts; the admin form refuses them, the PWA refuses admin accounts, and the
  page gate refuses a field token on every key except `field`.
- **Passwords are stored as Argon2id** (per-user random salt), rehashed on the next
  successful sign-in for any row still holding a legacy hash. Rows written before the
  migration stay verifiable in the meantime.
- **Session tokens expire after 12 hours of inactivity.** The window slides: an
  authenticated request pushes the expiry forward. Expired sessions stop authenticating and
  are swept after a grace period. Tokens are 32 random bytes from `getrandom`.
- **The session cookie is readable by JavaScript** (not `httpOnly`), so any XSS on the app's
  origin can lift the token.
- **CORS accepts any origin while allowing credentials.** This one is now closed: the API
  answers loopback and private-range origins by default (`127.0.0.1`, RFC1918, `::1`,
  `fc00::/7`, `fe80::/10`, `.local`, `.localhost`) and refuses every public origin, so a page
  on `attacker.example` cannot read a response with the operator's session attached. Setting
  `ALLOWED_ORIGINS` replaces that default with an explicit list - a bare host entry matches
  any port and scheme, a full origin has to match exactly. A `WARN` at startup says which mode
  is in effect.
- The backend **binds to `0.0.0.0` by default**, so it is reachable from the whole network.
- The demo user's password hash is _committed_ in `migrations/…_data_seed.sql`. That is
  deliberate (so a clean install has a demo account), but it means the demo credentials are
  publicly known.

## Frequently asked questions

**Q: Why is it called LuLu?**
A: Why not. It sounds cute, short, and is easy to remember.

**Q: My device is in the list but does not work.**
A: Check that the device matches the requirements and its configuration, then check
the LuLu server - it may be a firewall, or the program may not be started. If it still
fails, open an issue in the repository.

**Q: Will there be paid features?**
A: LuLu will always be free and open source.

**Q: Are you cat?**
A: Fix your brain please.

## Supporting

- ETH: `0x56AC2134df9357aaD3FF2C6d9B42Cb930708372E`
- Patreon: https://patreon.com/16PxLabs
- QRIS: coming soon
