use crate::smtc::session::SmtcSessionManager;
use panoptic_core::{MediaProvider, PlaybackState};
use std::future::Future;
use std::pin::Pin;

pub struct LocalSmtcProvider;

impl LocalSmtcProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalSmtcProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaProvider for LocalSmtcProvider {
    fn fetch_now_playing(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<PlaybackState, String>> + Send + '_>> {
        Box::pin(async move { SmtcSessionManager::get_active_session_state().await })
    }
}
