# Raugupatis Log - Project Setup & Status 🥒

## Current Development Phase

**Phase 1: Foundation & Authentication** ✅ **COMPLETE**
- User registration and login working
- Database schema implemented
- Testing infrastructure in place

**Phase 2: Session Management & Fermentation Tracking** ✅ **COMPLETE**
- Session management fully operational with tower-sessions and SQLite persistence
- Fermentation CRUD operations implemented (create, list, get profiles)
- Protected routes with authentication and session validation
- User profile management (view, update)
- Web interfaces for fermentation list and creation form

**Phase 3: Advanced Features** 🚧 **IN PROGRESS**
- Temperature logging and charts
- Photo upload functionality
- Dashboard analytics
- View individual fermentation details
- Update/edit fermentation records
- Delete fermentation functionality

## Project Structure

The project follows a **domain-based architecture** with high cohesion and low coupling:

```
raugupatis-log/
├── src/
│   ├── main.rs              # Main application entry point with Axum server
│   ├── lib.rs               # Library entry point and router setup
│   ├── config.rs            # Configuration management with TOML support
│   ├── database.rs          # SQLite database connection and migration handling
│   ├── templates.rs         # General template handlers (home, dashboard)
│   └── users/               # User management domain (all user-related code)
│       ├── mod.rs           # Module exports
│       ├── auth.rs          # Password hashing and verification
│       ├── handlers.rs      # User API handlers (register, login, logout)
│       ├── models.rs        # User data models and DTOs
│       ├── repository.rs    # User database operations
│       └── templates.rs     # User template handlers
├── templates/
│   ├── home.html            # Home page template
│   ├── dashboard.html       # Dashboard template
│   └── users/               # User-related templates
│       ├── login.html       # Login page template
│       └── register.html    # Registration page template
├── migrations/
│   └── 001_initial_schema.sql # Database schema with all core tables
├── config/
│   ├── default.toml         # Default configuration
│   ├── development.toml     # Development-specific settings
│   └── production.toml      # Production configuration template
├── scripts/
│   ├── setup-dev.sh         # Development environment setup script
│   └── deploy.sh            # Production deployment script
├── tests/
│   ├── common/              # Shared test utilities
│   │   └── mod.rs           # Common test helper functions
│   ├── general.rs           # Infrastructure tests (health, home, dashboard)
│   ├── users.rs             # User domain integration tests
│   └── fermentation.rs      # Fermentation domain integration tests
├── Cargo.toml               # Rust dependencies and project metadata
├── Dockerfile               # Multi-stage OCI container specification
├── docker-compose.yml       # Docker Compose for local development
├── Makefile                 # Development and deployment automation
├── .gitignore               # Git ignore rules for Rust projects
├── .dockerignore            # Docker build context exclusions
├── LICENSE                  # Project license
└── README.md                # Project documentation
```

### Architecture Benefits

- **Domain-driven structure**: Code organized by feature domain (users, fermentation, etc.) rather than by type
- **High cohesion**: Related code is grouped together in domain modules
- **Low coupling**: Domains are isolated with clear boundaries
- **Scalability**: New domains can be added following the same pattern
- **Reduced merge conflicts**: Changes to one domain don't affect others

## ✅ What's Working

### Web Server
- **Axum-based async web server** running on port 3000
- **Home page** at `http://localhost:3000` with beautiful gradient styling
- **Registration page** at `http://localhost:3000/register` with form validation
- **Login page** at `http://localhost:3000/login` with authentication
- **Dashboard page** at `http://localhost:3000/dashboard` (basic template, not yet protected)
- **Health check** endpoint at `http://localhost:3000/health`
- **Askama templates** with compile-time validation
- **Proper error handling** with comprehensive logging

### Authentication & API
- **User registration API** at `/api/users/register` with:
  - Email validation (format checking)
  - Password strength validation (minimum 8 characters)
  - Argon2 password hashing
  - Duplicate email detection
- **User login API** at `/api/users/login` with:
  - Email/password verification
  - Secure password comparison
  - User information response
  - Session creation with tower-sessions
