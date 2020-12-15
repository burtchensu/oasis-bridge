//! Bridge runtime.
use oasis_core_runtime::common::version::Version;
use oasis_runtime_sdk::{
    self as sdk,
    module::Module,
    modules,
    phf::{self, phf_map},
};

/*
runtime! {
    #[version(0, 0, 1)]
    pub struct Runtime {
        accounts: modules::accounts::Module,
    }
}
*/

/// Bridge runtime.
pub struct Runtime;

// TODO: Add a custom procedural macro for easier runtime specification.
impl sdk::Runtime for Runtime {
    const VERSION: Version = Version::new(0, 0, 1);

    const CALLABLE_MODULES: phf::Map<&'static str, sdk::runtime::ModuleInfo> = phf_map! {
        "accounts" => sdk::runtime::ModuleInfo{
            name: modules::accounts::Module::NAME,
            dispatch: modules::accounts::Module::dispatch,
        },

        "bridge" => sdk::runtime::ModuleInfo{
            name: oasis_module_bridge::Module::NAME,
            dispatch: oasis_module_bridge::Module::dispatch,
        },
    };
}
