use crate::caps::Capability;
use crate::diag::Diagnostic;
use crate::plugin::{unimplemented_diagnostic, Context, Contract, Outcome, Plugin, Version};

pub static CONTRACT: Contract = Contract {
    id: "plugin.rust",
    display_name: "Rust",
    version: Version::new(0, 1, 0),
    produces_artifacts: false,
    summary: "Compiles Rust crates for Android targets.",
    inputs: &[
        "rust.source",
        "cargo.manifest",
        "native.build.config",
    ],
    outputs: &[
        "native.object",
        "native.library",
    ],
    required_capabilities: &[
        Capability::FsRead,
        Capability::TempStorage,
        Capability::Cache,
        Capability::Native,
    ],
    forbidden_capabilities: &[
        Capability::Network,
        Capability::Internet,
        Capability::KeyAccess,
        Capability::SensitiveOutput,
    ],
    non_responsibilities: &[
        "Resolving crates from a registry; that requires a reviewed dependency resolution design (directive section 62).",
        "Packaging libraries into an APK.",
        "Replacing rustc for the Core's own build until self-hosting is real.",
    ],
};

pub struct RustPlugin;

pub static PLUGIN: RustPlugin = RustPlugin;

impl Plugin for RustPlugin {
    fn contract(&self) -> &'static Contract {
        &CONTRACT
    }

    fn execute(&self, _ctx: &mut Context<'_>) -> Result<Outcome, Diagnostic> {
        Err(unimplemented_diagnostic(&CONTRACT))
    }
}
