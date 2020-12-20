use std::collections::HashMap;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};

pub use unic_langid::{langid, langids, LanguageIdentifier};

/// A simple Loader implementation, with statically-loaded fluent data.
/// Typically created with the [`static_loader!`] macro
///
/// [`static_loader!`]: ./macro.static_loader.html
pub struct StaticLoader {
    bundles: &'static HashMap<LanguageIdentifier, FluentBundle<&'static FluentResource>>,
    fallbacks: &'static HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
    fallback: LanguageIdentifier,
}

impl StaticLoader {
    /// Construct a new `StaticLoader`.
    ///
    /// This is exposed as publicly so that it can be used inside the
    /// `static_loader!` macro. it's not meant to be called directly.
    #[doc(hidden)]
    pub fn new(
        bundles: &'static HashMap<LanguageIdentifier, FluentBundle<&'static FluentResource>>,
        fallbacks: &'static HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
        fallback: LanguageIdentifier,
    ) -> Self {
        Self {
            bundles,
            fallbacks,
            fallback,
        }
    }

    /// Convenience function to look up a string for a single language
    pub fn lookup_single_language<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> Option<String> {
        if let Some(bundle) = self.bundles.get(lang) {
            let pattern = if text_id.contains('.') {
                // TODO: #![feature(str_split_once)]
                let ids: Vec<_> = text_id.splitn(2, '.').collect();
                bundle
                    .get_message(ids[0])?
                    .attributes
                    .iter()
                    .find(|attribute| attribute.id == ids[1])?
                    .value
            } else {
                bundle.get_message(text_id)?.value?
            };
            let mut errors = Vec::new();
            let args = super::map_to_fluent_args(args);
            let value = bundle.format_pattern(pattern, args.as_ref(), &mut errors);
            if errors.is_empty() {
                Some(value.into())
            } else {
                panic!(
                    "Failed to format a message for locale {} and id {}.\nErrors\n{:?}",
                    lang, text_id, errors
                )
            }
        } else {
            panic!("Unknown language {}", lang)
        }
    }

    /// Convenience function to look up a string without falling back to the default fallback language
    pub fn lookup_no_default_fallback(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<String, FluentValue>>,
    ) -> Option<String> {
        if let Some(fallbacks) = self.fallbacks.get(lang) {
            for l in fallbacks {
                if let Some(val) = self.lookup_single_language(l, text_id, args) {
                    return Some(val);
                }
            }
        }

        None
    }
}

impl super::Loader for StaticLoader {
    // Traverse the fallback chain,
    fn lookup_complete<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> String {
        if let Some(fallbacks) = self.fallbacks.get(lang) {
            for l in fallbacks {
                if let Some(val) = self.lookup_single_language(l, text_id, args) {
                    return val;
                }
            }
        }
        if *lang != self.fallback {
            if let Some(val) = self.lookup_single_language(&self.fallback, text_id, args) {
                return val;
            }
        }
        format!("Unknown localization {}", text_id)
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        Box::new(self.fallbacks.keys())
    }
}
