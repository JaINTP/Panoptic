use panoptic_core::PlaybackState;
use std::collections::HashMap;
use zbus::{zvariant, Connection};

pub struct MprisMetadataParser;

impl MprisMetadataParser {
    pub async fn parse(conn: &Connection, player: &str) -> zbus::Result<PlaybackState> {
        let proxy = zbus::Proxy::new(
            conn,
            player,
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player",
        )
        .await?;

        let metadata: HashMap<String, zvariant::Value> = proxy.get_property("Metadata").await?;
        let playback_status: String = proxy.get_property("PlaybackStatus").await?;
        let position_us: i64 = proxy.get_property("Position").await.unwrap_or(0);

        Ok(Self::parse_metadata_map(
            &metadata,
            &playback_status,
            position_us,
        ))
    }

    pub fn parse_metadata_map(
        metadata: &HashMap<String, zvariant::Value>,
        playback_status: &str,
        position_us: i64,
    ) -> PlaybackState {
        let mut artist = String::new();
        if let Some(zvariant::Value::Array(artists)) = metadata.get("xesam:artist") {
            artist = artists
                .iter()
                .filter_map(|v| {
                    if let zvariant::Value::Str(s) = v {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
        }

        let title = metadata
            .get("xesam:title")
            .and_then(|v| {
                if let zvariant::Value::Str(s) = v {
                    Some(s.as_str().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let album = metadata
            .get("xesam:album")
            .and_then(|v| {
                if let zvariant::Value::Str(s) = v {
                    Some(s.as_str().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let duration_ms = metadata
            .get("mpris:length")
            .and_then(|v| match v {
                zvariant::Value::I64(i) => Some((*i / 1000) as u32),
                zvariant::Value::U64(u) => Some((*u / 1000) as u32),
                zvariant::Value::I32(i) => Some((*i as i64 / 1000) as u32),
                zvariant::Value::U32(u) => Some((*u as u64 / 1000) as u32),
                _ => None,
            })
            .unwrap_or(0);

        let art_source = metadata
            .get("mpris:artUrl")
            .and_then(|v| {
                if let zvariant::Value::Str(s) = v {
                    Some(s.as_str().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        PlaybackState {
            title,
            artist,
            album,
            art_source,
            progress_ms: (position_us / 1000) as u32,
            duration_ms,
            is_playing: playback_status == "Playing",
            formatted_output: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::{Array, Signature, Value};

    #[test]
    fn test_parse_metadata_map_basic() {
        let mut metadata = HashMap::new();
        metadata.insert("xesam:title".to_string(), Value::from("Test Title"));
        metadata.insert("xesam:album".to_string(), Value::from("Test Album"));
        metadata.insert("mpris:length".to_string(), Value::from(240000000i64));
        metadata.insert(
            "mpris:artUrl".to_string(),
            Value::from("http://example.com/art.png"),
        );

        let artist_sig = Signature::try_from("s").unwrap();
        let mut array = Array::new(artist_sig);
        array.append(Value::from("Artist A")).unwrap();
        array.append(Value::from("Artist B")).unwrap();
        metadata.insert("xesam:artist".to_string(), Value::Array(array));

        let state = MprisMetadataParser::parse_metadata_map(&metadata, "Playing", 120000000);

        assert_eq!(state.title, "Test Title");
        assert_eq!(state.artist, "Artist A, Artist B");
        assert_eq!(state.album, "Test Album");
        assert_eq!(state.art_source, "http://example.com/art.png");
        assert_eq!(state.progress_ms, 120000);
        assert_eq!(state.duration_ms, 240000);
        assert!(state.is_playing);
    }

    #[test]
    fn test_parse_metadata_map_missing_fields() {
        let metadata = HashMap::new();
        let state = MprisMetadataParser::parse_metadata_map(&metadata, "Paused", 0);

        assert_eq!(state.title, "");
        assert_eq!(state.artist, "");
        assert_eq!(state.album, "");
        assert_eq!(state.art_source, "");
        assert_eq!(state.progress_ms, 0);
        assert_eq!(state.duration_ms, 0);
        assert!(!state.is_playing);
    }
}
