#[cfg(target_os = "windows")]
use panoptic_core::PlaybackState;
#[cfg(target_os = "windows")]
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

pub struct SmtcSessionManager;

impl SmtcSessionManager {
    #[cfg(target_os = "windows")]
    pub async fn get_active_session_state() -> Result<PlaybackState, String> {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .map_err(|e| e.to_string())?
            .await
            .map_err(|e| e.to_string())?;

        let session = manager.GetCurrentSession().map_err(|e| e.to_string())?;

        let timeline = session.GetTimelineProperties().map_err(|e| e.to_string())?;

        let media_properties = session
            .TryGetMediaPropertiesAsync()
            .map_err(|e| e.to_string())?
            .await
            .map_err(|e| e.to_string())?;

        let playback_info = session.GetPlaybackInfo().map_err(|e| e.to_string())?;

        let mut state = PlaybackState::default();
        state.title = media_properties
            .Title()
            .map_err(|e| e.to_string())?
            .to_string();
        state.artist = media_properties
            .Artist()
            .map_err(|e| e.to_string())?
            .to_string();
        state.album = media_properties
            .AlbumTitle()
            .map_err(|e| e.to_string())?
            .to_string();

        //End duration is 100ns units
        state.duration_ms =
            (timeline.EndTime().map_err(|e| e.to_string())?.Duration / 10000) as u32;
        state.progress_ms =
            (timeline.Position().map_err(|e| e.to_string())?.Duration / 10000) as u32;

        state.is_playing = playback_info.PlaybackStatus().map_err(|e| e.to_string())? == windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;

        Ok(state)
    }

    #[cfg(not(target_os = "windows"))]
    pub async fn get_active_session_state() -> Result<panoptic_core::PlaybackState, String> {
        Err("SMTC is only available on Windows".to_string())
    }
}
