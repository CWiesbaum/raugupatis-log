pub mod handlers;
pub mod models;
pub mod repository;
pub mod templates;

// Re-export commonly used items
pub use handlers::{copy_profile, create_profile, list_all_profiles, set_profile_active_status};
pub use models::{AdminProfileResponse, CopyProfileRequest, CreateProfileRequest, DeactivateProfileRequest};
pub use repository::AdminProfileRepository;
pub use templates::admin_profiles_list_handler;
