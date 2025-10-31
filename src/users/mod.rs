pub mod auth;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod templates;

// Re-export commonly used items for convenience
pub use handlers::{change_password, login_user, logout_user, register_user, update_profile};
pub use models::{
    ChangePasswordRequest, CreateUserRequest, ExperienceLevel, LoginRequest, LoginResponse,
    UpdateProfileRequest, User, UserResponse, UserRole, UserSession,
};
pub use repository::UserRepository;
pub use templates::{change_password_handler, login_handler, profile_handler, register_handler};
