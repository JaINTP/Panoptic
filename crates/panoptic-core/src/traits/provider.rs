use crate::models::playback::PlaybackState;
use std::future::Future;
use std::pin::Pin;

pub trait MediaProvider: Send + Sync {
    fn fetch_now_playing(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<PlaybackState, String>> + Send + '_>>;
}
