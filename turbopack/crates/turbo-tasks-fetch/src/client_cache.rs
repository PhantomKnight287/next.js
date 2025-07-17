use std::{
    hash::Hash,
    sync::{Arc, LazyLock},
};

use quick_cache::sync::Cache;
use turbo_dyn_eq_hash::{
    DynEq, DynHash, impl_eq_for_dyn, impl_hash_for_dyn, impl_partial_eq_for_dyn,
};

const MAX_CLIENTS: usize = 16;
static CLIENT_CACHE: LazyLock<Cache<Arc<dyn ClientFactory>, reqwest::Client>> =
    LazyLock::new(|| Cache::new(MAX_CLIENTS));

/// Represents the configuration needed to construct a client. This allows us to cache clients keyed
/// by their configuration.
///
/// This is needed because [`reqwest::ClientBuilder`] does not implement the required traits. This
/// factory cannot be a closure because closures do not implement `Eq` or `Hash`.
pub trait ClientFactory: DynEq + DynHash + Send + Sync {
    fn try_build(&self) -> reqwest::Result<reqwest::Client>;
}

impl_partial_eq_for_dyn!(dyn ClientFactory);
impl_eq_for_dyn!(dyn ClientFactory);
impl_hash_for_dyn!(dyn ClientFactory);

/// Given a factory, returns a cached instance of `CLIENT_CACHE` if it exists, otherwise constructs
/// a new one.
///
/// The cache is bound in size to prevent acciental blowups or leaks. However, in practice, very
/// few clients should be created, likely only when the bundler configuration changes.
pub fn get_cached_client(
    factory: impl ClientFactory + Eq + Hash,
) -> reqwest::Result<reqwest::Client> {
    let factory = Arc::new(factory) as Arc<dyn ClientFactory>;
    CLIENT_CACHE.get_or_insert_with(&factory, {
        let factory = factory.clone();
        move || factory.try_build()
    })
}
