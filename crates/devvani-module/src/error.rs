#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    #[error("koṣaḥ na prāptaḥ: module '{0}' not found")]
    KoshaNaPraptah(String),

    #[error("cakra-avalambanam: circular dependency detected: {0}")]
    ChakraAvalambanam(String),

    #[error("anumati-rahitam: visibility error — '{0}' is private")]
    AnumatiRahitam(String),

    #[error("aprāmāṇika-pakṣaḥ: unverified community package '{0}'")]
    ApramanikaPaksha(String),

    #[error("manifest parse error: {0}")]
    ManifestParseError(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
