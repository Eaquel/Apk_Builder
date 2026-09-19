use crate::caps::Capability;
use crate::diag::Diagnostic;
use crate::plugin::{unimplemented_diagnostic, Context, Contract, Outcome, Plugin, Version};

pub static CONTRACT: Contract = Contract {
    id: "omni.plugin.kotlin",
    display_name: "Kotlin",
    version: Version::new(0, 1, 0),
    produces_artifacts: false,
    summary: "Translates Kotlin source into the Omni compiler intermediate representation.",
    inputs: &["kotlin.source", "project.model", "classpath.index"],
    outputs: &["compiler.ir", "jvm.class", "diagnostics"],
    required_capabilities: &[
        Capability::FsRead,
        Capability::TempStorage,
        Capability::Cache,
    ],
    forbidden_capabilities: &[
        Capability::Network,
        Capability::Internet,
        Capability::ProcessExec,
        Capability::KeyAccess,
        Capability::SensitiveOutput,
    ],
    non_responsibilities: &[
        "Converting to DEX; that is the Dex plugin's work.",
        "Packaging or signing anything.",
        "Resolving external dependencies from a network.",
    ],
};

pub struct KotlinPlugin;

pub static PLUGIN: KotlinPlugin = KotlinPlugin;

impl Plugin for KotlinPlugin {
    fn contract(&self) -> &'static Contract {
        &CONTRACT
    }

    fn execute(&self, _ctx: &mut Context<'_>) -> Result<Outcome, Diagnostic> {
        Err(unimplemented_diagnostic(&CONTRACT))
    }
}
