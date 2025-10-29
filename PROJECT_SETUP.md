# Raugupatis Log - Project Setup & Status 🥒

## Current Development Phase

**Phase 1: Foundation & Authentication** ✅ **COMPLETE**
- User registration and login working
- Database schema implemented
- Testing infrastructure in place

**Phase 2: Session Management & Fermentation Tracking** 🚧 **IN PROGRESS**
- Session management implementation needed
- Fermentation CRUD operations to be built
- Protected routes to be added

**Phase 3: Advanced Features** 📋 **PLANNED**
- Temperature logging and charts
- Photo upload functionality
- Dashboard analytics

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
│   └── integration_tests.rs # Integration test framework
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
- **Integration tests** for all API endpoints and pages
- **13 passing tests** covering registration and login flows
- **Test database** using in-memory SQLite for isolation

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
- **/dashboard** - Dashboard page (template ready, not yet with full functionality)
- **/health** - Health check endpoint (returns "OK")

### API Endpoints (POST)
- **/api/users/register** - Create new user account
  - Accepts: `{ "email": "user@example.com", "password": "password123", "experience_level": "beginner" }`
  - Returns: User object on success (201), error message on failure
- **/api/users/login** - Authenticate user
  - Accepts: `{ "email": "user@example.com", "password": "password123" }`
  - Returns: `{ "success": true/false, "user": {...}, "message": "..." }`

## 🏗️ Ready for Implementation

The foundation is now complete with basic authentication working. Next steps for implementing core fermentation tracking features:

1. **Session management** - Add tower-sessions middleware for persistent login sessions
2. **Protected routes** - Secure dashboard and fermentation endpoints with authentication
3. **Fermentation CRUD operations** - Create, read, update, delete fermentations using the existing database schema
4. **Temperature logging** - Manual data point entry and visualization with charts
5. **Photo uploads** - Document fermentation stages with file storage
6. **Dashboard and analytics** - Interactive charts and progress tracking for active fermentations
7. **User profile** - View and edit user settings, change password

## 🔧 Technology Stack Implemented

- **Backend**: Rust with Axum async web framework ✅
- **Templates**: Askama compile-time template engine ✅
- **Database**: SQLite with rusqlite and migration support ✅
- **Configuration**: TOML-based with environment overrides ✅
- **Logging**: Structured logging with tracing ✅
- **Containers**: Multi-stage Docker with security best practices ✅
- **Development**: Hot-reload, testing framework, automation scripts ✅

The Baltic God of Fermentation would be proud! 🥒⚡