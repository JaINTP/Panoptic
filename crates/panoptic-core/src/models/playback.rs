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
            let p_tot_secs = self.progress_ms / 1000;
            let p_h = p_tot_secs / 3600;
            let p_m = (p_tot_secs % 3600) / 60;
            let p_s = p_tot_secs % 60;
            let p_m_total = p_tot_secs / 60;

            let d_tot_secs = self.duration_ms / 1000;
            let d_h = d_tot_secs / 3600;
            let d_m = (d_tot_secs % 3600) / 60;
            let d_s = d_tot_secs % 60;
            let d_m_total = d_tot_secs / 60;

            let format_time = |ms: u32| -> String {
                let tot_secs = ms / 1000;
                let h = tot_secs / 3600;
                let m = (tot_secs % 3600) / 60;
                let s = tot_secs % 60;
                if h > 0 {
                    format!("{}:{:02}:{:02}", h, m, s)
                } else {
                    format!("{}:{:02}", m, s)
                }
            };

            let p_formatted = format_time(self.progress_ms);
            let d_formatted = format_time(self.duration_ms);

            template
                .replace("{title}", &self.title)
                .replace("{artist}", &self.artist)
                .replace("{album}", &self.album)
                .replace("{progress_ms}", &self.progress_ms.to_string())
                .replace("{duration_ms}", &self.duration_ms.to_string())
                .replace("{progress}", &p_formatted)
                .replace("{duration}", &d_formatted)
                .replace("{progress_h}", &p_h.to_string())
                .replace("{progress_m}", &format!("{:02}", p_m))
                .replace("{progress_s}", &format!("{:02}", p_s))
                .replace("{progress_m_raw}", &p_m.to_string())
                .replace("{progress_s_raw}", &p_s.to_string())
                .replace("{progress_m_total}", &p_m_total.to_string())
                .replace("{progress_m_total_padded}", &format!("{:02}", p_m_total))
                .replace("{progress_s_total}", &p_tot_secs.to_string())
                .replace("{duration_h}", &d_h.to_string())
                .replace("{duration_m}", &format!("{:02}", d_m))
                .replace("{duration_s}", &format!("{:02}", d_s))
                .replace("{duration_m_raw}", &d_m.to_string())
                .replace("{duration_s_raw}", &d_s.to_string())
                .replace("{duration_m_total}", &d_m_total.to_string())
                .replace("{duration_m_total_padded}", &format!("{:02}", d_m_total))
                .replace("{duration_s_total}", &d_tot_secs.to_string())
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
            progress_ms: 65000,   // 1m 5s
            duration_ms: 3665000, // 1h 1m 5s
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
            "Test Title - Test Artist (Test Album) [65000/3665000]"
        );

        // Formatted pattern
        assert_eq!(
            state.format("{title} - {progress}/{duration}"),
            "Test Title - 1:05/1:01:05"
        );

        // Components pattern
        assert_eq!(
            state.format("{progress_h}:{progress_m}:{progress_s} ({progress_m_total})"),
            "0:01:05 (1)"
        );
        assert_eq!(
            state.format("{duration_h}:{duration_m}:{duration_s} ({duration_m_total})"),
            "1:01:05 (61)"
        );

        // Raw components pattern
        assert_eq!(state.format("{progress_m_raw}:{progress_s_raw}"), "1:5");

        // Empty title yields empty string
        let empty_state = PlaybackState::default();
        assert_eq!(empty_state.format("Now Playing: {title}"), "");
    }
}
