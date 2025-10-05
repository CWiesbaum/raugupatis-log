# raugupatis-log
Raugupatis is the ultimate fermentation tracker. Log and monitor your food ferments—temperature, ingredients, time, and more. Master your batches with the digital precision of the Baltic God of Fermentation.

Named after the Baltic deity of fermentation and pickles, Raugupatis Log brings ancient wisdom into the modern kitchen, helping you achieve consistent, high-quality fermentation results through precise monitoring and data-driven insights.

# Functionality
The following functionality is supported by **Raugupatis Log**

## User Management
- **User registration**: New users can create accounts with secure password hashing and email validation. Registration includes basic profile setup with fermentation experience level and preferred fermentation types.
- **User login**: Secure authentication system with session management and optional "remember me" functionality for convenient access across devices.
- **User logout**: Clean session termination with proper security cleanup and optional logout from all devices.
- **User Administration (via Admin role)**: Administrative interface for managing users, viewing system statistics, moderating shared fermentation logs, and maintaining fermentation profile templates.

## Fermentation Logging
- **Create new fermentation**: Start tracking a new batch with metadata including ingredients and initial conditions. Support for photo uploads to document the starting state.
- **Finish running fermentation**: Mark fermentations as complete with final notes, success ratings, taste profiles, and lessons learned. Generate completion reports with full batch history.
- **Browse historical fermentations**: Search and filter past fermentations by type, date range, success rating, or ingredients. Compare different batches to identify patterns and optimize techniques.
- **Add optional temperature data points for fermentation**: Log temperature readings manually.
- **Display graph for temperature data points**: Interactive charts showing temperature curves over time.
- **Select from predefined List of common fermentation profiles**: Comprehensive database of fermentation templates including:
  - **Pickles**: 3-7 days, 65-75°F, salt brine fermentation
  - **Kombucha**: 7-14 days, 68-78°F, SCOBY-based fermentation
  - **Kimchi**: 3-5 days room temp + refrigeration, 65-75°F initial fermentation
  - **Sauerkraut**: 2-4 weeks, 65-72°F, dry salt fermentation
  - **Sourdough starter**: 5-7 days establishment, 70-80°F maintenance
  - **Kefir**: 12-24 hours, 68-76°F, milk or water kefir grains
- **Countdown timer functionality**: When starting a new fermentation, users specify either a date range (minimum-maximum days) or exact duration. The system provides:
  - Daily countdown display showing remaining time
  - Progress indicators with visual milestones
  - Flexible extension options if fermentation needs more time
  - Early completion tracking for faster-than-expected batches

# Technology
Raugupati Log follows the following architectural principles:

## Development Environment
- **Development in devcontainer**: Fully containerized development environment using Docker with VS Code integration. Includes all necessary tools, dependencies, and extensions pre-configured for immediate productivity. Ensures consistent development experience across different machines and operating systems.

## Architecture Philosophy
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
  - **Authentication**: Tower middleware for secure session management
  - **Testing**: Built-in unit testing, integration testing, and property-based testing support

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