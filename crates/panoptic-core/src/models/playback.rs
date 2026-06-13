use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PlaybackState {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub art_source: String,
    pub progress_ms: u32,
    pub duration_ms: u32,
    pub is_playing: bool,
    pub formatted_output: String,
}

impl PlaybackState {
    pub fn format(&self, template: &str) -> String {
        if self.title.is_empty() {
            String::new()
        } else {
            template
                .replace("{title}", &self.title)
                .replace("{artist}", &self.artist)
                .replace("{album}", &self.album)
                .replace("{progress_ms}", &self.progress_ms.to_string())
                .replace("{duration_ms}", &self.duration_ms.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_state_format() {
        let state = PlaybackState {
            title: "Test Title".to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            progress_ms: 12000,
            duration_ms: 240000,
            ..Default::default()
        };

        // Standard pattern
        assert_eq!(
            state.format("Now Playing: {title} by {artist}"),
            "Now Playing: Test Title by Test Artist"
        );

        // All fields pattern
        assert_eq!(
            state.format("{title} - {artist} ({album}) [{progress_ms}/{duration_ms}]"),
            "Test Title - Test Artist (Test Album) [12000/240000]"
        );

        // Empty title yields empty string
        let empty_state = PlaybackState::default();
        assert_eq!(empty_state.format("Now Playing: {title}"), "");
    }
}
