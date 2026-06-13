use crate::mpris::parser::MprisMetadataParser;
use panoptic_core::{MediaProvider, PlaybackState};
use std::future::Future;
use std::pin::Pin;

pub struct LocalMprisProvider;

impl LocalMprisProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalMprisProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaProvider for LocalMprisProvider {
    fn fetch_now_playing(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<PlaybackState, String>> + Send + '_>> {
        Box::pin(async move {
            let conn = zbus::Connection::session()
                .await
                .map_err(|e| e.to_string())?;
            // Targeting spotify specifically for now as an example, but could loop over active players
            MprisMetadataParser::parse(&conn, "org.mpris.MediaPlayer2.spotify")
                .await
                .map_err(|e| e.to_string())
        })
    }
}
