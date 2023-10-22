

use std::io::Write;

use crate::{Preprocessing, model::Filters, Codec};


pub trait Command {
    fn as_buf(&self) -> Vec<u8>;
}

#[repr(u16)]
#[allow(dead_code)]
pub enum StructureTypes {
    // Commands/Responses, these are container TLVs. The Value will be a set of TLV structures.
    OK = 0,      // Standard response when a command was successful
    NOK,         // Standard error response
    FlashHeader, // A special container for the config stored in flash. Hopefully there is some useful
    // metadata in here to allow us to migrate an old config to a new version.
    GetVersion, // Returns the current config version, and the minimum supported version so clients
    // can decide if they can talk to us or not.
    SetConfiguration, // Updates the active configuration with the supplied TLVs
    GetActiveConfiguration, // Retrieves the current active configuration TLVs from RAM
    GetStoredConfiguration, // Retrieves the current stored configuration TLVs from Flash
    SaveConfiguration, // Writes the active configuration to Flash
    FactoryReset,     // Invalidates the flash memory

    // Configuration structures, these are returned in the body of a command/response
    PreProcessingConfiguration = 0x200,
    FilterConfiguration,
    Pcm3060Configuration,

    // Status structures, these are returned in the body of a command/response but they are
    // not persisted as part of the configuration
    VersionStatus = 0x400,
}

pub struct GetVersion();

impl GetVersion {
    pub fn new() -> Self {
        Self()
    }
}

impl Command for GetVersion {
    fn as_buf(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(StructureTypes::GetVersion as u16).to_le_bytes());
        buf.extend_from_slice(&(4u16).to_le_bytes());
        buf
    }
}

pub struct SetPreprocessingConfiguration<'a>(&'a Preprocessing);

impl<'a> SetPreprocessingConfiguration<'a> {
    pub fn new(preprocessing: &'a Preprocessing) -> Self {
        Self(preprocessing)
    }
}

impl Command for SetPreprocessingConfiguration<'_> {
    fn as_buf(&self) -> Vec<u8> {
        let payload = self.0.to_payload();
        let mut buf = Vec::new();
        buf.extend_from_slice(&(StructureTypes::PreProcessingConfiguration as u16).to_le_bytes());
        buf.extend_from_slice(&((4 + payload.len()) as u16).to_le_bytes());
        buf.extend_from_slice(&payload);
        buf
    }
}

pub struct SetFilterConfiguration<'a>(&'a Filters);

impl<'a> SetFilterConfiguration<'a> {
    pub fn new(filters: &'a Filters) -> Self {
        Self(filters)
    }
}

impl Command for SetFilterConfiguration<'_> {
    fn as_buf(&self) -> Vec<u8> {
        let payload = self.0.to_payload();
        let mut buf = Vec::new();
        buf.extend_from_slice(&(StructureTypes::FilterConfiguration as u16).to_le_bytes());
        buf.extend_from_slice(&((4 + payload.len()) as u16).to_le_bytes());
        buf.extend_from_slice(&payload);
        buf
    }
}

pub struct SetPcm3060Configuration<'a>(&'a Codec);

impl<'a> SetPcm3060Configuration<'a> {
    pub fn new(codec: &'a Codec) -> Self {
        Self(codec)
    }
}

impl Command for SetPcm3060Configuration<'_> {
    fn as_buf(&self) -> Vec<u8> {
        let payload = self.0.to_payload();
        let mut buf = Vec::new();
        buf.extend_from_slice(&(StructureTypes::Pcm3060Configuration as u16).to_le_bytes());
        buf.extend_from_slice(&((4 + payload.len()) as u16).to_le_bytes());
        buf.extend_from_slice(&payload);
        buf
    }
}

pub struct SetConfiguration<'a, 'b, 'c>{
    preprocessing: SetPreprocessingConfiguration<'a>,
    filter: SetFilterConfiguration<'b>,
    codec: SetPcm3060Configuration<'c>
}

impl<'a, 'b, 'c> SetConfiguration<'a, 'b, 'c> {
    pub fn new(preprocessing: SetPreprocessingConfiguration<'a>, filter: SetFilterConfiguration<'b>, codec: SetPcm3060Configuration<'c>) -> Self {
        Self {
            preprocessing,
            filter,
            codec
        }
    }
}

impl Command for SetConfiguration<'_, '_, '_> {
    fn as_buf(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&(StructureTypes::SetConfiguration as u16).to_le_bytes());
        buf.extend_from_slice(
            &((16 + self.filter.0.to_payload().len() 
            + self.preprocessing.0.to_payload().len() 
            + self.codec.0.to_payload().len()) as u16)
                .to_le_bytes(),
        );
        buf.extend_from_slice(&self.preprocessing.as_buf());
        buf.extend_from_slice(&self.filter.as_buf());
        buf.extend_from_slice(&self.codec.as_buf());
        buf
    }
}

pub struct FactoryReset();

impl FactoryReset {
    pub fn new() -> Self {
        Self()
    }
}

impl Command for FactoryReset {
    fn as_buf(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&(StructureTypes::FactoryReset as u16).to_le_bytes());
        buf.extend_from_slice(&(4u16).to_le_bytes());
        buf
    }
}

pub struct SaveConfiguration();

impl SaveConfiguration {
    pub fn new() -> Self {
        Self()
    }
}

impl Command for SaveConfiguration {
    fn as_buf(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&(StructureTypes::SaveConfiguration as u16).to_le_bytes());
        buf.extend_from_slice(&(4u16).to_le_bytes());
        buf
    }
}

pub struct GetStoredConfiguration();

impl GetStoredConfiguration {
    pub fn new() -> Self {
        Self()
    }
}

impl Command for GetStoredConfiguration {
    fn as_buf(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&(StructureTypes::GetStoredConfiguration as u16).to_le_bytes());
        buf.extend_from_slice(&(4u16).to_le_bytes());
        buf
    }
}
