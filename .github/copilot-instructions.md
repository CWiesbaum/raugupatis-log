# Raugupatis Log - AI Coding Agent Instructions

## Project Overview
Raugupatis Log is a fermentation tracking application built with Rust, featuring server-side rendering with Axum + Askama templates and SQLite for data persistence. Named after the Baltic deity of fermentation, it helps users monitor and optimize their fermentation processes through precise data logging and visualization.

### Current Implementation Status
- **Phase 1 (Complete)**: Basic authentication, user registration/login, database schema, testing infrastructure
- **Phase 2 (Complete)**: Session management with tower-sessions, protected routes with authentication, fermentation CRUD operations (create, list, get profiles), user profile management
- **Phase 3 (In Progress)**: Temperature logging with charts, photo uploads, dashboard analytics, view/update/delete fermentation details

## Architecture & Technology Stack

### Core Stack
- **Backend**: Rust with Axum async web framework
- **Templates**: Askama (compile-time Jinja2-like templates)  
- **Database**: SQLite with rusqlite crate
- **Deployment**: OCI containers with devcontainer development

### Key Design Principles
- **Domain-driven structure** - Code organized by feature domain (users, fermentation, etc.) for high cohesion and low coupling
- **Simple monolithic architecture** over microservices complexity
- **Server-side rendering first** - SEO optimized, accessible HTML with progressive enhancement
- **Compile-time safety** - leverage Rust's type system and Askama's template validation
- **Mobile-first responsive design** with device-specific optimizations

### Project Structure
The project follows a domain-based architecture:
```
src/
├── users/           # User management domain
│   ├── auth.rs      # Password hashing/verification
│   ├── handlers.rs  # User API handlers
│   ├── models.rs    # User models and DTOs
│   ├── repository.rs # User database operations
│   └── templates.rs # User template handlers
├── config.rs        # Configuration management
├── database.rs      # Database connection
├── templates.rs     # General template handlers
└── lib.rs          # Main library entry point

templates/
├── users/          # User domain templates
│   ├── login.html
│   └── register.html
├── home.html
└── dashboard.html
```

When adding new features:
- Create a new domain module (e.g., `src/fermentation/`) for cohesive feature sets
- Include all related code (models, handlers, repository, templates) in the domain module
- Keep domain modules isolated to reduce coupling
- Follow the pattern established by the `users` module

## Development Environment Setup
- Use the devcontainer configuration for consistent development environment
- All dependencies and tools are pre-configured in the container
- Supports hot-reload for templates and Rust code during development

## Core Domain Models

### Fermentation Profiles (Predefined Templates)
```rust
// Key profiles to implement with specific parameters:
// - Pickles: 3-7 days, 65-75°F, salt brine
// - Kombucha: 7-14 days, 68-78°F, SCOBY-based  
// - Kimchi: 3-5 days room temp + refrigeration, 65-75°F
// - Sauerkraut: 2-4 weeks, 65-72°F, dry salt
// - Sourdough: 5-7 days establishment, 70-80°F maintenance
// - Kefir: 12-24 hours, 68-76°F, milk/water grains
```

## Authentication & Session Management

### Current Implementation
- **User registration**: Argon2 password hashing, email validation, experience level tracking
- **User login**: Server-side session management using tower-sessions with SQLite persistence
- **User logout**: Server-side session destruction with proper cleanup
- **Protected routes**: Session validation with redirect to login when not authenticated
- **Database repository pattern**: UserRepository for clean data access layer
- **API endpoints**: `/api/users/register`, `/api/users/login`, `/api/users/logout`

### Authentication Flow Implementation
```rust
// Using tower-sessions with SQLite backend for session persistence
// Password hashing with argon2 crate (OWASP recommended)
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use tower_sessions::{Session, SessionManagerLayer};

// Session structure to implement
#[derive(Serialize, Deserialize)]
struct UserSession {
    user_id: i64,
    email: String,
    role: UserRole,
}
```

### Middleware Stack (Order Matters)
1. **TraceLayer** - Request logging and tracing ✅
2. **SessionManagerLayer** - Session handling before auth ✅
3. **CompressionLayer** - Response compression ✅
4. **CorsLayer** - CORS handling for API endpoints ✅

