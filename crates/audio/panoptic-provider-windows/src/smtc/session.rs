#[cfg(target_os = "windows")]
use panoptic_core::PlaybackState;
#[cfg(target_os = "windows")]
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

/// Set once at startup by `NativeMediaPlugin::setup()` via `app_handle.path().app_cache_dir()`.
/// Falls back to `std::env::temp_dir()` if never set.
static ART_CACHE_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

pub fn set_art_cache_dir(path: std::path::PathBuf) {
    let _ = ART_CACHE_DIR.set(path);
}

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
        state.duration_ms =
            (timeline.EndTime().map_err(|e| e.to_string())?.Duration / 10000) as u32;
        state.progress_ms =
            (timeline.Position().map_err(|e| e.to_string())?.Duration / 10000) as u32;
        state.is_playing = playback_info.PlaybackStatus().map_err(|e| e.to_string())?
            == windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;

        // IRandomAccessStreamReference is not Send, so all thumbnail COM work is
        // confined to a plain synchronous helper. No non-Send type crosses any
        // await boundary in this function.
        if let Ok(thumb_ref) = media_properties.Thumbnail() {
            if let Some(bytes) = try_read_thumbnail_sync(thumb_ref) {
                let ext = detect_image_ext(&bytes);
                let cache_dir = ART_CACHE_DIR
                    .get()
                    .cloned()
                    .unwrap_or_else(std::env::temp_dir);
                let _ = std::fs::create_dir_all(&cache_dir);
                let filename = format!("panoptic_art.{}", ext);
                let art_path = cache_dir.join(&filename);
                if std::fs::write(&art_path, &bytes).is_ok() {
                    let url_path = art_path.to_string_lossy().replace('\\', "/");
                    state.art_source = format!("file:///{}", url_path);
                }
            }
        }

        Ok(state)
    }

    #[cfg(not(target_os = "windows"))]
    pub async fn get_active_session_state() -> Result<panoptic_core::PlaybackState, String> {
        Err("SMTC is only available on Windows".to_string())
    }
}

/// Detect whether bytes are PNG or JPEG by magic bytes; defaults to "jpg".
#[cfg(target_os = "windows")]
fn detect_image_ext(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else {
        "jpg"
    }
}

/// Read SMTC thumbnail bytes without any async/await so non-Send COM types
/// never cross an await point in the calling async function.
/// WinRT async ops are spin-polled via IAsyncInfo::Status().
#[cfg(target_os = "windows")]
fn try_read_thumbnail_sync(
    thumb_ref: windows::Storage::Streams::IRandomAccessStreamReference,
) -> Option<Vec<u8>> {
    use windows::Foundation::{AsyncStatus, IAsyncInfo};
    use windows::Storage::Streams::{DataReader, IInputStream, IRandomAccessStream};
    use windows::core::Interface;

    // Start the open operation and spin-poll via IAsyncInfo until it completes.
    let open_op = thumb_ref.OpenReadAsync().ok()?;
    {
        let info = open_op.cast::<IAsyncInfo>().ok()?;
        loop {
            let s = info.Status().ok()?;
            if s == AsyncStatus::Completed {
                break;
            } else if s == AsyncStatus::Error || s == AsyncStatus::Canceled {
                return None;
            }
            std::thread::yield_now();
        }
    }
    let stream = open_op.GetResults().ok()?;

    // Cast to IRandomAccessStream to read Size(), then to IInputStream for DataReader.
    let rand_stream = stream.cast::<IRandomAccessStream>().ok()?;
    let size = rand_stream.Size().ok().filter(|&s| s > 0 && s < 10_000_000)?;
    let input_stream = rand_stream.cast::<IInputStream>().ok()?;
    let reader = DataReader::CreateDataReader(&input_stream).ok()?;

    // Start the load operation and spin-poll until it completes.
    let load_op = reader.LoadAsync(size as u32).ok()?;
    {
        let info = load_op.cast::<IAsyncInfo>().ok()?;
        loop {
            let s = info.Status().ok()?;
            if s == AsyncStatus::Completed {
                break;
            } else if s == AsyncStatus::Error || s == AsyncStatus::Canceled {
                return None;
            }
            std::thread::yield_now();
        }
    }
    let bytes_read = load_op.GetResults().ok()?;

    let mut buf = vec![0u8; bytes_read as usize];
    reader.ReadBytes(&mut buf).ok()?;
    Some(buf)
}
