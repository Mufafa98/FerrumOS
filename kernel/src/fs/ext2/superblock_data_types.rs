#[derive(Debug)]
#[repr(u16)]
pub enum State {
    Valid,
    Error,
}
impl State {
    pub fn from_u16(value: u16) -> Self {
        match value {
            1 => State::Valid,
            2 => State::Error,
            _ => panic!("Invalid state value"),
        }
    }
}

#[derive(Debug)]
#[repr(u16)]
pub enum Error {
    Ignore,
    RemountReadOnly,
    Panic,
}
impl Error {
    pub fn from_u16(value: u16) -> Self {
        match value {
            1 => Error::Ignore,
            2 => Error::RemountReadOnly,
            3 => Error::Panic,
            _ => panic!("Invalid error handling value"),
        }
    }
}

#[derive(Debug)]
#[repr(u32)]
pub enum CreatorOS {
    Linux,
    Hurd,
    Masix,
    FreeBsd,
    Lites,
}
impl CreatorOS {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => CreatorOS::Linux,
            1 => CreatorOS::Hurd,
            2 => CreatorOS::Masix,
            3 => CreatorOS::FreeBsd,
            4 => CreatorOS::Lites,
            _ => panic!("Invalid creator OS value"),
        }
    }
}

#[derive(Debug)]
#[repr(u32)]
pub enum OptionalFeatures {
    DirectoryPrealloc,
    IMagicInodes,
    HasJournal,
    ExtendedAttributes,
    ResizeInode,
    DirectoryIndex,
}
impl OptionalFeatures {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0x1 => OptionalFeatures::DirectoryPrealloc,
            0x2 => OptionalFeatures::IMagicInodes,
            0x4 => OptionalFeatures::HasJournal,
            0x8 => OptionalFeatures::ExtendedAttributes,
            0x10 => OptionalFeatures::ResizeInode,
            0x20 => OptionalFeatures::DirectoryIndex,
            _ => panic!("Invalid optional features value {}", value),
        }
    }
}

#[derive(Debug)]
#[repr(u32)]
pub enum RequiredFeatures {
    Compression,
    FileType,
    Recover,
    JournalDev,
    MetaBg,
}
impl RequiredFeatures {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0x1 => RequiredFeatures::Compression,
            0x2 => RequiredFeatures::FileType,
            0x4 => RequiredFeatures::Recover,
            0x8 => RequiredFeatures::JournalDev,
            0x10 => RequiredFeatures::MetaBg,
            _ => panic!("Invalid required features value {}", value),
        }
    }
}

#[derive(Debug)]
#[repr(u32)]
pub enum ReadOnlyFeatures {
    SparseSuper,
    LargeFile,
    BTreeDir,
}
impl ReadOnlyFeatures {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0x1 => ReadOnlyFeatures::SparseSuper,
            0x2 => ReadOnlyFeatures::LargeFile,
            0x4 => ReadOnlyFeatures::BTreeDir,
            _ => panic!("Invalid read-only features value {}", value),
        }
    }
}

#[derive(Debug)]
pub struct Feature(u32);
impl Feature {
    pub fn new(value: u32) -> Self {
        Feature(value)
    }
    pub fn has_optional_feature(&self, feature: OptionalFeatures) -> bool {
        self.0 & feature as u32 != 0
    }
    pub fn has_required_feature(&self, feature: RequiredFeatures) -> bool {
        self.0 & feature as u32 != 0
    }
    pub fn has_read_only_feature(&self, feature: ReadOnlyFeatures) -> bool {
        self.0 & feature as u32 != 0
    }
}

#[derive(Debug)]
#[repr(u32)]
pub enum CompressionAlg {
    NoComp,
    LZV1,
    LZRW3A,
    GZIP,
    BZIP2,
    LZO,
}
impl CompressionAlg {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0x0 => CompressionAlg::NoComp,
            0x1 => CompressionAlg::LZV1,
            0x2 => CompressionAlg::LZRW3A,
            0x4 => CompressionAlg::GZIP,
            0x8 => CompressionAlg::BZIP2,
            0x10 => CompressionAlg::LZO,
            _ => panic!("Invalid compression algorithm value {}", value),
        }
    }
}
