//! # amethyst_assets
//!
//! Asset management crate.
//! Designed with the following goals in mind:
//!
//! * Extensibility
//! * Asynchronous & Parallel using Rayon
//! * Allow different sources
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

#[cfg(feature = "json")]
#[cfg(not(feature = "atelier"))]
pub use crate::formats::JsonFormat;
#[cfg(not(feature = "atelier"))]
pub use crate::{
    asset::{Asset, Format, FormatValue, ProcessableAsset},
    cache::Cache,
    dyn_format::FormatRegisteredData,
    formats::RonFormat,
    helper::AssetLoaderSystemData,
    loader::Loader,
    prefab::{AssetPrefab, Prefab, PrefabData, PrefabLoader, PrefabLoaderSystem},
    progress::{Completion, Progress, ProgressCounter, Tracker},
    reload::{HotReloadBundle, HotReloadStrategy, HotReloadSystem, Reload, SingleFile},
    source::{Directory, Source},
    storage::{AssetStorage, Handle, ProcessingState, Processor, WeakHandle},
};
#[cfg(not(feature = "atelier"))]
pub use rayon::ThreadPool;

#[cfg(not(feature = "atelier"))]
mod asset;
#[cfg(not(feature = "atelier"))]
mod cache;
#[cfg(not(feature = "atelier"))]
mod dyn_format;
#[cfg(not(feature = "atelier"))]
mod error;
#[cfg(not(feature = "atelier"))]
mod formats;
#[cfg(not(feature = "atelier"))]
mod helper;
#[cfg(not(feature = "atelier"))]
mod loader;
#[cfg(not(feature = "atelier"))]
mod prefab;
#[cfg(not(feature = "atelier"))]
mod progress;
#[cfg(not(feature = "atelier"))]
mod reload;
#[cfg(not(feature = "atelier"))]
mod source;
#[cfg(not(feature = "atelier"))]
mod storage;

// used in macros. Private API otherwise.
#[doc(hidden)]
#[cfg(not(feature = "atelier"))]
pub use crate::dyn_format::{DeserializeFn, Registry};

// used in macros. Private API otherwise.
#[doc(hidden)]
pub use {erased_serde, inventory, lazy_static};

#[cfg(feature = "atelier")]
pub use amethyst_atelier::*;