### Authorization Patterns
- Protected routes check for valid session using `Session` extractor ✅
- Dashboard redirects to login when session is missing ✅
- Session expiry: 24h inactivity timeout ✅
- Role-based access with custom extractors (admin vs. user) - To implement
- Extended session duration for "remember me" - To implement

### Fermentation Tracking
- **Flexible timing**: Support both date ranges (min-max days) and exact durations
- **Temperature logging**: Manual data points with interactive chart visualization
- **Photo uploads**: Document fermentation stages visually
- **Completion tracking**: Success ratings, taste profiles, lessons learned
- **Historical analysis**: Search, filter, and compare batches for optimization

## Database Schema Design

### Migration Strategy
```rust
// Use rusqlite_migration crate for version-controlled schema evolution
// migrations/001_initial_schema.sql, 002_add_photos.sql, etc.
```

### Core Tables
```sql
-- Users with role-based permissions
users (id, email, password_hash, role, experience_level, created_at)

-- Fermentation profiles (templates)
fermentation_profiles (id, name, type, min_days, max_days, temp_min, temp_max, description)

-- Active/completed fermentations
fermentations (id, user_id, profile_id, name, start_date, target_end_date, actual_end_date, 
               status, success_rating, notes, ingredients_json)

-- Time-series temperature data (optimized for charts)
temperature_logs (id, fermentation_id, recorded_at, temperature, notes)

-- Photo documentation
fermentation_photos (id, fermentation_id, file_path, caption, taken_at, stage)
```

### Performance Indexes
- `CREATE INDEX idx_temp_logs_fermentation_time ON temperature_logs(fermentation_id, recorded_at)`
- `CREATE INDEX idx_fermentations_user_status ON fermentations(user_id, status)`

## Template Organization
- Base layout templates for consistent navigation and structure
- Specialized templates for:
  - Fermentation logging and monitoring dashboards
  - User management (registration, login, profile)
  - Data visualization (temperature charts, progress indicators)
  - Administrative interfaces

## Performance & Security Requirements
- **Async-first**: Use Axum's tower middleware for non-blocking operations
- **Type safety**: Leverage Rust's ownership system for memory safety
- **Template safety**: Compile-time validation prevents runtime template errors
- **Authentication**: Implement secure session management with Tower middleware
- **Responsive design**: Optimize for desktop (full features), tablet (touch), mobile (essential functions)

## Testing Strategy (Best-of-Breed Rust/Axum)

### Test Organization (Domain-Based Structure)

Integration tests are organized by domain to match the application architecture and reduce merge conflicts:

```
tests/
├── common/
│   └── mod.rs           # Shared test utilities (create_test_app, create_test_app_state)
├── general.rs           # Infrastructure tests (health checks, home page)
├── users.rs             # User domain integration tests
├── fermentation.rs      # Fermentation domain integration tests
└── [domain].rs          # Additional domain test files as needed
```

**Key principles:**
- Each domain has its own test file to minimize merge conflicts
- Common test utilities are shared in `tests/common/mod.rs`
- Tests use temporary SQLite databases for isolation
- Follow the same domain structure as the application code

### Testing Stack
- **Unit Tests**: Built-in `#[cfg(test)]` with `tokio::test` for async
- **Integration Tests**: Domain-organized files using Axum's test helpers
- **Property Testing**: `proptest` for fermentation state validation
- **Database Tests**: Temporary SQLite files for isolation
  - Uses temp files (not in-memory) to match production environment behavior
  - Each test gets a unique temp DB file with timestamp for complete isolation
  - Slight performance trade-off for better production parity
- **Template Tests**: Compile-time validation via Askama + unit tests for data binding

### Integration Test Patterns

