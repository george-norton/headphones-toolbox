use std::{f64::consts::PI, io::Read, marker::PhantomData};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::low_level::{DeserializeFilter, Payload};

const FS: f64 = 48000.0;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FreqGainQualFilter<T: FilterName> {
    #[serde(skip, default)]
    _type: PhantomData<T>,
    pub f0: f32,
    pub q: f32,
    pub db_gain: f32,
}

impl<T: FilterName> FreqGainQualFilter<T> {
    pub fn new(f0: f32, q: f32, db_gain: f32) -> Result<Self, String> {
        if q <= 0.0 {
            return Err("Quality shall not be lower than 0.".to_owned());
        }

        Ok(Self {
            _type: PhantomData,
            f0,
            q,
            db_gain,
        })
    }
}

impl<T: FilterName> DeserializeFilter for FreqGainQualFilter<T> {
    fn from_reader(mut cur: impl Read) -> Result<Self, String> {
        let f0 = cur.read_f32::<LittleEndian>().unwrap();
        let db_gain = cur.read_f32::<LittleEndian>().unwrap();
        let q = cur.read_f32::<LittleEndian>().unwrap();
        Self::new(f0, q, db_gain)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FreqQualFilter<T: FilterName> {
    #[serde(skip, default)]
    _type: PhantomData<T>,
    pub f0: f32,
    pub q: f32,
}

impl<T: FilterName> FreqQualFilter<T> {
    pub fn new(f0: f32, q: f32) -> Result<Self, String> {
        if q <= 0.0 {
            return Err("Quality shall not be lower than 0.".to_owned());
        }

        Ok(Self {
            _type: PhantomData,
            f0,
            q,
        })
    }
}

impl<T: FilterName> DeserializeFilter for FreqQualFilter<T> {
    fn from_reader(mut cur: impl Read) -> Result<Self, String> {
        let f0 = cur.read_f32::<LittleEndian>().unwrap();
        let q = cur.read_f32::<LittleEndian>().unwrap();
        Self::new(f0, q)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomIIRFilter {
    pub a0: f64,
    pub a1: f64,
    pub a2: f64,
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
}

impl CustomIIRFilter {
    pub fn new(a0: f64, a1: f64, a2: f64, b0: f64, b1: f64, b2: f64) -> Self {
        Self {
            a0,
            a1,
            a2,
            b0,
            b1,
            b2,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "filter_type")]
pub enum FilterConfig {
    Lowpass(LowpassFilter),
    Highpass(HighpassFilter),
    BandpassSkirt(BandpassSkirtFilter),
    BandpassPeak(BandpassPeakFilter),
    Notch(NotchFilter),
    Allpass(AllpassFilter),
    Peaking(PeakingFilter),
    LowShelf(LowShelfFilter),
    HighShelf(HighShelfFilter),
    CustomIIR(CustomIIRFilter),
}

pub type LowpassFilter = FreqQualFilter<Lowpass>;
pub type HighpassFilter = FreqQualFilter<Highpass>;
pub type BandpassSkirtFilter = FreqQualFilter<BandpassSkirt>;
pub type BandpassPeakFilter = FreqQualFilter<BandpassPeak>;
pub type NotchFilter = FreqQualFilter<Notch>;
pub type AllpassFilter = FreqQualFilter<Allpass>;
pub type PeakingFilter = FreqGainQualFilter<Peaking>;
pub type LowShelfFilter = FreqGainQualFilter<LowShelf>;
pub type HighShelfFilter = FreqGainQualFilter<HighShelf>;

pub trait Validate {
    fn validate(&self) -> Result<(), String>;
}

impl Validate for FilterConfig {
    fn validate(&self) -> Result<(), String> {
        match self {
            FilterConfig::Lowpass(x) => x.validate(),
            FilterConfig::Highpass(x) => x.validate(),
            FilterConfig::BandpassSkirt(x) => x.validate(),
            FilterConfig::BandpassPeak(x) => x.validate(),
            FilterConfig::Notch(x) => x.validate(),
            FilterConfig::Allpass(x) => x.validate(),
            FilterConfig::Peaking(x) => x.validate(),
            FilterConfig::LowShelf(x) => x.validate(),
            FilterConfig::HighShelf(x) => x.validate(),
            FilterConfig::CustomIIR(x) => x.validate(),
        }
    }
}

impl<T: FilterName> Validate for FreqQualFilter<T> {
    fn validate(&self) -> Result<(), String> {
        if self.q <= 0.0 {
            return Err("Quality can't be zero.".to_owned());
        }
        Ok(())
    }
}

impl<T: FilterName> Validate for FreqGainQualFilter<T> {
    fn validate(&self) -> Result<(), String> {
        if self.q <= 0.0 {
            return Err("Quality can't be zero.".to_owned());
        }
        Ok(())
    }
}

impl Validate for CustomIIRFilter {
    fn validate(&self) -> Result<(), String> {
        Ok(()) // TODO
    }
}

impl FilterConfig {
    #[allow(dead_code)]
    fn into_iir(self) -> CustomIIRFilter {
        match self {
            FilterConfig::Lowpass(x) => x.into(),
            FilterConfig::Highpass(x) => x.into(),
            FilterConfig::BandpassSkirt(x) => x.into(),
            FilterConfig::BandpassPeak(x) => x.into(),
            FilterConfig::Notch(x) => x.into(),
            FilterConfig::Allpass(x) => x.into(),
            FilterConfig::Peaking(x) => x.into(),
            FilterConfig::LowShelf(x) => x.into(),
            FilterConfig::HighShelf(x) => x.into(),
            FilterConfig::CustomIIR(x) => x.into(),
        }
    }

    fn payload(&self) -> Vec<u8> {
        match self {
            FilterConfig::Lowpass(x) => x.to_payload(),
            FilterConfig::Highpass(x) => x.to_payload(),
            FilterConfig::BandpassSkirt(x) => x.to_payload(),
            FilterConfig::BandpassPeak(x) => x.to_payload(),
            FilterConfig::Notch(x) => x.to_payload(),
            FilterConfig::Allpass(x) => x.to_payload(),
            FilterConfig::Peaking(x) => x.to_payload(),
            FilterConfig::LowShelf(x) => x.to_payload(),
            FilterConfig::HighShelf(x) => x.to_payload(),
            FilterConfig::CustomIIR(x) => x.to_payload(),
        }
    }
}

impl Into<FilterConfig> for LowpassFilter {
    fn into(self) -> FilterConfig {
        FilterConfig::Lowpass(self)
    }
}

impl Into<FilterConfig> for HighpassFilter {
    fn into(self) -> FilterConfig {
        FilterConfig::Highpass(self)
    }
}

impl Into<FilterConfig> for BandpassSkirtFilter {
    fn into(self) -> FilterConfig {
        FilterConfig::BandpassSkirt(self)
    }
}

impl Into<FilterConfig> for BandpassPeakFilter {
    fn into(self) -> FilterConfig {
        FilterConfig::BandpassPeak(self)
    }
}

impl Into<FilterConfig> for NotchFilter {
    fn into(self) -> FilterConfig {
        FilterConfig::Notch(self)
    }
}

impl Into<FilterConfig> for AllpassFilter {
    fn into(self) -> FilterConfig {
        FilterConfig::Allpass(self)
    }
}

impl Into<FilterConfig> for PeakingFilter {
    fn into(self) -> FilterConfig {
        FilterConfig::Peaking(self)
    }
}

impl Into<FilterConfig> for LowShelfFilter {
    fn into(self) -> FilterConfig {
        FilterConfig::LowShelf(self)
    }
}

impl Into<FilterConfig> for HighShelfFilter {
    fn into(self) -> FilterConfig {
        FilterConfig::HighShelf(self)
    }
}

impl Into<FilterConfig> for CustomIIRFilter {
    fn into(self) -> FilterConfig {
        FilterConfig::CustomIIR(self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedFilter {
    enabled: bool,

    #[serde(flatten)]
    filter: FilterConfig,
}

impl SavedFilter {
    pub fn new(enabled: bool, filter: FilterConfig) -> Self {
        Self { enabled, filter }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Filters(Vec<SavedFilter>);

impl Filters {
    pub fn to_payload(&self) -> Vec<u8> {
        self.0
            .iter()
            .filter(|f| f.enabled)
            .map(|f| f.filter.payload())
            .flatten()
            .collect()
    }

    pub fn add(&mut self, filter: FilterConfig, enabled: bool) {
        self.0.push(SavedFilter::new(enabled, filter));
    }
}

impl Validate for Filters {
    fn validate(&self) -> Result<(), String> {
        self.0.iter().map(|f| f.filter.validate()).collect()
    }
}

pub trait FilterName {}

#[derive(Debug, Clone)]
pub struct Lowpass();

impl FilterName for Lowpass {}

#[derive(Debug, Clone)]
pub struct Highpass();

impl FilterName for Highpass {}

#[derive(Debug, Clone)]
pub struct BandpassSkirt();

impl FilterName for BandpassSkirt {}

#[derive(Debug, Clone)]
pub struct BandpassPeak();

impl FilterName for BandpassPeak {}

#[derive(Debug, Clone)]
pub struct Notch();

impl FilterName for Notch {}

#[derive(Debug, Clone)]
pub struct Allpass();

impl FilterName for Allpass {}

#[derive(Debug, Clone)]
pub struct Peaking();

impl FilterName for Peaking {}

#[derive(Debug, Clone)]
pub struct LowShelf();

impl FilterName for LowShelf {}

#[derive(Debug, Clone)]
pub struct HighShelf();

impl FilterName for HighShelf {}

pub trait ToCustomIIR {
    fn to_custom(&self) -> CustomIIRFilter;
}

impl Into<CustomIIRFilter> for LowpassFilter {
    fn into(self) -> CustomIIRFilter {
        let w0: f64 = 2.0 * PI * (f64::from(self.f0)) / FS;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * f64::from(self.q));
        CustomIIRFilter {
            b0: (1.0 - cosw0) / 2.0,
            b1: 1.0 - cosw0,
            b2: (1.0 - cosw0) / 2.0,
            a0: 1.0 + alpha,
            a1: -2.0 * cosw0,
            a2: 1.0 - alpha,
        }
    }
}

impl Into<CustomIIRFilter> for HighpassFilter {
    fn into(self) -> CustomIIRFilter {
        let w0: f64 = 2.0 * PI * (f64::from(self.f0)) / FS;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * f64::from(self.q));
        CustomIIRFilter {
            b0: (1.0 + cosw0) / 2.0,
            b1: -(1.0 + cosw0),
            b2: (1.0 + cosw0) / 2.0,
            a0: 1.0 + alpha,
            a1: -2.0 * cosw0,
            a2: 1.0 - alpha,
        }
    }
}

impl Into<CustomIIRFilter> for BandpassSkirtFilter {
    fn into(self) -> CustomIIRFilter {
        let w0: f64 = 2.0 * PI * (f64::from(self.f0)) / FS;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * f64::from(self.q));
        CustomIIRFilter {
            b0: sinw0 / 2.0,
            b1: 0.0,
            b2: -sinw0 / 2.0,
            a0: 1.0 + alpha,
            a1: -2.0 * cosw0,
            a2: 1.0 - alpha,
        }
    }
}

impl Into<CustomIIRFilter> for BandpassPeakFilter {
    fn into(self) -> CustomIIRFilter {
        let w0: f64 = 2.0 * PI * (f64::from(self.f0)) / FS;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * f64::from(self.q));
        CustomIIRFilter {
            b0: alpha,
            b1: 0.0,
            b2: -alpha,
            a0: 1.0 + alpha,
            a1: -2.0 * cosw0,
            a2: 1.0 - alpha,
        }
    }
}

impl Into<CustomIIRFilter> for NotchFilter {
    fn into(self) -> CustomIIRFilter {
        let w0: f64 = 2.0 * PI * (f64::from(self.f0)) / FS;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * f64::from(self.q));
        CustomIIRFilter {
            b0: 1.0,
            b1: -2.0 * cosw0,
            b2: 1.0,
            a0: 1.0 + alpha,
            a1: -2.0 * cosw0,
            a2: 1.0 - alpha,
        }
    }
}

impl Into<CustomIIRFilter> for AllpassFilter {
    fn into(self) -> CustomIIRFilter {
        let w0: f64 = 2.0 * PI * (f64::from(self.f0)) / FS;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * f64::from(self.q));
        CustomIIRFilter {
            b0: 1.0 - alpha,
            b1: -2.0 * cosw0,
            b2: 1.0 + alpha,
            a0: 1.0 + alpha,
            a1: -2.0 * cosw0,
            a2: 1.0 - alpha,
        }
    }
}

impl Into<CustomIIRFilter> for PeakingFilter {
    fn into(self) -> CustomIIRFilter {
        let w0: f64 = 2.0 * PI * (f64::from(self.f0)) / FS;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * f64::from(self.q));

        let a = 10.0_f64.powf(f64::from(self.db_gain) / 40.0);
        CustomIIRFilter {
            b0: 1.0 + (alpha * a),
            b1: -2.0 * cosw0,
            b2: 1.0 - (alpha * a),
            a0: 1.0 + (alpha / a),
            a1: -2.0 * cosw0,
            a2: 1.0 - (alpha / a),
        }
    }
}

impl Into<CustomIIRFilter> for LowShelfFilter {
    fn into(self) -> CustomIIRFilter {
        let w0: f64 = 2.0 * PI * (f64::from(self.f0)) / FS;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * f64::from(self.q));

        let a = 10.0_f64.powf(f64::from(self.db_gain) / 40.0);
        let traa = 2.0 * a.sqrt() * alpha;
        CustomIIRFilter {
            b0: a * ((a + 1.0) - ((a - 1.0) * cosw0) + traa),
            b1: 2.0 * a * ((a - 1.0) - ((a + 1.0) * cosw0)),
            b2: a * ((a + 1.0) - ((a - 1.0) * cosw0) - traa),
            a0: (a + 1.0) + ((a - 1.0) * cosw0) + traa,
            a1: -2.0 * ((a - 1.0) + ((a + 1.0) * cosw0)),
            a2: (a + 1.0) + ((a - 1.0) * cosw0) - traa,
        }
    }
}

impl Into<CustomIIRFilter> for HighShelfFilter {
    fn into(self) -> CustomIIRFilter {
        let w0: f64 = 2.0 * PI * (f64::from(self.f0)) / FS;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * f64::from(self.q));

        let a = 10.0_f64.powf(f64::from(self.db_gain) / 40.0);
        let traa = 2.0 * a.sqrt() * alpha;
        CustomIIRFilter {
            b0: a * ((a + 1.0) + ((a - 1.0) * cosw0) + traa),
            b1: -2.0 * a * ((a - 1.0) + ((a + 1.0) * cosw0)),
            b2: a * ((a + 1.0) + ((a - 1.0) * cosw0) - traa),
            a0: (a + 1.0) - ((a - 1.0) * cosw0) + traa,
            a1: 2.0 * ((a - 1.0) - ((a + 1.0) * cosw0)),
            a2: (a + 1.0) - ((a - 1.0) * cosw0) - traa,
        }
    }
}
