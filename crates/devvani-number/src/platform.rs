//! Platform detection at compile time

#[derive(Debug)]
pub struct PlatformInfo {
    pub arch: Architecture,
    pub os: OperatingSystem,
    pub pointer_width: PointerWidth,
    pub endianness: Endianness,
}

#[derive(Debug)]
pub enum Architecture {
    X86_64,
    X86_32,
    Arm64,
    Arm32,
    Wasm32,
    Wasm64,
    RiscV64,
    Unknown,
}

#[derive(Debug)]
pub enum OperatingSystem {
    Windows,
    Linux,
    MacOS,
    Wasm,
    Embedded,
    Unknown,
}

#[derive(Debug)]
pub enum PointerWidth {
    Bits32,
    Bits64,
}

#[derive(Debug)]
pub enum Endianness {
    Little,
    Big,
}

pub const fn detect() -> PlatformInfo {
    PlatformInfo {
        arch: {
            #[cfg(target_arch = "x86_64")]
            {
                Architecture::X86_64
            }
            #[cfg(target_arch = "x86")]
            {
                Architecture::X86_32
            }
            #[cfg(target_arch = "aarch64")]
            {
                Architecture::Arm64
            }
            #[cfg(target_arch = "arm")]
            {
                Architecture::Arm32
            }
            #[cfg(target_arch = "wasm32")]
            {
                Architecture::Wasm32
            }
            #[cfg(not(any(
                target_arch = "x86_64",
                target_arch = "x86",
                target_arch = "aarch64",
                target_arch = "arm",
                target_arch = "wasm32"
            )))]
            {
                Architecture::Unknown
            }
        },
        os: {
            #[cfg(target_os = "windows")]
            {
                OperatingSystem::Windows
            }
            #[cfg(target_os = "linux")]
            {
                OperatingSystem::Linux
            }
            #[cfg(target_os = "macos")]
            {
                OperatingSystem::MacOS
            }
            #[cfg(target_arch = "wasm32")]
            {
                OperatingSystem::Wasm
            }
            #[cfg(not(any(
                target_os = "windows",
                target_os = "linux",
                target_os = "macos",
                target_arch = "wasm32"
            )))]
            {
                OperatingSystem::Unknown
            }
        },
        pointer_width: {
            #[cfg(target_pointer_width = "64")]
            {
                PointerWidth::Bits64
            }
            #[cfg(target_pointer_width = "32")]
            {
                PointerWidth::Bits32
            }
        },
        endianness: {
            #[cfg(target_endian = "little")]
            {
                Endianness::Little
            }
            #[cfg(target_endian = "big")]
            {
                Endianness::Big
            }
        },
    }
}

pub const PLATFORM: PlatformInfo = detect();

pub fn platform_report() -> String {
    format!(
        "वास्तुकला: {:?}\nप्रणाली: {:?}\nसूचक-चौड़ाई: {:?}\nक्रम: {:?}",
        PLATFORM.arch, PLATFORM.os, PLATFORM.pointer_width, PLATFORM.endianness
    )
}
