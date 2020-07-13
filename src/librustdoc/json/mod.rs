mod conversions;
mod types;

use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

use rustc_data_structures::fx::FxHashMap;
use rustc_span::edition::Edition;

use crate::clean;
use crate::config::{RenderInfo, RenderOptions};
use crate::error::Error;
use crate::formats::cache::Cache;
use crate::formats::FormatRenderer;
use crate::html::render::cache::ExternalLocation;

#[derive(Clone)]
pub struct JsonRenderer {
    index: Rc<RefCell<FxHashMap<types::Id, types::Item>>>,
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
        Ok((JsonRenderer { index: Rc::new(RefCell::new(FxHashMap::default())) }, krate))
    }

    fn item(&mut self, item: clean::Item, _cache: &Cache) -> Result<(), Error> {
        self.index.borrow_mut().insert(item.def_id.into(), item.into());
        Ok(())
    }

    fn mod_item_in(
        &mut self,
        item: &clean::Item,
        _item_name: &str,
        _cache: &Cache,
    ) -> Result<(), Error> {
        self.index.borrow_mut().insert(item.def_id.into(), item.clone().into());
        Ok(())
    }

    fn mod_item_out(&mut self, _item_name: &str) -> Result<(), Error> {
        // debug!("Exiting module: {}", item_name);
        Ok(())
    }

    fn after_krate(&mut self, krate: &clean::Crate, cache: &Cache) -> Result<(), Error> {
        debug!("Done with crate");
        let type_to_trait_impls = cache
            .impls
            .clone()
            .into_iter()
            .filter(|(k, _)| k.krate == rustc_span::def_id::LOCAL_CRATE)
            .map(|(k, v)| {
                for i in &v {
                    self.index
                        .borrow_mut()
                        .insert(i.impl_item.def_id.into(), i.clone().impl_item.into());
                }
                (k.into(), v.into_iter().map(|i| i.impl_item.def_id.into()).collect())
            })
            .collect();
        let trait_to_implementors = cache
            .implementors
            .clone()
            .into_iter()
            .map(|(k, v)| {
                (
                    k.into(),
                    v.into_iter()
                        .filter(|i| i.impl_item.def_id.krate == rustc_span::def_id::LOCAL_CRATE)
                        .map(|i| i.impl_item.def_id.into())
                        .collect(),
                )
            })
            .filter(|(_, v): &(types::Id, Vec<types::Id>)| !v.is_empty())
            .collect();
        let output = types::Crate {
            root: types::Id("0:0".to_string()),
            version: krate.version.clone(),
            includes_private: cache.document_private,
            index: (*self.index).clone().into_inner(),
            type_to_trait_impls,
            trait_to_implementors,
            extern_crates: cache
                .extern_locations
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_u32(),
                        (
                            v.0.clone(),
                            match &v.2 {
                                ExternalLocation::Remote(s) => Some(s.clone()),
                                _ => None,
                            },
                        ),
                    )
                })
                .collect(),
        };
        serde_json::ser::to_writer_pretty(&File::create("test.json").unwrap(), &output).unwrap();
        Ok(())
    }

    fn after_run(&mut self, _diag: &rustc_errors::Handler) -> Result<(), Error> {
        Ok(())
    }
}
