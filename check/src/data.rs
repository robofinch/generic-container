use std::path::{Path, PathBuf};

use anyhow::anyhow;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Channel {
    Stable,
    Nightly,
}

impl Channel {
    pub const fn all_channels() -> &'static [Self] {
        &[Self::Stable, Self::Nightly]
    }

    pub const fn default_channels() -> &'static [Self] {
        Self::all_channels()
    }

    pub fn parse(channel: &str) -> anyhow::Result<Self> {
        Ok(match channel {
            "stable"  => Self::Stable,
            "nightly" => Self::Nightly,
            _ => return Err(anyhow!("Unknown channel name: {channel}")),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Target {
    Native,
    AppleSilicon,
    Linux,
    Windows,
    Wasm,
    Custom(String),
}

impl Target {
    pub const fn all_targets() -> &'static [Self] {
        &[
            Self::AppleSilicon,
            Self::Linux,
            Self::Windows,
            Self::Wasm,
        ]
    }

    pub const fn default_targets() -> &'static [Self] {
        &[Self::AppleSilicon, Self::Wasm]
    }

    pub fn parse(target: String) -> Self {
        match &*target {
            "native"                                           => Self::Native,
            "apple" | "apple-silicon" | "aarch64-apple-darwin" => Self::AppleSilicon,
            "linux" | "x86_64-unknown-linux-gnu"               => Self::Linux,
            "windows" | "x86_64-pc-windows-msvc"               => Self::Windows,
            "wasm" | "wasm32" | "wasm32-unknown-unknown"       => Self::Wasm,
            _                                                  => Self::Custom(target),
        }
    }

    pub const fn target_triple(&self) -> Option<&str> {
        Some(match self {
            Self::Native         => return None,
            Self::AppleSilicon   => "aarch64-apple-darwin",
            Self::Linux          => "x86_64-unknown-linux-gnu",
            Self::Windows        => "x86_64-pc-windows-msvc",
            Self::Wasm           => "wasm32-unknown-unknown",
            Self::Custom(target) => target.as_str()
        })
    }
}

#[expect(clippy::upper_case_acronyms, reason = "Looks better")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Package {
    GenericContainer,
    ThreadCheckedMutex,
}

impl Package {
    pub const fn all_packages() -> &'static [Self] {
        &[
            Self::GenericContainer, Self::ThreadCheckedMutex,
        ]
    }

    pub const fn default_packages() -> &'static [Self] {
        Self::all_packages()
    }

    pub fn parse(package: &str) -> anyhow::Result<Self> {
        Ok(match package {
            "container" | "generic-container" => Self::GenericContainer,
            "mutex" | "thread-checked-mutex"  => Self::ThreadCheckedMutex,
            _ => return Err(anyhow!("Unknown package name: {package}")),
        })
    }

    pub const fn package_name(self) -> &'static str {
        match self {
            Self::GenericContainer   => "generic-container",
            Self::ThreadCheckedMutex => "thread-checked-mutex",
        }
    }

    pub fn package_dir(self) -> PathBuf {
        Path::new("crates/").join(self.package_name())
    }

    pub fn dependencies(self) -> Vec<PathBuf> {
        let mut dependencies = Vec::with_capacity(5);
        dependencies.extend(
            ["Cargo.toml", "clippy.toml", "check", "Justfile"].map(PathBuf::from),
        );
        dependencies.push(self.package_dir());
        dependencies
    }

    pub fn flags(self, _channel: Channel, _target: &Target) -> Vec<&'static str> {
        ["--package", self.package_name()].to_vec()
    }
}
