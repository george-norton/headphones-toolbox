use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::filters::{
    AllpassFilter, BandpassPeakFilter, BandpassSkirtFilter, CustomIIRFilter, FilterConfig,
    FilterName, FreqGainQualFilter, FreqQualFilter, HighShelfFilter, HighpassFilter,
    LowShelfFilter, LowpassFilter, NotchFilter, PeakingFilter,
};

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

pub trait Discriminant {
    fn discriminant() -> u8;
}

impl<T: LowLevelFilter> Discriminant for T {
    fn discriminant() -> u8 {
        T::filter_type() as u8
    }
}

pub trait DeserializeFilter: Sized {
    fn from_reader(cur: impl Read) -> Result<Self, String>;
}

trait LowLevelFilter {
    fn filter_type() -> FilterType;
}

impl LowLevelFilter for LowpassFilter {
    fn filter_type() -> FilterType {
        FilterType::Lowpass
    }
}

impl LowLevelFilter for HighpassFilter {
    fn filter_type() -> FilterType {
        FilterType::Highpass
    }
}

impl LowLevelFilter for BandpassSkirtFilter {
    fn filter_type() -> FilterType {
        FilterType::BandpassSkirt
    }
}

impl LowLevelFilter for BandpassPeakFilter {
    fn filter_type() -> FilterType {
        FilterType::BandpassPeak
    }
}

impl LowLevelFilter for NotchFilter {
    fn filter_type() -> FilterType {
        FilterType::Notch
    }
}

impl LowLevelFilter for AllpassFilter {
    fn filter_type() -> FilterType {
        FilterType::Allpass
    }
}

impl LowLevelFilter for PeakingFilter {
    fn filter_type() -> FilterType {
        FilterType::Peaking
    }
}

impl LowLevelFilter for LowShelfFilter {
    fn filter_type() -> FilterType {
        FilterType::LowShelf
    }
}

impl LowLevelFilter for HighShelfFilter {
    fn filter_type() -> FilterType {
        FilterType::HighShelf
    }
}

impl LowLevelFilter for CustomIIRFilter {
    fn filter_type() -> FilterType {
        FilterType::CustomIIR
    }
}

pub fn read_filter(mut cur: impl Read + Seek) -> Result<FilterConfig, String> {
    let filter_type = cur.read_u8().unwrap();
    let _ = cur.seek(SeekFrom::Current(3)); // reserved bytes

    let filter: FilterConfig = match filter_type {
        x if x == LowpassFilter::discriminant() => LowpassFilter::from_reader(cur)?.into(),
        x if x == HighpassFilter::discriminant() => HighpassFilter::from_reader(cur)?.into(),
        x if x == BandpassSkirtFilter::discriminant() => {
            BandpassSkirtFilter::from_reader(cur)?.into()
        }
        x if x == BandpassPeakFilter::discriminant() => {
            BandpassPeakFilter::from_reader(cur)?.into()
        }
        x if x == NotchFilter::discriminant() => NotchFilter::from_reader(cur)?.into(),
        x if x == AllpassFilter::discriminant() => AllpassFilter::from_reader(cur)?.into(),
        x if x == PeakingFilter::discriminant() => PeakingFilter::from_reader(cur)?.into(),
        x if x == LowShelfFilter::discriminant() => LowShelfFilter::from_reader(cur)?.into(),
        x if x == HighShelfFilter::discriminant() => HighShelfFilter::from_reader(cur)?.into(),
        x if x == CustomIIRFilter::discriminant() => CustomIIRFilter::from_reader(cur)?.into(),
        other => {
            return Err(format!("Unknown filter type: {}", other));
        }
    };

    Ok(filter)
}

pub trait Payload: StructuralPayload {
    fn to_payload(&self) -> Vec<u8>;
}

impl<T: StructuralPayload + LowLevelFilter> Payload for T {
    fn to_payload(&self) -> Vec<u8> {
        let mut filter_payload = Vec::new();
        filter_payload.push(Self::discriminant());
        filter_payload.extend_from_slice(&[0u8; 3]);
        filter_payload.extend_from_slice(&self.to_structural_payload());
        filter_payload
    }
}

pub trait StructuralPayload {
    fn to_structural_payload(&self) -> Vec<u8>;
}

impl<T: FilterName> StructuralPayload for FreqQualFilter<T> {
    fn to_structural_payload(&self) -> Vec<u8> {
        [self.f0.to_le_bytes(), self.q.to_le_bytes()]
            .into_iter()
            .flatten()
            .collect()
    }
}

impl<T: FilterName> StructuralPayload for FreqGainQualFilter<T> {
    fn to_structural_payload(&self) -> Vec<u8> {
        [
            self.f0.to_le_bytes(),
            self.db_gain.to_le_bytes(),
            self.q.to_le_bytes(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl StructuralPayload for CustomIIRFilter {
    fn to_structural_payload(&self) -> Vec<u8> {
        [
            self.a0.to_le_bytes(),
            self.a1.to_le_bytes(),
            self.a2.to_le_bytes(),
            self.b0.to_le_bytes(),
            self.b1.to_le_bytes(),
            self.b2.to_le_bytes(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl DeserializeFilter for CustomIIRFilter {
    fn from_reader(mut cur: impl Read) -> Result<Self, String> {
        let a0 = cur.read_f64::<LittleEndian>().unwrap();
        let a1 = cur.read_f64::<LittleEndian>().unwrap();
        let a2 = cur.read_f64::<LittleEndian>().unwrap();
        let b0 = cur.read_f64::<LittleEndian>().unwrap();
        let b1 = cur.read_f64::<LittleEndian>().unwrap();
        let b2 = cur.read_f64::<LittleEndian>().unwrap();
        Ok(Self::new(a0, a1, a2, b0, b1, b2))
    }
}
