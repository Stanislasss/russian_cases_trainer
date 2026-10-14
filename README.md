# Russian Cases Trainer

A web app for practising the six Russian grammatical cases: browse the declension rules case by case, then drill them with games (e.g. drag the right ending onto a noun stem).

- **Backend:** Rust (axum + sqlx), hexagonal workspace in [`backend/`](backend)
- **Frontend:** React + TypeScript + Vite in [`frontend/`](frontend)
- **Database:** PostgreSQL 17
- **UI languages:** English, French, Russian

## Getting started

Requires Docker Desktop and [`just`](https://github.com/casey/just).

```sh
cp .env.example .env
just up        # db, migrations, backend (:8080), frontend (:5173)
just import    # load the noun lexicon and grammar rules
```

Open http://localhost:5173.

| Command | What it does |
|---|---|
| `just up` / `just down` | Start / stop the stack |
| `just logs [service]` | Follow logs |
| `just migrate` | Apply database migrations |
| `just adminer` | Database web UI on http://localhost:8081 |
| `just test` | Run the backend tests in a Rust container |
| `just check` | Backend fmt check + clippy + tests |
| `just cargo <args>` | Any cargo command in a Rust container |
| `just gen-api` | Regenerate the TypeScript API types from the running backend |

Frontend checks run natively from `frontend/`: `npm run typecheck`, `npm run lint`, `npm test`.

## Backend layout

| Crate | Role |
|---|---|
| `domain` | Grammar types and pure logic, no I/O |
| `application` | Use cases and the ports (traits) they depend on |
| `infrastructure` | Postgres repositories, password hashing, migrations |
| `api` | axum routes, DTOs, OpenAPI document |
| `server` | Binary: config and wiring (`serve`, `migrate`, `print-openapi`) |
| `importer` | Binary: loads the OpenRussian word lists |

## Data attribution

Word data comes from [OpenRussian.org](https://en.openrussian.org), licensed under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).

## License

Apache-2.0
