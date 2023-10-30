use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::{
    filters::{
        Allpass, BandpassPeak, BandpassSkirt, CustomIIRFilter, FilterConfig, FilterName, Filters,
        FreqGainQualFilter, FreqQualFilter, HighShelf, Highpass, LowShelf, Lowpass, Notch, Peaking,
        SavedFilter,
    },
    Codec, Preprocessing, Config,
};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ConfigDTO {
    pub preprocessing: Preprocessing,
    pub filters: FiltersDTO,
    pub codec: Codec,
}

impl TryInto<Config> for ConfigDTO {
    type Error = String;
    fn try_into(self) -> Result<Config, Self::Error> {
        Ok(Config::new(self.preprocessing, self.filters.try_into()?, self.codec))
    }
}

impl From<Config> for ConfigDTO {
    fn from(value: Config) -> Self {
        Self { preprocessing: value.preprocessing, filters: value.filters.into(), codec: value.codec }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct FiltersDTO(Vec<SavedFilterDTO>);

impl FiltersDTO {
    pub fn add(&mut self, filter: FilterConfigDTO) {
        self.0.push(SavedFilterDTO { enabled: true, filter: filter })
    }
}

impl TryInto<Filters> for FiltersDTO {
    type Error = String;
    fn try_into(self) -> Result<Filters, Self::Error> {
        self.0.into_iter().map(|f| f.try_into()).collect()
    }
}

impl From<Filters> for FiltersDTO {
    fn from(value: Filters) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<SavedFilter> for FiltersDTO {
    fn from_iter<T: IntoIterator<Item = SavedFilter>>(iter: T) -> Self {
        iter.into_iter().map(|f| f.into()).collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SavedFilterDTO {
    enabled: bool,

    #[serde(flatten)]
    filter: FilterConfigDTO,
}

impl TryInto<SavedFilter> for SavedFilterDTO {
    type Error = String;
    fn try_into(self) -> Result<SavedFilter, Self::Error> {
        Ok(SavedFilter::new(self.enabled, self.filter.try_into()?))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "filter_type")]
pub enum FilterConfigDTO {
    Lowpass(LowpassFilterDTO),
    Highpass(HighpassFilterDTO),
    BandpassSkirt(BandpassSkirtFilterDTO),
    BandpassPeak(BandpassPeakFilterDTO),
    Notch(NotchFilterDTO),
    Allpass(AllpassFilterDTO),
    Peaking(PeakingFilterDTO),
    LowShelf(LowShelfFilterDTO),
    HighShelf(HighShelfFilterDTO),
    CustomIIR(CustomIIRFilter),
}

pub type LowpassFilterDTO = FreqQualFilterDTO<Lowpass>;
pub type HighpassFilterDTO = FreqQualFilterDTO<Highpass>;
pub type BandpassSkirtFilterDTO = FreqQualFilterDTO<BandpassSkirt>;
pub type BandpassPeakFilterDTO = FreqQualFilterDTO<BandpassPeak>;
pub type NotchFilterDTO = FreqQualFilterDTO<Notch>;
pub type AllpassFilterDTO = FreqQualFilterDTO<Allpass>;
pub type PeakingFilterDTO = FreqGainQualFilterDTO<Peaking>;
pub type LowShelfFilterDTO = FreqGainQualFilterDTO<LowShelf>;
pub type HighShelfFilterDTO = FreqGainQualFilterDTO<HighShelf>;

impl TryInto<FilterConfig> for FilterConfigDTO {
    type Error = String;
    fn try_into(self) -> Result<FilterConfig, Self::Error> {
        match self {
            FilterConfigDTO::Lowpass(x) => Ok(FilterConfig::Lowpass(x.try_into()?)),
            FilterConfigDTO::Highpass(x) => Ok(FilterConfig::Highpass(x.try_into()?)),
            FilterConfigDTO::BandpassSkirt(x) => Ok(FilterConfig::BandpassSkirt(x.try_into()?)),
            FilterConfigDTO::BandpassPeak(x) => Ok(FilterConfig::BandpassPeak(x.try_into()?)),
            FilterConfigDTO::Notch(x) => Ok(FilterConfig::Notch(x.try_into()?)),
            FilterConfigDTO::Allpass(x) => Ok(FilterConfig::Allpass(x.try_into()?)),
            FilterConfigDTO::Peaking(x) => Ok(FilterConfig::Peaking(x.try_into()?)),
            FilterConfigDTO::LowShelf(x) => Ok(FilterConfig::LowShelf(x.try_into()?)),
            FilterConfigDTO::HighShelf(x) => Ok(FilterConfig::HighShelf(x.try_into()?)),
            FilterConfigDTO::CustomIIR(x) => Ok(FilterConfig::CustomIIR(x)),
        }
    }
}

impl From<FilterConfig> for FilterConfigDTO {
    fn from(value: FilterConfig) -> Self {
        match value {
            FilterConfig::Lowpass(x) => FilterConfigDTO::Lowpass(x.into()),
            FilterConfig::Highpass(x) => FilterConfigDTO::Highpass(x.into()),
            FilterConfig::BandpassSkirt(x) => FilterConfigDTO::BandpassSkirt(x.into()),
            FilterConfig::BandpassPeak(x) => FilterConfigDTO::BandpassPeak(x.into()),
            FilterConfig::Notch(x) => FilterConfigDTO::Notch(x.into()),
            FilterConfig::Allpass(x) => FilterConfigDTO::Allpass(x.into()),
            FilterConfig::Peaking(x) => FilterConfigDTO::Peaking(x.into()),
            FilterConfig::LowShelf(x) => FilterConfigDTO::LowShelf(x.into()),
            FilterConfig::HighShelf(x) => FilterConfigDTO::HighShelf(x.into()),
            FilterConfig::CustomIIR(x) => FilterConfigDTO::CustomIIR(x.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FreqQualFilterDTO<T: FilterName> {
    _type: PhantomData<T>,
    pub f0: f32,
    pub q: f32,
}

impl<T: FilterName> TryInto<FreqQualFilter<T>> for FreqQualFilterDTO<T> {
    type Error = String;
    fn try_into(self) -> Result<FreqQualFilter<T>, Self::Error> {
        FreqQualFilter::new(self.f0, self.q)
    }
}

impl<T: FilterName> From<FreqQualFilter<T>> for FreqQualFilterDTO<T> {
    fn from(value: FreqQualFilter<T>) -> Self {
        Self { _type: PhantomData, f0: value.f0, q: value.q }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FreqGainQualFilterDTO<T: FilterName> {
    _type: PhantomData<T>,
    pub f0: f32,
    pub q: f32,
    pub db_gain: f32,
}

impl<T: FilterName> TryInto<FreqGainQualFilter<T>> for FreqGainQualFilterDTO<T> {
    type Error = String;
    fn try_into(self) -> Result<FreqGainQualFilter<T>, Self::Error> {
        FreqGainQualFilter::new(self.f0, self.q, self.db_gain)
    }
}

impl<T: FilterName> From<FreqGainQualFilter<T>> for FreqGainQualFilterDTO<T> {
    fn from(value: FreqGainQualFilter<T>) -> Self {
        Self { _type: PhantomData, f0: value.f0, q: value.q, db_gain: value.db_gain }
    }
}
