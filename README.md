# raugupatis-log
Raugupatis is the ultimate fermentation tracker. Log and monitor your food ferments—temperature, ingredients, time, and more. Master your batches with the digital precision of the Baltic God of Fermentation.

Named after the Baltic deity of fermentation and pickles, Raugupatis Log brings ancient wisdom into the modern kitchen, helping you achieve consistent, high-quality fermentation results through precise monitoring and data-driven insights.

# Functionality

## Current Status
Raugupatis Log is under active development. The core foundation and authentication system are operational, with fermentation tracking features in progress.

## ✅ Implemented Features

### User Management
- **User registration**: New users can create accounts with secure Argon2 password hashing and email validation. Registration includes basic profile setup with fermentation experience level.
- **User login**: Secure authentication system with server-side session management using tower-sessions with SQLite persistence. Sessions expire after 24 hours of inactivity with HttpOnly cookies.
- **User logout**: Clean session termination with server-side session destruction. Client-side session storage is cleared to ensure complete logout.
- **Protected routes**: Dashboard and authenticated pages validate server-side sessions and redirect to login when not authenticated.

### Database & Infrastructure
- **Complete database schema**: All tables designed and created (users, fermentation_profiles, fermentations, temperature_logs, fermentation_photos)
- **Predefined fermentation profiles**: Database seeded with 7 common fermentation templates:
  - **Pickles**: 3-7 days, 65-75°F, salt brine fermentation
  - **Kombucha**: 7-14 days, 68-78°F, SCOBY-based fermentation
  - **Kimchi**: 3-5 days room temp + refrigeration, 65-75°F initial fermentation
  - **Sauerkraut**: 2-4 weeks, 65-72°F, dry salt fermentation
  - **Sourdough starter**: 5-7 days establishment, 70-80°F maintenance
  - **Kefir (Milk)**: 12-24 hours, 68-76°F, milk kefir grains
  - **Water Kefir**: 1-3 days, 68-76°F, water kefir grains

### User Profile Management
- **View profile**: Access personal profile page displaying account information and fermentation experience level
- **Update profile**: Edit first name, last name, and experience level through dedicated API endpoint
- **Profile validation**: Server-side validation ensures data integrity when updating profile information

### Fermentation Logging
- **Create new fermentation**: Start tracking a new batch with metadata including profile selection, name, start date, target end date, notes, and ingredients. Full validation ensures data integrity.
- **List fermentations**: View all fermentations for the authenticated user through both web interface (/fermentations) and API endpoint (/api/fermentations)
- **Browse fermentation profiles**: Access 7 predefined fermentation templates (Pickles, Kombucha, Kimchi, Sauerkraut, Sourdough, Kefir, Water Kefir) with optimal temperature ranges and timing guidance
- **New fermentation form**: User-friendly web interface at /fermentation/new for creating fermentations with profile selection
- **Protected fermentation routes**: All fermentation pages and API endpoints require authentication, redirecting to login when session is missing

## 🚧 Planned Features (Phase 3)

### User Management
- **"Remember me" functionality**: Extended session duration (30 days) for convenient access
- **User Administration**: Administrative interface for managing users, viewing system statistics, and maintaining fermentation profile templates

### Fermentation Tracking - Advanced Features
- **View fermentation details**: Display individual fermentation with complete history, notes, and status tracking
- **Update fermentation**: Edit fermentation details, notes, and status (active, paused, completed, failed)
- **Add temperature data points**: Manual temperature logging for tracking fermentation progress
- **Display temperature graphs**: Interactive charts showing temperature curves over time
- **Photo uploads**: Document fermentation stages visually with file storage and management
- **Finish fermentation**: Mark batches as complete with success ratings, taste profiles, and lessons learned
- **Search and filter**: Advanced filtering by type, date range, success rating, or ingredients
- **Countdown timer**: Daily countdown display with progress indicators and flexible completion tracking
- **Delete fermentation**: Remove fermentation records with confirmation

# Technology
Raugupati Log follows the following architectural principles:

## Development Environment
- **Development in devcontainer**: Fully containerized development environment using Docker with VS Code integration. Includes all necessary tools, dependencies, and extensions pre-configured for immediate productivity. Ensures consistent development experience across different machines and operating systems.

## Architecture Philosophy
- **Domain-driven structure**: Code organized by feature domain (users, fermentation, etc.) rather than by technical layers. This approach follows the coupling and cohesion principles, resulting in:
  - **High cohesion**: Related code (models, handlers, repositories, templates) grouped together in domain modules
  - **Low coupling**: Domains are isolated with clear boundaries, reducing dependencies between features
  - **Better maintainability**: Easier to find and modify related code
  - **Reduced merge conflicts**: Changes to one domain don't affect others
  - **Scalability**: New domains can be added following the same pattern

- **Simple architecture approach**: Monolithic application design prioritizing maintainability and ease of deployment over complex microservices. Clear separation of concerns with well-defined layers (presentation, business logic, data access) while avoiding over-engineering for the application scale.

## Deployment & Runtime
- **OCI Container compatible App**: Application packaged as standard container images following Open Container Initiative specifications. Supports deployment on any container orchestration platform (Docker, Kubernetes, Podman) with health checks, graceful shutdown, and resource constraints properly configured.

## Development Language
- **Rust Programming Language**: Modern systems programming language providing memory safety, thread safety, and zero-cost abstractions without garbage collection. Rust's ownership system prevents common bugs like null pointer dereferences, buffer overflows, and data races at compile time, ensuring robust fermentation tracking without runtime crashes.

