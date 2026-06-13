pub mod mpris {
    pub mod parser;
    pub mod provider;
}

pub use mpris::provider::LocalMprisProvider;
