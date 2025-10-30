pub mod auth;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod templates;

// Re-export commonly used items for convenience
pub use handlers::{login_user, logout_user, register_user, update_profile};
pub use models::{
    CreateUserRequest, ExperienceLevel, LoginRequest, LoginResponse, UpdateProfileRequest, User, UserResponse,
    UserRole, UserSession,
};
pub use repository::UserRepository;
pub use templates::{login_handler, profile_handler, register_handler};
