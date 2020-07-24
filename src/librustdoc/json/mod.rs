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

impl JsonRenderer {
    fn insert(&self, item: clean::Item, cache: &Cache) {
        let id = item.def_id;
        let mut new_item: types::Item = item.into();
        if let types::ItemEnum::TraitItem(ref mut t) = new_item.inner {
            t.implementors = self.get_trait_implementors(id, cache)
        } else if let types::ItemEnum::StructItem(ref mut s) = new_item.inner {
            s.impls = self.get_impls(id, cache)
        } else if let types::ItemEnum::EnumItem(ref mut e) = new_item.inner {
            e.impls = self.get_impls(id, cache)
        }
        self.index.borrow_mut().insert(id.into(), new_item);
    }

    fn get_trait_implementors(
        &self,
        id: rustc_span::def_id::DefId,
        cache: &Cache,
    ) -> Vec<types::Id> {
        cache
            .implementors
            .get(&id)
            .map(|implementors| {
                implementors
                    .iter()
                    .map(|i| {
                        let item = &i.impl_item;
                        self.insert(item.clone(), cache);
                        item.def_id.into()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_impls(&self, id: rustc_span::def_id::DefId, cache: &Cache) -> Vec<types::Id> {
        cache
            .impls
            .get(&id)
            .map(|impls| {
                impls
                    .iter()
                    .map(|i| {
                        let item = &i.impl_item;
                        if item.def_id.is_local() {
                            self.insert(item.clone(), cache);
                            Some(item.def_id.into())
                        } else {
                            None
                        }
                    })
                    .filter(Option::is_some)
                    .map(Option::unwrap)
                    .collect()
            })
            .unwrap_or_default()
    }
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

    fn item(&mut self, item: clean::Item, cache: &Cache) -> Result<(), Error> {
        use clean::ItemEnum::*;
        // Flatten items that recursively store other items
        match item.inner.clone() {
            StructItem(s) => s.fields.into_iter().for_each(|i| self.insert(i, cache)),
            UnionItem(u) => u.fields.into_iter().for_each(|i| self.insert(i, cache)),
            VariantItem(clean::Variant { kind: clean::VariantKind::Struct(v) }) => {
                v.fields.into_iter().for_each(|i| self.insert(i, cache));
            }
            EnumItem(e) => e.variants.into_iter().for_each(|i| self.insert(i, cache)),
            TraitItem(t) => t.items.into_iter().for_each(|i| self.insert(i, cache)),
            ImplItem(i) => i.items.into_iter().for_each(|i| self.insert(i, cache)),
            _ => {}
        }
        self.insert(item.clone(), cache);
        Ok(())
    }

    fn mod_item_in(
        &mut self,
        item: &clean::Item,
        _item_name: &str,
        cache: &Cache,
    ) -> Result<(), Error> {
        self.insert(item.clone(), cache);
        Ok(())
    }

    fn mod_item_out(&mut self, _item_name: &str) -> Result<(), Error> {
        Ok(())
    }

    fn after_krate(&mut self, krate: &clean::Crate, cache: &Cache) -> Result<(), Error> {
        debug!("Done with crate");
        let output = types::Crate {
            root: types::Id(String::from("0:0")),
            version: krate.version.clone(),
            includes_private: cache.document_private,
            index: (*self.index).clone().into_inner(),
            // traits: cache.traits.clone().into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            traits: FxHashMap::default(),
            paths: cache
                .paths
                .clone()
                .into_iter()
                .chain(cache.external_paths.clone().into_iter())
                .map(|(k, (path, kind))| {
                    (
                        k.into(),
                        types::ItemSummary { crate_num: k.krate.as_u32(), path, kind: kind.into() },
                    )
                })
                .collect(),
            external_crates: cache
                .extern_locations
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_u32(),
                        types::ExternalCrate {
                            name: v.0.clone(),
                            html_root_url: match &v.2 {
                                ExternalLocation::Remote(s) => Some(s.clone()),
                                _ => None,
                            },
                        },
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
