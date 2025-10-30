pub mod handlers;
pub mod models;
pub mod repository;
pub mod templates;

// Re-export commonly used items
pub use handlers::{create_user, delete_user, list_users, lock_user, update_user};
pub use models::{
    AdminCreateUserRequest, AdminUpdateUserRequest, AdminUserResponse, LockUserRequest,
};
pub use repository::AdminUserRepository;
pub use templates::admin_users_list_handler;
