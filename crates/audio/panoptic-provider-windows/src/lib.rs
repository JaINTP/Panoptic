pub mod smtc {
    pub mod provider;
    pub mod session;
}

pub use smtc::provider::LocalSmtcProvider;
pub use smtc::session::set_art_cache_dir;
