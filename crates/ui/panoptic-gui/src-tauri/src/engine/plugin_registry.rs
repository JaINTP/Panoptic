use panoptic_core::PanopticPlugin;
use std::sync::Arc;

pub struct PluginRegistry {
    pub plugins: Arc<Vec<Box<dyn PanopticPlugin>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(Vec::new()),
        }
    }

    pub fn register(mut self, plugin: Box<dyn PanopticPlugin>) -> Self {
        // Safe because we are building the registry during setup before multi-threading starts
        if let Some(list) = Arc::get_mut(&mut self.plugins) {
            list.push(plugin);
        }
        self
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