- **Performance Characteristics**: Compiled to native machine code with optimizations comparable to C/C++, delivering:
  - Fast temperature data processing and chart generation for real-time fermentation monitoring
  - Minimal memory footprint ideal for deployment on resource-constrained servers
  - Efficient concurrent handling of multiple users logging fermentation data simultaneously
  - Near-zero runtime overhead for the web server and database operations

- **Type Safety & Reliability**: Rust's strict type system and compiler checks provide:
  - Compile-time prevention of common web application vulnerabilities
  - Guaranteed memory safety without performance penalties
  - Exhaustive pattern matching ensuring all fermentation states are properly handled
  - Zero-cost abstractions allowing high-level code with low-level performance

- **Ecosystem & Tooling**: Rich crate ecosystem specifically beneficial for Raugupatis Log:
  - **Web frameworks**: Axum for async HTTP handling with excellent performance
  - **Database integration**: rusqlite for type-safe SQLite operations with zero-copy optimizations
  - **Template engines**: Askama for compile-time HTML template validation
  - **Serialization**: serde for efficient JSON/form data processing
  - **Authentication**: tower-sessions with tower-sessions-rusqlite-store for secure server-side session management with SQLite persistence
  - **Security**: argon2 for OWASP-recommended password hashing
  - **Testing**: Built-in unit testing, integration testing with axum-test, and property-based testing support

- **Developer Experience**: Modern tooling enhancing productivity:
  - Cargo package manager with dependency resolution and build automation
  - rustfmt for consistent code formatting across the project
  - clippy linter catching common mistakes and suggesting idiomatic improvements
  - Comprehensive compiler error messages with helpful suggestions
  - Excellent IDE support with rust-analyzer providing autocompletion and refactoring

## Data Storage
- **SQLite embedded database**: Lightweight, file-based SQL database providing ACID compliance without external database server requirements. Supports:
  - Zero-configuration setup and deployment
  - File-based persistence with automatic backup capabilities
  - Full SQL compliance for complex queries and reporting
  - Excellent Rust ecosystem support via `rusqlite`
  - Cross-platform compatibility and easy migration capabilities

## User Interface
- **Server Side rendered UI with Axum + Askama**: Modern async web framework (Axum) combined with compile-time template engine (Askama) for optimal performance and type safety. Templates are compiled at build time, eliminating runtime template parsing overhead while providing Rust's type safety guarantees for template data binding.

- **Template Architecture**: Jinja2-inspired template syntax with template inheritance and component reusability:
  - Base layout templates for consistent site structure and navigation
  - Specialized templates for fermentation logging, user management, and data visualization
  - Compile-time validation prevents template rendering errors in production
  - Hot-reload capability in development for rapid iteration

- **Async-First Performance**: Axum's tower-based architecture provides:
  - Non-blocking request handling for concurrent user sessions
  - Efficient resource utilization for temperature data logging and chart generation
  - Fast response times even during database operations
  - Built-in support for middleware (authentication, logging, compression)

- **Responsive design**: Mobile-first approach ensuring optimal user experience across all device sizes:
  - **Desktop**: Full-featured interface with advanced charting and multi-column layouts
  - **Tablet**: Adaptive layouts optimized for touch interaction and medium screen sizes  
  - **Mobile**: Streamlined interface focusing on essential logging functions with thumb-friendly navigation
  - Progressive enhancement with JavaScript for improved interactivity while maintaining full functionality without it

- **SEO and Accessibility Optimized**: Server-side rendering ensures:
  - Complete HTML content delivered on first request for search engine indexing
  - Fast initial page loads without JavaScript dependency
  - Screen reader compatibility and keyboard navigation support
  - Semantic HTML structure with proper ARIA labels for fermentation data

# Getting Started

## Prerequisites
- Rust 1.75 or later
- Docker (optional, for containerized deployment)
- Git

## Quick Start

### Local Development
```bash
# Clone the repository
git clone https://github.com/CWiesbaum/raugupatis-log.git
cd raugupatis-log

# Build and run
cargo build
cargo run

# Or use the Makefile
make run
```

The application will start on `http://localhost:3000`

### Available Commands
```bash
make help          # Show all available commands
make build         # Build the application
make test          # Run all tests
make lint          # Run clippy linter
make format        # Format code with rustfmt
make dev           # Run with hot-reload (requires cargo-watch)
```

### Docker Deployment
```bash
# Build and run with Docker Compose
docker-compose up --build

# Or build and run manually
docker build -t raugupatis-log:latest .
docker run -p 3000:3000 -v $(pwd)/data:/app/data raugupatis-log:latest
```

## Testing the Application

### Try the Authentication Flow
1. Visit `http://localhost:3000`
2. Click "Register" and create an account
3. Log in with your credentials
4. View the dashboard (basic template currently)

### API Testing
```bash
# Register a new user
curl -X POST http://localhost:3000/api/users/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"securepass123","experience_level":"beginner"}'

# Login
curl -X POST http://localhost:3000/api/users/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"securepass123"}'

# Health check
curl http://localhost:3000/health
```

## Development Roadmap

See [PROJECT_SETUP.md](PROJECT_SETUP.md) for detailed information about the project structure and development workflow.

## Contributing

This project is in active development. Contributions are welcome! Please ensure:
- All tests pass: `make test`
- Code is formatted: `make format`
- Linter passes: `make lint`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.