# Raugupatis Log - AI Coding Agent Instructions

## Project Overview
Raugupatis Log is a fermentation tracking application built with Rust, featuring server-side rendering with Axum + Askama templates and SQLite for data persistence. Named after the Baltic deity of fermentation, it helps users monitor and optimize their fermentation processes through precise data logging and visualization.

## Architecture & Technology Stack

### Core Stack
- **Backend**: Rust with Axum async web framework
- **Templates**: Askama (compile-time Jinja2-like templates)  
- **Database**: SQLite with rusqlite crate
- **Deployment**: OCI containers with devcontainer development

### Key Design Principles
- **Simple monolithic architecture** over microservices complexity
- **Server-side rendering first** - SEO optimized, accessible HTML with progressive enhancement
- **Compile-time safety** - leverage Rust's type system and Askama's template validation
- **Mobile-first responsive design** with device-specific optimizations

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

### Authentication Flow Implementation
```rust
// Use tower-sessions with SQLite backend for session persistence
// Password hashing with argon2 crate (OWASP recommended)
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use tower_sessions::{Session, SessionManagerLayer};

// Session structure
#[derive(Serialize, Deserialize)]
struct UserSession {
    user_id: i64,
    email: String,
    role: UserRole,
    remember_me: bool,
}
```

### Middleware Stack (Order Matters)
1. **TraceLayer** - Request logging and tracing
2. **SessionManagerLayer** - Session handling before auth
3. **AuthLayer** - Custom auth middleware checking sessions
4. **CorsLayer** - CORS handling for API endpoints
5. **CompressionLayer** - Response compression

### Authorization Patterns
- Use `RequireAuthorizationLayer` for protected routes
- Implement role-based access with custom extractors
- Session expiry: 24h normal, 30 days for "remember me"

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

### Test Organization
```rust
// Use cargo-nextest for faster test execution
// tests/
//   ├── integration/     # Full HTTP integration tests
//   ├── unit/           # Individual function tests  
//   └── common/         # Shared test utilities
```

### Testing Stack
- **Unit Tests**: Built-in `#[cfg(test)]` with `tokio::test` for async
- **Integration Tests**: `axum-test` crate for HTTP endpoint testing
- **Property Testing**: `proptest` for fermentation state validation
- **Database Tests**: `sqlx-test` or in-memory SQLite for isolation
- **Template Tests**: Compile-time validation via Askama + unit tests for data binding

### Key Test Patterns
```rust
// HTTP endpoint testing with axum-test
#[tokio::test]
async fn test_create_fermentation() {
    let server = TestServer::new(app()).unwrap();
    let response = server
        .post("/fermentations")
        .json(&new_fermentation)
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
}

// Property-based testing for fermentation logic
proptest! {
    #[test]
    fn fermentation_never_negative_days(start in any::<DateTime<Utc>>()) {
        let fermentation = Fermentation::new(start);
        prop_assert!(fermentation.days_elapsed() >= 0);
    }
}
```

### Test Database Setup
- Use `rusqlite::Connection::open_in_memory()` for fast, isolated tests
- Create `TestApp` struct wrapping application with test database
- Implement `Default` trait for easy test data creation

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
6. Add integration tests covering the full HTTP request cycle
7. Ensure mobile-responsive design and accessibility in templates
8. Update API documentation and add logging/tracing as needed