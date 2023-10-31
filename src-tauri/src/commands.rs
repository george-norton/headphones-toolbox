use std::io::Write;

use crate::{model::Filters, Codec, Preprocessing};

pub trait Command {
    fn write_as_binary(&self, buf: impl Write) -> Result<(), String>;
}

#[repr(u16)]
#[allow(dead_code)]
pub enum StructureTypes {
    // Commands/Responses, these are container TLVs. The Value will be a set of TLV structures.
    Ok = 0,      // Standard response when a command was successful
    Nok,         // Standard error response
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
    fn write_as_binary(&self, mut buf: impl Write) -> Result<(), String> {
        let _ = buf.write(&(StructureTypes::GetVersion as u16).to_le_bytes());
        let _ = buf.write(&(4u16).to_le_bytes());
        Ok(())
    }
}

pub struct SetPreprocessingConfiguration<'a>(&'a Preprocessing);

impl<'a> SetPreprocessingConfiguration<'a> {
    pub fn new(preprocessing: &'a Preprocessing) -> Self {
        Self(preprocessing)
    }
}

impl Command for SetPreprocessingConfiguration<'_> {
    fn write_as_binary(&self, mut buf: impl Write) -> Result<(), String> {
        let payload = self.0.to_payload();
        let _ = buf.write(&(StructureTypes::PreProcessingConfiguration as u16).to_le_bytes());
        let _ = buf.write(&((4 + payload.len()) as u16).to_le_bytes());
        let _ = buf.write(&payload);
        Ok(())
    }
}

pub struct SetFilterConfiguration<'a>(&'a Filters);

impl<'a> SetFilterConfiguration<'a> {
    pub fn new(filters: &'a Filters) -> Self {
        Self(filters)
    }
}

impl Command for SetFilterConfiguration<'_> {
    fn write_as_binary(&self, mut buf: impl Write) -> Result<(), String> {
        let payload = self.0.to_payload()?;
        let _ = buf.write(&(StructureTypes::FilterConfiguration as u16).to_le_bytes());
        let _ = buf.write(&((4 + payload.len()) as u16).to_le_bytes());
        let _ = buf.write(&payload);
        Ok(())
    }
}

pub struct SetPcm3060Configuration<'a>(&'a Codec);

impl<'a> SetPcm3060Configuration<'a> {
    pub fn new(codec: &'a Codec) -> Self {
        Self(codec)
    }
}

impl Command for SetPcm3060Configuration<'_> {
    fn write_as_binary(&self, mut buf: impl Write) -> Result<(), String> {
        let payload = self.0.to_payload();
        let _ = buf.write(&(StructureTypes::Pcm3060Configuration as u16).to_le_bytes());
        let _ = buf.write(&((4 + payload.len()) as u16).to_le_bytes());
        let _ = buf.write(&payload);
        Ok(())
    }
}

pub struct SetConfiguration<'a, 'b, 'c> {
    preprocessing: SetPreprocessingConfiguration<'a>,
    filter: SetFilterConfiguration<'b>,
    codec: SetPcm3060Configuration<'c>,
}

impl<'a, 'b, 'c> SetConfiguration<'a, 'b, 'c> {
    pub fn new(
        preprocessing: SetPreprocessingConfiguration<'a>,
        filter: SetFilterConfiguration<'b>,
        codec: SetPcm3060Configuration<'c>,
    ) -> Self {
        Self {
            preprocessing,
            filter,
            codec,
        }
    }
}

impl Command for SetConfiguration<'_, '_, '_> {
    fn write_as_binary(&self, mut buf: impl Write) -> Result<(), String> {
        let _ = buf.write(&(StructureTypes::SetConfiguration as u16).to_le_bytes());
        let _ = buf.write(
            &((16
                + self.filter.0.to_payload()?.len()
                + self.preprocessing.0.to_payload().len()
                + self.codec.0.to_payload().len()) as u16)
                .to_le_bytes(),
        );
        let _ = &self.preprocessing.write_as_binary(&mut buf);
        let _ = &self.filter.write_as_binary(&mut buf);
        let _ = &self.codec.write_as_binary(&mut buf);
        Ok(())
    }
}

pub struct FactoryReset();

impl FactoryReset {
    pub fn new() -> Self {
        Self()
    }
}

impl Command for FactoryReset {
    fn write_as_binary(&self, mut buf: impl Write) -> Result<(), String> {
        let _ = buf.write(&(StructureTypes::FactoryReset as u16).to_le_bytes());
        let _ = buf.write(&(4u16).to_le_bytes());
        Ok(())
    }
}

