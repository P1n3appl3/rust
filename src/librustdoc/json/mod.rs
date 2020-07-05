mod types;

use rustc_span::edition::Edition;

use crate::clean;
use crate::config::{RenderInfo, RenderOptions};
use crate::error::Error;
use crate::formats::cache::Cache;
use crate::formats::FormatRenderer;

#[derive(Clone)]
pub struct JsonRenderer {}

impl FormatRenderer for JsonRenderer {
    fn init(
        krate: clean::Crate,
        _options: RenderOptions,
        _render_info: RenderInfo,
        _edition: Edition,
        _cache: &mut Cache,
    ) -> Result<(Self, clean::Crate), Error> {
        debug!("Initializing json renderer");
        Ok((JsonRenderer {}, krate))
    }

    fn item(&mut self, item: clean::Item, _cache: &Cache) -> Result<(), Error> {
        debug!("Documenting item: {:?}", item.name.unwrap_or_default());
        Ok(())
    }

    fn mod_item_in(
        &mut self,
        _item: &clean::Item,
        item_name: &str,
        _cache: &Cache,
    ) -> Result<(), Error> {
        debug!("Entering module: {}", item_name);
        Ok(())
    }

    fn mod_item_out(&mut self, item_name: &str) -> Result<(), Error> {
        debug!("Exiting module: {}", item_name);
        Ok(())
    }

    fn after_krate(&mut self, _krate: &clean::Crate, _cache: &Cache) -> Result<(), Error> {
        debug!("Done with crate");
        Ok(())
    }

    fn after_run(&mut self, _diag: &rustc_errors::Handler) -> Result<(), Error> {
        Ok(())
    }
}
