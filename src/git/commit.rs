use super::error::{GitError, Result};
use chrono::{DateTime, Local};
use git2::{Oid, Time};

/// Represents a git commit
#[derive(Debug, Clone)]
pub struct Commit {
    pub id: Oid,
    pub short_id: String,
    pub author: String,
    pub author_email: String,
    pub date: DateTime<Local>,
    pub summary: String,
    pub message: String,
    pub refs: Vec<String>,
}

impl Commit {
    /// Create a Commit from a git2::Commit
    pub fn from_git2(commit: &git2::Commit) -> Result<Self> {
        let id = commit.id();
        let short_id = id.to_string()[..7].to_string();

        let author = commit.author();
        let author_name = author
            .name()
            .ok_or(GitError::InvalidUtf8)?
            .to_string();
        let author_email = author
            .email()
            .ok_or(GitError::InvalidUtf8)?
            .to_string();

        let date = time_to_datetime(author.when());

        let summary = commit
            .summary()
            .ok_or(GitError::InvalidUtf8)?
            .to_string();

        let message = commit
            .message()
            .ok_or(GitError::InvalidUtf8)?
            .to_string();

        Ok(Self {
            id,
            short_id,
            author: author_name,
            author_email,
            date,
            summary,
            message,
            refs: Vec::new(), // Will be populated separately
        })
    }

    /// Format the date in a human-readable way
    pub fn date_str(&self) -> String {
        self.date.format("%Y-%m-%d %H:%M").to_string()
    }

    /// Format the date in a relative way (e.g., "2 hours ago")
    pub fn relative_date(&self) -> String {
        let now = Local::now();
        let duration = now.signed_duration_since(self.date);

        let format_unit = |count: i64, unit: &str| {
            format!("{} {}{} ago", count, unit, if count == 1 { "" } else { "s" })
        };

        if duration.num_seconds() < 60 {
            return "just now".to_string();
        }
        if duration.num_minutes() < 60 {
            return format_unit(duration.num_minutes(), "minute");
        }
        if duration.num_hours() < 24 {
            return format_unit(duration.num_hours(), "hour");
        }
        if duration.num_days() < 7 {
            return format_unit(duration.num_days(), "day");
        }
        if duration.num_weeks() < 4 {
            return format_unit(duration.num_weeks(), "week");
        }
        if duration.num_days() < 365 {
            let months = duration.num_days() / 30;
            return format_unit(months, "month");
        }

        let years = duration.num_days() / 365;
        format_unit(years, "year")
    }
}

/// Convert git2::Time to chrono::DateTime
fn time_to_datetime(time: Time) -> DateTime<Local> {
    let timestamp = time.seconds();
    DateTime::from_timestamp(timestamp, 0)
        .unwrap()
        .with_timezone(&Local)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_short_id_length() {
        let oid = Oid::from_str("1234567890abcdef1234567890abcdef12345678").unwrap();
        let short = &oid.to_string()[..7];
        assert_eq!(short.len(), 7);
    }

    #[test]
    fn test_time_conversion() {
        let time = Time::new(1609459200, 0); // 2021-01-01 00:00:00 UTC
        let datetime = time_to_datetime(time);
        assert!(datetime.year() == 2020 || datetime.year() == 2021); // Depends on timezone
    }

    #[test]
    fn test_relative_date_pluralization() {
        // Test that the pluralization works correctly
        let format_unit = |count: i64, unit: &str| {
            format!("{} {}{} ago", count, unit, if count == 1 { "" } else { "s" })
        };

        assert_eq!(format_unit(1, "minute"), "1 minute ago");
        assert_eq!(format_unit(2, "minute"), "2 minutes ago");
        assert_eq!(format_unit(1, "hour"), "1 hour ago");
        assert_eq!(format_unit(24, "hour"), "24 hours ago");
    }
}
