mod conversions;
mod types;

// use std::fs::File;

use rustc_data_structures::fx::FxHashMap;
use rustc_span::edition::Edition;

use crate::clean;
use crate::config::{RenderInfo, RenderOptions};
use crate::error::Error;
use crate::formats::cache::Cache;
use crate::formats::FormatRenderer;

#[derive(Clone)]
pub struct JsonRenderer {
    index: FxHashMap<types::DefId, types::Item>,
}

impl FormatRenderer for JsonRenderer {
    fn init(
        krate: clean::Crate,
        _options: RenderOptions,
        _render_info: RenderInfo,
        _edition: Edition,
        _cache: &mut Cache,
    ) -> Result<(Self, clean::Crate), Error> {
        debug!("Initializing json renderer");
        Ok((JsonRenderer { index: FxHashMap::default() }, krate))
    }

    fn item(&mut self, item: clean::Item, _cache: &Cache) -> Result<(), Error> {
        debug!(
            "Documenting item: {}: {}",
            item.clone().name.unwrap_or_default().as_str(),
            serde_json::to_string(&types::Item::from(item)).unwrap().as_str()
        );
        // self.index.insert(item.def_id.into(), item.into()).unwrap();
        Ok(())
    }

    fn mod_item_in(
        &mut self,
        item: &clean::Item,
        item_name: &str,
        _cache: &Cache,
    ) -> Result<(), Error> {
        debug!(
            "Entering module: {}: {}",
            item_name,
            serde_json::to_string(&types::Item::from(item.clone())).unwrap().as_str(),
        );
        Ok(())
    }

    fn mod_item_out(&mut self, item_name: &str) -> Result<(), Error> {
        debug!("Exiting module: {}", item_name);
        Ok(())
    }

    fn after_krate(&mut self, _krate: &clean::Crate, _cache: &Cache) -> Result<(), Error> {
        debug!("Done with crate");
        // debug!("Index: {:?}", self.index);
        // debug!("Impls: {:?}", cache.impls);
        // debug!("Implementors: {:?}", cache.implementors);
        // debug!("Traits: {:?}", cache.traits);
        // debug!("Paths: {:?}", cache.paths);
        // for item in self.index.values() {
        //     debug!("{}", serde_json::to_string(item).unwrap());
        // }
        // serde_json::to_writer(
        //     &File::create("test.json").unwrap(),
        //     &types::Item::from(krate.module.clone().unwrap()),
        // )
        // .unwrap();
        Ok(())
    }

    fn after_run(&mut self, _diag: &rustc_errors::Handler) -> Result<(), Error> {
        Ok(())
    }
}