#### Common Utilities (tests/common/mod.rs)
```rust
use axum::Router;
use raugupatis_log::{config::AppConfig, database::Database, AppState};
use std::sync::Arc;

/// Creates a test app with a fresh database
pub async fn create_test_app() -> Router {
    let app_state = create_test_app_state().await;
    raugupatis_log::create_router(app_state).await
}

/// Creates app state with a unique temporary database
pub async fn create_test_app_state() -> AppState {
    // Creates unique temp DB file with timestamp to ensure isolation
    let temp_dir = std::env::temp_dir();
    let test_db_path = temp_dir
        .join(format!("test_raugupatis_{}.db", timestamp))
        .to_string_lossy()
        .to_string();
    
    let config = Arc::new(AppConfig {
        database_url: test_db_path,
        environment: "test".to_string(),
        session_secret: "test-secret".to_string(),
        // ... other config fields
    });
    
    let db = Arc::new(Database::new(&config.database_url).await.unwrap());
    db.migrate().await.unwrap();
    
    AppState { db, config }
}
```

#### Domain Test File Pattern
```rust
mod common;  // Import common test utilities

use axum::{body::Body, http::{Request, StatusCode}};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_domain_feature() {
    let app = common::create_test_app().await;
    
    let request_body = json!({"field": "value"});
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/endpoint")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

### When Adding Tests for New Features

1. **Identify the domain** - Determine which domain your feature belongs to
2. **Add tests to the appropriate domain file** - Keep related tests together
   - User registration/login/profile → `tests/users.rs`
   - Fermentation CRUD → `tests/fermentation.rs`
   - Temperature logs → create `tests/temperature_logs.rs`
3. **Use common utilities** - Always use `common::create_test_app()` for consistency
4. **Test edge cases** - Include success, failure, validation, and authorization tests
5. **Follow naming conventions** - `test_[feature]_[scenario]` (e.g., `test_login_invalid_credentials`)

### For New Domains

When creating a new domain (e.g., `photos`, `analytics`):
1. Create application code in `src/[domain]/`
2. Create test file `tests/[domain].rs`
3. Add `mod common;` at the top
4. Follow the same patterns as existing domain test files

This ensures:
- **Reduced merge conflicts** - Multiple developers can work on different domains simultaneously
- **Better test organization** - Tests are grouped with their related functionality
- **Easier maintenance** - Find all tests for a feature in one place
- **Consistent patterns** - Standard structure makes it easy to add new tests

## Deployment Process

### Development Environment
```dockerfile
# Use devcontainer with pre-configured Rust toolchain
# .devcontainer/devcontainer.json includes:
# - Rust analyzer, cargo extensions
# - SQLite tools and browser
# - Node.js for frontend asset processing
```

### Container Build Process
```dockerfile
# Multi-stage Dockerfile for optimal image size
FROM rust:1.75-alpine AS builder
# Install build dependencies, compile release binary
FROM alpine:latest AS runtime  
# Copy binary, set up non-root user, configure health checks
```

### Build & Deployment Commands
```bash
# Local development
cargo watch -x run                    # Hot reload during development
cargo test --workspace              # Run all tests
cargo clippy --all-targets         # Lint code

# Production build
docker build -t raugupatis-log:latest .
docker run -p 3000:3000 -v ./data:/app/data raugupatis-log:latest

# Container deployment (Kubernetes/Docker Compose)
# Mount persistent volume for SQLite database
# Configure health checks on /health endpoint
# Set resource limits and restart policies
```

### Configuration Management
- Use `config` crate for environment-based settings
- Database path, session secret, upload directories via env vars
- Separate configs for development, staging, production
- Validate configuration at startup with helpful error messages

## When Implementing New Features
1. Start with the database schema and migrations using `rusqlite_migration`
2. Define Rust structs with proper ownership, lifetimes, and serde derives
3. Write unit tests for business logic before implementation
4. Create Askama templates with type-safe data binding and error handling
5. Implement Axum handlers with async/await patterns and proper extractors
6. **Add integration tests to the appropriate domain test file** (or create a new `tests/[domain].rs` file for new domains)
   - Use `common::create_test_app()` for consistency
   - Test success cases, edge cases, validation, and authorization
   - Follow naming convention: `test_[feature]_[scenario]`
7. Ensure mobile-responsive design and accessibility in templates
8. Update API documentation and add logging/tracing as needed
9. Ensure that README.md, PROJECT_SETUP.md and relevant docs are updated with new feature details
10. Ensure that copilot-instructions.md reflects any architectural or design changes