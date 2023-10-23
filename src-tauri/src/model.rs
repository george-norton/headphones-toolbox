use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

trait ReadFilter: Sized {
    fn from_reader(cur: impl Read) -> Self;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FreqGainQualFilter {
    enabled: bool,
    f0: f32,
    q: f32,
    db_gain: f32,
}

impl ReadFilter for FreqGainQualFilter {
    fn from_reader(mut cur: impl Read) -> Self {
        let f0 = cur.read_f32::<LittleEndian>().unwrap();
        let db_gain = cur.read_f32::<LittleEndian>().unwrap();
        let q = cur.read_f32::<LittleEndian>().unwrap();
        Self {
            enabled: true,
            f0,
            q,
            db_gain,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FreqQualFilter {
    enabled: bool,
    f0: f32,
    q: f32,
}

impl ReadFilter for FreqQualFilter {
    fn from_reader(mut cur: impl Read) -> Self {
        let f0 = cur.read_f32::<LittleEndian>().unwrap();
        let q = cur.read_f32::<LittleEndian>().unwrap();
        Self {
            enabled: true,
            f0,
            q,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomIIRFilter {
    enabled: bool,
    a0: f64,
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
}

impl ReadFilter for CustomIIRFilter {
    fn from_reader(mut cur: impl Read) -> Self {
        let a0 = cur.read_f64::<LittleEndian>().unwrap();
        let a1 = cur.read_f64::<LittleEndian>().unwrap();
        let a2 = cur.read_f64::<LittleEndian>().unwrap();
        let b0 = cur.read_f64::<LittleEndian>().unwrap();
        let b1 = cur.read_f64::<LittleEndian>().unwrap();
        let b2 = cur.read_f64::<LittleEndian>().unwrap();
        Self {
            enabled: true,
            a0,
            a1,
            a2,
            b0,
            b1,
            b2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FilterType {
    Lowpass = 0,
    Highpass,
    BandpassSkirt,
    BandpassPeak,
    Notch,
    Allpass,
    Peaking,
    LowShelf,
    HighShelf,
    CustomIIR,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "filter_type")]
pub enum Filter {
    Lowpass(FreqQualFilter),
    Highpass(FreqQualFilter),
    BandpassSkirt(FreqQualFilter),
    BandpassPeak(FreqQualFilter),
    Notch(FreqQualFilter),
    Allpass(FreqQualFilter),
    Peaking(FreqGainQualFilter),
    LowShelf(FreqGainQualFilter),
    HighShelf(FreqGainQualFilter),
    CustomIIR(CustomIIRFilter),
}

impl Filter {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Lowpass(x)
            | Self::Highpass(x)
            | Self::BandpassPeak(x)
            | Self::BandpassSkirt(x)
            | Self::Notch(x)
            | Self::Allpass(x) => x.enabled,
            Self::Peaking(x) | Self::LowShelf(x) | Self::HighShelf(x) => x.enabled,
            Self::CustomIIR(x) => x.enabled,
        }
    }

    pub fn discriminant(&self) -> u8 {
        match self {
            Self::Lowpass(_) => FilterType::Lowpass as u8,
            Self::Highpass(_) => FilterType::Highpass as u8,
            Self::BandpassSkirt(_) => FilterType::BandpassSkirt as u8,
            Self::BandpassPeak(_) => FilterType::BandpassPeak as u8,
            Self::Notch(_) => FilterType::Notch as u8,
            Self::Allpass(_) => FilterType::Allpass as u8,
            Self::Peaking(_) => FilterType::Peaking as u8,
            Self::LowShelf(_) => FilterType::LowShelf as u8,
            Self::HighShelf(_) => FilterType::HighShelf as u8,
            Self::CustomIIR(_) => FilterType::CustomIIR as u8,
        }
    }

    pub fn payload(&self) -> Result<Vec<u8>, String> {
        let mut filter_payload = Vec::new();
        filter_payload.push(self.discriminant());
        filter_payload.extend_from_slice(&[0u8; 3]);

        match self {
            Self::Lowpass(x)
            | Self::Highpass(x)
            | Self::BandpassPeak(x)
            | Self::BandpassSkirt(x)
            | Self::Notch(x)
            | Self::Allpass(x) => {
                if x.q <= 0.0 {
                    return Err("Quality shall not be lower than 0.".to_owned());
                }
                filter_payload.extend_from_slice(&x.f0.to_le_bytes());
                filter_payload.extend_from_slice(&x.q.to_le_bytes());
            }
            Self::Peaking(x) | Self::LowShelf(x) | Self::HighShelf(x) => {
                if x.q <= 0.0 {
                    return Err("Quality shall not be lower than 0.".to_owned());
                }
                filter_payload.extend_from_slice(&x.f0.to_le_bytes());
                filter_payload.extend_from_slice(&x.db_gain.to_le_bytes());
                filter_payload.extend_from_slice(&x.q.to_le_bytes());
            }
            Self::CustomIIR(x) => {
                filter_payload.extend_from_slice(&x.a0.to_le_bytes());
                filter_payload.extend_from_slice(&x.a1.to_le_bytes());
                filter_payload.extend_from_slice(&x.a2.to_le_bytes());
                filter_payload.extend_from_slice(&x.b0.to_le_bytes());
                filter_payload.extend_from_slice(&x.b1.to_le_bytes());
                filter_payload.extend_from_slice(&x.b2.to_le_bytes());
            }
        }
        Ok(filter_payload)
    }

    pub fn from_bytes(mut cur: impl Read + Seek) -> Result<Self, String> {
        let filter_type = cur.read_u8().unwrap();
        let _ = cur.seek(SeekFrom::Current(3)); // reserved bytes

        let filter = match filter_type {
            x if x == FilterType::Lowpass as u8 => Self::Lowpass(ReadFilter::from_reader(cur)),
            x if x == FilterType::Highpass as u8 => Self::Highpass(ReadFilter::from_reader(cur)),
            x if x == FilterType::BandpassSkirt as u8 => {
                Self::BandpassSkirt(ReadFilter::from_reader(cur))
            }
            x if x == FilterType::BandpassPeak as u8 => {
                Self::BandpassPeak(ReadFilter::from_reader(cur))
            }
            x if x == FilterType::Notch as u8 => Self::Notch(ReadFilter::from_reader(cur)),
            x if x == FilterType::Allpass as u8 => Self::Allpass(ReadFilter::from_reader(cur)),
            x if x == FilterType::Peaking as u8 => Self::Peaking(ReadFilter::from_reader(cur)),
            x if x == FilterType::LowShelf as u8 => Self::LowShelf(ReadFilter::from_reader(cur)),
            x if x == FilterType::HighShelf as u8 => Self::HighShelf(ReadFilter::from_reader(cur)),
            x if x == FilterType::CustomIIR as u8 => Self::CustomIIR(ReadFilter::from_reader(cur)),
            other => {
                return Err(format!("Unknown filter type: {}", other));
            }
        };

        debug_assert!(
            filter.discriminant() == filter_type,
            "Filter discriminant is wrong: is {}, expected {}",
            filter.discriminant(),
            filter_type
        );

        Ok(filter)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Filters(Vec<Filter>);

impl Filters {
    pub fn to_payload(&self) -> Result<Vec<u8>, String> {
        self.0
            .iter()
            .filter(|f| f.enabled())
            .map(|f| f.payload())
            .collect::<Result<Vec<_>, String>>()
            .map(|v| v.into_iter().flatten().collect())
    }

    pub fn add(&mut self, filter: Filter) {
        self.0.push(filter);
    }
}
