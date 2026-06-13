use panoptic_core::MediaProvider;

pub fn create_native_provider() -> Box<dyn MediaProvider> {
    #[cfg(target_os = "linux")]
    {
        Box::new(panoptic_provider_linux::LocalMprisProvider::new())
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(panoptic_provider_windows::LocalSmtcProvider::new())
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        unimplemented!("Platform not supported")
    }
}