pub struct SaveConfiguration();

impl SaveConfiguration {
    pub fn new() -> Self {
        Self()
    }
}

impl Command for SaveConfiguration {
    fn write_as_binary(&self, mut buf: impl Write) -> Result<(), String> {
        let _ = buf.write(&(StructureTypes::SaveConfiguration as u16).to_le_bytes());
        let _ = buf.write(&(4u16).to_le_bytes());
        Ok(())
    }
}

pub struct GetStoredConfiguration();

impl GetStoredConfiguration {
    pub fn new() -> Self {
        Self()
    }
}

impl Command for GetStoredConfiguration {
    fn write_as_binary(&self, mut buf: impl Write) -> Result<(), String> {
        let _ = buf.write(&(StructureTypes::GetStoredConfiguration as u16).to_le_bytes());
        let _ = buf.write(&(4u16).to_le_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_version_works() {
        let mut buf: Vec<u8> = Vec::new();
        GetVersion::new().write_as_binary(&mut buf).unwrap();
        assert!(buf.len() > 0, "Command didn't write anything");
        assert_eq!(buf.as_slice(), &[3, 0, 4, 0], "Wrong data");
    }

    #[test]
    fn preprocessing_works() {
        let mut buf = Vec::new();
        let config = Preprocessing::new(0.0, 0.0, false);
        SetPreprocessingConfiguration::new(&config).write_as_binary(&mut buf).unwrap();
        assert!(buf.len() > 0, "Command didn't write anything");
        assert_eq!(
            buf.as_slice(),
            &[0, 2, 16, 0, 0, 0, 128, 191, 0, 0, 128, 191, 0, 0, 0, 0],
            "Wrong data"
        );
    }

    #[test]
    fn filter_works() {
        let mut buf = Vec::new();
        let config = Filters::default();
        SetFilterConfiguration::new(&config).write_as_binary(&mut buf).unwrap();
        assert!(buf.len() > 0, "Command didn't write anything");
        assert_eq!(buf.as_slice(), &[1, 2, 4, 0], "Wrong data");
    }

    #[test]
    fn codec_works() {
        let mut buf = Vec::new();
        let config = Codec::default();
        SetPcm3060Configuration::new(&config).write_as_binary(&mut buf).unwrap();
        assert!(buf.len() > 0, "Command didn't write anything");
        assert_eq!(buf.as_slice(), &[2, 2, 8, 0, 0, 0, 0, 0], "Wrong data")
    }

    #[test]
    fn configuration_works() {
        let mut buf = Vec::new();
        let prep_config = Preprocessing::new(0.0, 0.0, false);
        let filters_config = Filters::default();
        let codec_config = Codec::default();

        let prep = SetPreprocessingConfiguration::new(&prep_config);
        let filters = SetFilterConfiguration::new(&filters_config);
        let codec = SetPcm3060Configuration::new(&codec_config);
        SetConfiguration::new(prep, filters, codec).write_as_binary(&mut buf).unwrap();
        assert!(buf.len() > 0, "Command didn't write anything");
        assert_eq!(
            buf.as_slice(),
            &[
                4, 0, 32, 0, 0, 2, 16, 0, 0, 0, 128, 191, 0, 0, 128, 191, 0, 0, 0, 0, 1, 2, 4, 0,
                2, 2, 8, 0, 0, 0, 0, 0
            ],
            "Wrong data"
        )
    }

    #[test]
    fn reset_works() {
        let mut buf = Vec::new();
        FactoryReset::new().write_as_binary(&mut buf).unwrap();
        assert!(buf.len() > 0, "Command didn't write anything");
        assert_eq!(buf.as_slice(), &[8, 0, 4, 0], "Wrong data")
    }

    #[test]
    fn save_config_works() {
        let mut buf = Vec::new();
        SaveConfiguration::new().write_as_binary(&mut buf).unwrap();
        assert!(buf.len() > 0, "Command didn't write anything");
        assert_eq!(buf.as_slice(), &[7, 0, 4, 0], "Wrong data")
    }

    #[test]
    fn get_config_works() {
        let mut buf = Vec::new();
        GetStoredConfiguration::new().write_as_binary(&mut buf).unwrap();
        assert!(buf.len() > 0, "Command didn't write anything");
        assert_eq!(buf.as_slice(), &[6, 0, 4, 0], "Wrong data")
    }
}
