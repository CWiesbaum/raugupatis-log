pub mod handlers;
pub mod models;
pub mod repository;
pub mod templates;

// Re-export commonly used items for convenience
pub use handlers::{
    create_fermentation, create_temperature_log, get_profiles, list_fermentations,
    list_temperature_logs, update_fermentation,
};
pub use models::{
    CreateFermentationRequest, CreateTemperatureLogRequest, Fermentation, FermentationProfile,
    FermentationResponse, FermentationStatus, TemperatureLog, UpdateFermentationRequest,
};
pub use repository::FermentationRepository;
pub use templates::{
    edit_fermentation_handler, fermentation_detail_handler, fermentation_list_handler,
    new_fermentation_handler,
};
