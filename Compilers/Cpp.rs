use crate::caps::Capability;
use crate::diag::Diagnostic;
use crate::plugin::{unimplemented_diagnostic, Context, Contract, Outcome, Plugin, Version};

pub static CONTRACT: Contract = Contract {
    id: "plugin.cpp",
    display_name: "C/C++",
    version: Version::new(0, 1, 0),
    produces_artifacts: false,
    summary: "Compiles C and C++ translation units into Android native libraries.",
    inputs: &["cpp.source", "cpp.header", "native.build.config"],
    outputs: &["native.object", "native.library"],
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
        "Packaging libraries into an APK; that is the Apk plugin's work.",
        "Providing a C or C++ standard library implementation.",
        "Deciding which ABIs a project targets; that comes from the project model.",
    ],
};

pub struct CppPlugin;

pub static PLUGIN: CppPlugin = CppPlugin;

impl Plugin for CppPlugin {
    fn contract(&self) -> &'static Contract {
        &CONTRACT
    }

    fn execute(&self, _ctx: &mut Context<'_>) -> Result<Outcome, Diagnostic> {
        Err(unimplemented_diagnostic(&CONTRACT))
    }
}