- **User logout API** at `/api/users/logout` with:
  - Server-side session destruction
  - Complete session cleanup
- **User profile API** at `/api/users/profile` with:
  - Update first name, last name, experience level
  - Protected endpoint requiring authentication
  - Validation for experience level values
- **Fermentation API** at `/api/fermentations` and `/api/fermentation` with:
  - List all fermentations for authenticated user (GET /api/fermentations)
  - Create new fermentation (POST /api/fermentation)
  - Get fermentation profiles (GET /api/fermentation/profiles)
  - Full validation and authorization checks

### Database
- **SQLite database** with connection management
- **Migration system** with rusqlite_migration
- **Complete database schema** with all required tables:
  - Users with role-based permissions
  - Fermentation profiles (predefined templates - 7 profiles seeded)
  - Active/completed fermentations
  - Temperature logs (time-series optimized)
  - Photo documentation
- **Performance indexes** for optimal queries
- **Database repository pattern** for clean data access

### Configuration
- **Environment-based configuration** (development/production)
- **TOML configuration files** with proper defaults
- **Environment variable overrides** support
- **Configuration validation** at startup

### Testing
- **Unit tests** for authentication (password hashing, email validation)
- **Integration tests** organized by domain following the application architecture:
  - **tests/common/mod.rs** - Shared test utilities and helper functions
  - **tests/general.rs** - Infrastructure tests (health, home, dashboard endpoints) - 5 tests
  - **tests/users.rs** - User domain tests covering registration, login, logout, profile - 21 tests
  - **tests/fermentation.rs** - Fermentation domain tests covering CRUD operations - 8 tests
- **34 passing integration tests** covering all main functionality
- **Test database** using temporary SQLite files for isolation
- **Domain-based test organization** reduces merge conflicts during parallel development

### Containerization
- **Multi-stage Dockerfile** for optimal image size and security
- **Non-root user** execution for security
- **Health checks** configured
- **Docker Compose** setup for local development
- **OCI-compliant** container specification

### Development Tools
- **Makefile** with common development tasks
- **Setup scripts** for development environment
- **Deployment scripts** for production
- **Comprehensive .gitignore** and .dockerignore
- **Devcontainer configuration** for consistent development environment

## 🚀 How to Run

### Local Development
```bash
# Start the development server
cargo run

# Or with hot-reload (requires cargo-watch)
make dev

# Run tests
make test
```

### Docker
```bash
# Build and run with Docker Compose
docker-compose up --build

# Or build and run manually
docker build -t raugupatis-log:latest .
docker run -p 3000:3000 -v $(pwd)/data:/app/data raugupatis-log:latest
```

### Using Make Commands
```bash
make help          # Show all available commands
make setup         # Set up development environment
make dev           # Run with hot-reload  
make docker-build  # Build Docker image
make test          # Run all tests
make clean         # Clean build artifacts
```

## 📡 Endpoints

### Web Pages (GET)
- **/** - Beautiful home page with fermentation-themed styling
- **/register** - User registration form
- **/login** - User login form
- **/dashboard** - Dashboard page (protected, requires authentication)
- **/profile** - User profile page for viewing and editing account information (protected)
- **/fermentations** - List of all fermentations for the authenticated user (protected)
- **/fermentation/new** - Form to create a new fermentation batch (protected)
- **/health** - Health check endpoint (returns "OK")

### API Endpoints

#### User Management (POST)
- **/api/users/register** - Create new user account
  - Accepts: `{ "email": "user@example.com", "password": "password123", "experience_level": "beginner", "first_name": "John", "last_name": "Doe" }`
  - Returns: User object on success (201), error message on failure
- **/api/users/login** - Authenticate user
  - Accepts: `{ "email": "user@example.com", "password": "password123" }`
  - Returns: `{ "success": true/false, "user": {...}, "message": "..." }`
- **/api/users/logout** - End user session
  - Requires: Valid session
  - Returns: Success message
- **/api/users/profile** - Update user profile
  - Requires: Valid session (protected)
  - Accepts: `{ "first_name": "John", "last_name": "Doe", "experience_level": "intermediate" }`
  - Returns: Updated user object

