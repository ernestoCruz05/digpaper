# DigPaper - AI Coding Instructions

## Project Overview
Document management system for a carpentry business ("Obras"). Three-component architecture:
- **Rust Backend** (`src/`) - Axum REST API with SQLite, serving API at `/api/*` and files at `/files/*`
- **Flutter Mobile** (`mobile/`) - Employee-facing app for field document capture
- **React Web** (`web-react/`) - Office staff PWA for document organization (primary web interface)

> Note: `web/` contains the built React app for Docker. Flutter web is obsolete.

## Architecture Patterns

### Backend Layered Structure
```
src/
├── handlers/     # Thin HTTP layer - extract params, call service, return JSON
├── services/     # Business logic - file processing, DB operations
├── models.rs     # Separated: Domain entities, Request DTOs, Response DTOs
├── error.rs      # Unified AppError with IntoResponse for Axum
└── db.rs         # Pool initialization + inline migrations (no files)
```

**Key pattern**: Handlers never touch the database directly—always delegate to services. See `document_handlers.rs` calling `DocumentService::upload()`.

### Inbox Workflow (Core Domain)
Documents follow: **Upload → Inbox → Assign to Project**
- `POST /api/upload` creates document with `project_id = NULL` (Inbox state)
- `PATCH /api/documents/:id/assign` moves document to a project
- Projects have `status`: `ACTIVE` (ongoing) or `ARCHIVED` (completed)

### File Handling
- Uploads stream directly to disk (`./uploads/`) to prevent memory exhaustion
- Filename format: `YYYY-MM-DD_HH-MM-SS_xxxx.ext` (date + 4-char UUID suffix)
- Files served via `ServeDir` at `/files/:filename`

## Build & Run Commands

```bash
# Backend (Rust)
cargo run                          # Dev mode (creates ./digpaper.db)
cargo build --release              # Production binary at ./target/release/digpaper

# Mobile (Flutter)  
cd mobile && flutter run           # Run on connected device
cd mobile && flutter build apk     # Android release

# Web (React + Vite)
cd web-react && npm run dev        # Dev server at localhost:5173
cd web-react && npm run build      # Output to ./dist, then copy to ./web for Docker
```

## Deployment
Production runs on server at `~/docker/digpaper`:
```bash
git pull                           # Get latest changes
docker compose build               # Rebuild container
docker compose up -d               # Deploy with new build
```
Traffic flow: **Cloudflare Tunnel → Nginx Proxy Manager → Docker container (port 3000)**

The container uses `web-public` external Docker network for reverse proxy access.

## Environment Variables
| Variable | Purpose | Default |
|----------|---------|---------|
| `DATABASE_URL` | SQLite path | `sqlite:./digpaper.db?mode=rwc` |
| `APP_API_KEY` | API authentication | Empty = no auth (dev mode) |
| `RUST_LOG` | Logging level | `digpaper=debug,tower_http=debug` |

## Code Conventions

### Rust
- Use `AppError` for all fallible operations—it implements `IntoResponse`
- Handler return type: `AppResult<Json<T>>` or `AppResult<(StatusCode, Json<T>)>`
- SQL migrations are inline in `db.rs` using `CREATE TABLE IF NOT EXISTS`
- UUIDs as `String` (not `Uuid` type) for SQLite compatibility

### Flutter
- State management: Provider pattern (see `pubspec.yaml`)
- API calls through `services/api_service.dart`
- Platform-specific upload: `upload_native.dart` vs `upload_web.dart`

### React
- Single-file app in `App.jsx` (~900 lines)—no component splitting yet
- API wrapper `apiFetch()` adds `X-API-Key` header from localStorage
- Image compression before upload (1280px max, 60% quality)

## API Authentication
Header-based: `X-API-Key: <value>` where value matches `APP_API_KEY` env var.
If `APP_API_KEY` is empty/unset, API is unprotected (development convenience).

## Key Files to Reference
- [src/main.rs](src/main.rs#L115-L145) - Router setup and API endpoint listing
- [src/services/document_service.rs](src/services/document_service.rs) - Streaming upload logic
- [src/error.rs](src/error.rs) - Error handling pattern
- [mobile/lib/services/api_service.dart](mobile/lib/services/api_service.dart) - Mobile API client
- [web-react/src/App.jsx](web-react/src/App.jsx) - Full web app implementation
