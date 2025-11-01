pub mod handlers;
pub mod models;
pub mod repository;
pub mod templates;

// Re-export commonly used items for convenience
pub use handlers::{create_fermentation, get_profiles, list_fermentations, update_fermentation};
pub use models::{
    CreateFermentationRequest, Fermentation, FermentationProfile, FermentationResponse,
    FermentationStatus, UpdateFermentationRequest,
};
pub use repository::FermentationRepository;
pub use templates::{edit_fermentation_handler, fermentation_detail_handler, fermentation_list_handler, new_fermentation_handler};
