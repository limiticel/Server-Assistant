mod devtools;
mod internal_systems;
mod rag;
mod sales;
mod server_admin;
mod tests;
mod web_search;

use crate::config::Settings;
use crate::registry::Registry;

pub fn build_registry(settings: &Settings) -> Registry {
    let mut registry = Registry::new();

    if settings.enable_internal_systems {
        internal_systems::register(&mut registry);
    }

    if settings.enable_sales {
        sales::register(&mut registry);
    }

    if settings.enable_devtools {
        devtools::register(&mut registry);
    }

    if settings.enable_server_admin_tools {
        server_admin::register(&mut registry);
    }

    web_search::register(&mut registry);

    if settings.rag_activate {
        rag::register(&mut registry);
    }

    registry
}
