use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FermentationStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

impl FermentationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            FermentationStatus::Active => "active",
            FermentationStatus::Paused => "paused",
            FermentationStatus::Completed => "completed",
            FermentationStatus::Failed => "failed",
        }
    }
}

impl From<String> for FermentationStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "paused" => FermentationStatus::Paused,
            "completed" => FermentationStatus::Completed,
            "failed" => FermentationStatus::Failed,
            _ => FermentationStatus::Active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FermentationProfile {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub min_days: i32,
    pub max_days: i32,
    pub temp_min: f64,
    pub temp_max: f64,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fermentation {
    pub id: i64,
    pub user_id: i64,
    pub profile_id: i64,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub target_end_date: Option<DateTime<Utc>>,
    pub actual_end_date: Option<DateTime<Utc>>,
    pub status: FermentationStatus,
    pub success_rating: Option<i32>,
    pub notes: Option<String>,
    pub ingredients_json: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined from profile
    pub profile_name: Option<String>,
    pub profile_type: Option<String>,
}

impl Fermentation {
    /// Returns true if the fermentation should display a countdown timer
    /// Conditions: has target_end_date, status is Active or Paused, and timer hasn't expired
    pub fn should_show_countdown(&self) -> bool {
        if self.target_end_date.is_none() {
            return false;
        }

        // Don't show countdown if already completed or failed
        if matches!(
            self.status,
            FermentationStatus::Completed | FermentationStatus::Failed
        ) {
            return false;
        }

        // Don't show countdown if timer has expired
        if let Some(target_end) = self.target_end_date {
            let now = Utc::now();
            if now >= target_end {
                return false;
            }
        }

        true
    }

    /// Returns true if the fermentation schedule has finished (timer expired but not completed)
    pub fn is_schedule_finished(&self) -> bool {
        // Only show "Finished fermentation schedule" if:
        // - Has a target_end_date
        // - Status is still Active or Paused (not completed/failed)
        // - Current time is past target_end_date
        if let Some(target_end) = self.target_end_date {
            let now = Utc::now();
            let is_active_or_paused = matches!(
                self.status,
                FermentationStatus::Active | FermentationStatus::Paused
            );
            return now >= target_end && is_active_or_paused;
        }
        false
    }

    /// Returns the countdown display string
    /// Format: "X days" for more than 1 day remaining
    /// Format: "Xh Ym" for final day (less than 24 hours)
    pub fn countdown_display(&self) -> Option<String> {
        let target_end = self.target_end_date?;
        let now = Utc::now();

        if now >= target_end {
            return None;
        }

        let duration = target_end.signed_duration_since(now);
        let total_seconds = duration.num_seconds();

        if total_seconds <= 0 {
            return None;
        }

        // Calculate time components
        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;

        if days > 0 {
            // More than 1 day remaining
            if days == 1 {
                Some("1 day".to_string())
            } else {
                Some(format!("{} days", days))
            }
        } else {
            // Final day - show hours and minutes
            Some(format!("{}h {}m", hours, minutes))
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FermentationWithProfile {
    pub fermentation: Fermentation,
    pub profile: FermentationProfile,
}

#[derive(Debug, Deserialize)]
pub struct CreateFermentationRequest {
    pub profile_id: i64,
    pub name: String,
    pub start_date: String,              // ISO 8601 format
    pub target_end_date: Option<String>, // ISO 8601 format
    pub notes: Option<String>,
    pub ingredients: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FermentationResponse {
    pub id: i64,
    pub profile_id: i64,
    pub profile_name: String,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub target_end_date: Option<DateTime<Utc>>,
    pub status: FermentationStatus,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl FermentationResponse {
    pub fn from_fermentation_and_profile(
        fermentation: Fermentation,
        profile: FermentationProfile,
    ) -> Self {
        Self {
            id: fermentation.id,
            profile_id: fermentation.profile_id,
            profile_name: profile.name,
            name: fermentation.name,
            start_date: fermentation.start_date,
            target_end_date: fermentation.target_end_date,
            status: fermentation.status,
            notes: fermentation.notes,
            created_at: fermentation.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_fermentation(
        target_end_date: Option<DateTime<Utc>>,
        status: FermentationStatus,
    ) -> Fermentation {
        let now = Utc::now();
        Fermentation {
            id: 1,
            user_id: 1,
            profile_id: 1,
            name: "Test Fermentation".to_string(),
            start_date: now - Duration::days(3),
            target_end_date,
            actual_end_date: None,
            status,
            success_rating: None,
            notes: None,
            ingredients_json: None,
            created_at: now,
            updated_at: now,
            profile_name: Some("Test Profile".to_string()),
            profile_type: Some("test".to_string()),
        }
    }

    #[test]
    fn test_should_show_countdown_with_future_target_active() {
        let future_date = Utc::now() + Duration::days(5);
        let fermentation = create_test_fermentation(Some(future_date), FermentationStatus::Active);
        assert!(fermentation.should_show_countdown());
    }

    #[test]
    fn test_should_show_countdown_with_future_target_paused() {
        let future_date = Utc::now() + Duration::days(5);
        let fermentation = create_test_fermentation(Some(future_date), FermentationStatus::Paused);
        assert!(fermentation.should_show_countdown());
    }

    #[test]
    fn test_should_not_show_countdown_when_completed() {
        let future_date = Utc::now() + Duration::days(5);
        let fermentation =
            create_test_fermentation(Some(future_date), FermentationStatus::Completed);
        assert!(!fermentation.should_show_countdown());
    }

    #[test]
    fn test_should_not_show_countdown_when_failed() {
        let future_date = Utc::now() + Duration::days(5);
        let fermentation = create_test_fermentation(Some(future_date), FermentationStatus::Failed);
        assert!(!fermentation.should_show_countdown());
    }

    #[test]
    fn test_should_not_show_countdown_without_target_date() {
        let fermentation = create_test_fermentation(None, FermentationStatus::Active);
        assert!(!fermentation.should_show_countdown());
    }

    #[test]
    fn test_should_not_show_countdown_when_expired() {
        let past_date = Utc::now() - Duration::days(1);
        let fermentation = create_test_fermentation(Some(past_date), FermentationStatus::Active);
        assert!(!fermentation.should_show_countdown());
    }

    #[test]
    fn test_is_schedule_finished_when_expired_and_active() {
        let past_date = Utc::now() - Duration::days(1);
        let fermentation = create_test_fermentation(Some(past_date), FermentationStatus::Active);
        assert!(fermentation.is_schedule_finished());
    }

    #[test]
    fn test_is_schedule_finished_when_expired_and_paused() {
        let past_date = Utc::now() - Duration::days(1);
        let fermentation = create_test_fermentation(Some(past_date), FermentationStatus::Paused);
        assert!(fermentation.is_schedule_finished());
    }

    #[test]
    fn test_is_not_schedule_finished_when_completed() {
        let past_date = Utc::now() - Duration::days(1);
        let fermentation =
            create_test_fermentation(Some(past_date), FermentationStatus::Completed);
        assert!(!fermentation.is_schedule_finished());
    }

    #[test]
    fn test_is_not_schedule_finished_when_future_date() {
        let future_date = Utc::now() + Duration::days(5);
        let fermentation = create_test_fermentation(Some(future_date), FermentationStatus::Active);
        assert!(!fermentation.is_schedule_finished());
    }

    #[test]
    fn test_countdown_display_multiple_days() {
        let target = Utc::now() + Duration::days(5);
        let fermentation = create_test_fermentation(Some(target), FermentationStatus::Active);
        let countdown = fermentation.countdown_display().unwrap();
        assert!(countdown.contains("days"));
    }

    #[test]
    fn test_countdown_display_one_day() {
        let target = Utc::now() + Duration::days(1) + Duration::hours(2);
        let fermentation = create_test_fermentation(Some(target), FermentationStatus::Active);
        let countdown = fermentation.countdown_display().unwrap();
        assert!(countdown.contains("1 day"));
    }

    #[test]
    fn test_countdown_display_final_day_with_hours_and_minutes() {
        let target = Utc::now() + Duration::hours(5) + Duration::minutes(30);
        let fermentation = create_test_fermentation(Some(target), FermentationStatus::Active);
        let countdown = fermentation.countdown_display().unwrap();
        assert!(countdown.contains("h"));
        assert!(countdown.contains("m"));
    }

    #[test]
    fn test_countdown_display_returns_none_when_expired() {
        let past_date = Utc::now() - Duration::hours(1);
        let fermentation = create_test_fermentation(Some(past_date), FermentationStatus::Active);
        assert!(fermentation.countdown_display().is_none());
    }

    #[test]
    fn test_countdown_display_returns_none_without_target_date() {
        let fermentation = create_test_fermentation(None, FermentationStatus::Active);
        assert!(fermentation.countdown_display().is_none());
    }
}
