pub mod handlers;
pub mod models;
pub mod repository;
pub mod templates;

// Re-export commonly used items for convenience
pub use handlers::{
    create_fermentation, create_taste_profile, create_temperature_log, finish_fermentation,
    get_profiles, list_fermentations, list_taste_profiles, list_temperature_logs,
    update_fermentation,
};
pub use models::{
    CreateFermentationRequest, CreateTasteProfileRequest, CreateTemperatureLogRequest,
    Fermentation, FermentationProfile, FermentationResponse, FermentationStatus,
    FinishFermentationRequest, TasteProfile, TemperatureLog, UpdateFermentationRequest,
};
pub use repository::FermentationRepository;
pub use templates::{
    edit_fermentation_handler, fermentation_detail_handler, fermentation_list_handler,
    new_fermentation_handler,
};
