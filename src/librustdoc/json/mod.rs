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
    fn insert(&self, item: clean::Item) {
        self.index.borrow_mut().insert(item.def_id.into(), item.into());
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

    fn item(&mut self, item: clean::Item, _cache: &Cache) -> Result<(), Error> {
        use clean::ItemEnum::*;
        // Flatten items that recursively store other items
        match item.inner.clone() {
            StructItem(s) => s.fields.into_iter().for_each(|i| self.insert(i)),
            UnionItem(u) => u.fields.into_iter().for_each(|i| self.insert(i)),
            VariantItem(clean::Variant { kind: clean::VariantKind::Struct(v) }) => {
                v.fields.into_iter().for_each(|i| self.insert(i));
            }
            EnumItem(e) => e.variants.into_iter().for_each(|i| self.insert(i)),
            TraitItem(t) => t.items.into_iter().for_each(|i| self.insert(i)),
            ImplItem(i) => i.items.into_iter().for_each(|i| self.insert(i)),
            _ => {}
        }
        self.insert(item);
        Ok(())
    }

    fn mod_item_in(
        &mut self,
        item: &clean::Item,
        _item_name: &str,
        _cache: &Cache,
    ) -> Result<(), Error> {
        self.insert(item.clone());
        Ok(())
    }

    fn mod_item_out(&mut self, _item_name: &str) -> Result<(), Error> {
        // debug!("Exiting module: {}", item_name);
        Ok(())
    }

    fn after_krate(&mut self, krate: &clean::Crate, cache: &Cache) -> Result<(), Error> {
        debug!("Done with crate");
        // let type_to_trait_impls = cache
        //     .impls
        //     .clone()
        //     .into_iter()
        //     .filter(|(k, _)| k.is_local())
        //     .map(|(k, v)| {
        //         v.clone().into_iter().for_each(|i| {
        //             self.insert(i.impl_item.clone());
        //             if let clean::ImplItem(inner_impl) = i.impl_item.inner {
        //                 inner_impl.items.into_iter().for_each(|i| self.insert(i));
        //             } else {
        //                 unreachable!()
        //             }
        //         });
        //         (k.into(), v.into_iter().map(|i| i.impl_item.def_id.into()).collect())
        //     })
        //     .collect();
        // let trait_to_implementors = cache
        //     .implementors
        //     .clone()
        //     .into_iter()
        //     .map(|(k, v)| {
        //         (
        //             k.into(),
        //             v.into_iter()
        //                 .filter(|i| i.impl_item.def_id.is_local())
        //                 .map(|i| i.impl_item.def_id.into())
        //                 .collect(),
        //         )
        //     })
        //     // TODO: check that k is local
        //     .filter(|(_k, v): &(types::Id, Vec<types::Id>)| !v.is_empty())
        //     .collect();
        let output = types::Crate {
            root: types::Id("0:0".to_string()),
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
            // type_to_trait_impls,
            // trait_to_implementors,
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