#### Fermentation Management (GET/POST)
- **GET /api/fermentations** - List all fermentations for authenticated user
  - Requires: Valid session (protected)
  - Returns: Array of fermentation objects with profile information
- **POST /api/fermentation** - Create new fermentation batch
  - Requires: Valid session (protected)
  - Accepts: `{ "profile_id": 1, "name": "My Kimchi Batch", "start_date": "2024-01-15T10:00:00Z", "target_end_date": "2024-01-20T10:00:00Z", "notes": "Using napa cabbage", "ingredients": "cabbage, salt, garlic, ginger" }`
  - Returns: Created fermentation object (201)
- **GET /api/fermentation/profiles** - Get all fermentation profile templates
  - Returns: Array of predefined fermentation profiles (Pickles, Kombucha, Kimchi, etc.)

## 🏗️ Next Steps for Phase 3 Implementation

The core application is now fully functional with authentication, session management, and basic fermentation tracking. Next steps for implementing advanced features:

1. **View fermentation details** - Display individual fermentation page with complete history and status
2. **Update/edit fermentation** - Allow users to modify fermentation details, status, and notes
3. **Delete fermentation** - Remove fermentation records with confirmation dialog
4. **Temperature logging** - Manual data point entry and visualization with charts
5. **Photo uploads** - Document fermentation stages with file storage and gallery display
6. **Dashboard analytics** - Interactive charts and progress tracking for active fermentations with countdown timers
7. **Advanced filtering** - Search and filter fermentations by type, date range, success rating, or ingredients
8. **Password management** - Allow users to change their password securely

## 🧪 Testing Guidelines

When adding new features, follow these testing patterns:

### Integration Test Organization

Integration tests are organized by domain to match the application architecture and reduce merge conflicts:

1. **Common utilities** (`tests/common/mod.rs`):
   - `create_test_app()` - Creates a fresh app instance for testing
   - `create_test_app_state()` - Creates app state with a temporary test database
   - Add new shared utilities here as needed

2. **Domain-specific test files**:
   - `tests/users.rs` - All user-related tests (registration, login, profile, etc.)
   - `tests/fermentation.rs` - All fermentation-related tests (CRUD, profiles, etc.)
   - `tests/general.rs` - Infrastructure tests (health checks, home page, etc.)

### Adding Tests for New Features

When adding a new feature:

1. **Identify the domain** - Determine which domain your feature belongs to (users, fermentation, etc.)
2. **Add tests to the appropriate domain file** - Keep related tests together
3. **Use common utilities** - Reuse `create_test_app()` and `create_test_app_state()` from `tests/common/mod.rs`
4. **Follow naming conventions** - Test names should clearly describe what they test (e.g., `test_create_fermentation_unauthorized`)
5. **Test edge cases** - Include tests for success, failure, validation, and authorization scenarios

### Example Test Structure

```rust
mod common;

use axum::{body::Body, http::{Request, StatusCode}};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_your_feature() {
    let app = common::create_test_app().await;
    
    let request_body = json!({"key": "value"});
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/your/endpoint")
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

### For New Domains

If creating a new domain (e.g., `temperature_logs`, `photos`):

1. Create a new test file: `tests/your_domain.rs`
2. Add `mod common;` at the top
3. Follow the same pattern as existing domain test files
4. Add all tests for that domain to this single file

This organization ensures:
- **Reduced merge conflicts** - Developers working on different domains rarely touch the same files
- **Better maintainability** - Related tests are grouped together
- **Clearer test output** - Test results are organized by domain
- **Consistent patterns** - Easy to find and add tests for any feature

## 🔧 Technology Stack Implemented

- **Backend**: Rust with Axum async web framework ✅
- **Templates**: Askama compile-time template engine ✅
- **Database**: SQLite with rusqlite and migration support ✅
- **Configuration**: TOML-based with environment overrides ✅
- **Logging**: Structured logging with tracing ✅
- **Containers**: Multi-stage Docker with security best practices ✅
- **Development**: Hot-reload, testing framework, automation scripts ✅

The Baltic God of Fermentation would be proud! 🥒⚡