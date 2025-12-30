# Charta - Document Management System

A high-performance REST API backend for a carpentry business document management system. Built with Rust for reliability and speed.

## Features

- **Projects (Obras)**: Organize work orders into logical units
- **Inbox Workflow**: Upload documents first, organize later
- **Streaming Uploads**: Memory-efficient file handling for large files
- **Static File Serving**: Direct access to uploaded images and PDFs
- **SQLite Database**: Portable, self-hosted, zero-configuration

## Tech Stack

- **Axum** - Fast, ergonomic web framework
- **SQLx** - Compile-time verified SQL with async support
- **SQLite** - Portable file-based database
- **Tokio** - Async runtime for high concurrency
- **Tower-HTTP** - CORS and request tracing middleware

## Getting Started

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))

### Running the Server

```bash
# Development mode
cargo run

# Production build
cargo build --release
./target/release/charta
```

The server starts at `http://localhost:3000`

## API Endpoints

### Projects

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/projects` | Create a new project |
| `GET` | `/projects` | List all projects |
| `GET` | `/projects?status=active` | List active projects only |
| `GET` | `/projects/:id` | Get project details |
| `GET` | `/projects/:id/documents` | List documents in project |

### Documents

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/upload` | Upload a file (multipart/form-data) |
| `GET` | `/documents/inbox` | List unassigned documents |
| `PATCH` | `/documents/:id/assign` | Assign document to project |

### Files

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/files/:filename` | Download/view uploaded file |

## API Examples

### Create a Project

```bash
curl -X POST http://localhost:3000/projects \
  -H "Content-Type: application/json" \
  -d '{"name": "Obra Porto Seg Social"}'
```

### Upload a File

```bash
curl -X POST http://localhost:3000/upload \
  -F "file=@sketch.jpg"
```

### List Inbox Documents

```bash
curl http://localhost:3000/documents/inbox
```

### Assign Document to Project

```bash
curl -X PATCH http://localhost:3000/documents/{doc_id}/assign \
  -H "Content-Type: application/json" \
  -d '{"project_id": "project-uuid-here"}'
```

### List Active Projects (for Mobile Dropdown)

```bash
curl http://localhost:3000/projects?status=active
```

## Project Structure

```
charta/
├── Cargo.toml              # Dependencies
├── src/
│   ├── main.rs             # Entry point and router setup
│   ├── db.rs               # Database initialization and migrations
│   ├── error.rs            # Unified error handling
│   ├── models.rs           # Domain entities and DTOs
│   ├── handlers/           # HTTP request handlers
│   │   ├── mod.rs
│   │   ├── project_handlers.rs
│   │   └── document_handlers.rs
│   └── services/           # Business logic layer
│       ├── mod.rs
│       ├── project_service.rs
│       └── document_service.rs
├── uploads/                # Uploaded files storage
└── charta.db               # SQLite database (created on first run)
```

## Configuration

Environment variables:

- `RUST_LOG` - Log level (default: `charta=debug,tower_http=debug`)

## Backup

Important files to backup:

1. `charta.db` - SQLite database
2. `uploads/` - All uploaded documents

## License

MIT
