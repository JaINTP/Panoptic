pub mod models {
    pub mod auth;
    pub mod playback;
}
pub mod traits {
    pub mod provider;
}

pub use models::auth::AuthState;
pub use models::playback::PlaybackState;
pub use traits::provider::MediaProvider;
