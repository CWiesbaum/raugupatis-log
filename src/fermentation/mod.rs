pub mod handlers;
pub mod models;
pub mod repository;
pub mod templates;

// Re-export commonly used items for convenience
pub use handlers::{create_fermentation, get_profiles};
pub use models::{
    CreateFermentationRequest, Fermentation, FermentationProfile, FermentationResponse,
    FermentationStatus,
};
pub use repository::FermentationRepository;
pub use templates::new_fermentation_handler;
