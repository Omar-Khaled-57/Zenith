use crate::hardware_access::{AccessError, HardwareAccess};
use crate::mock::{MSR_PACKAGE_THERM_STATUS, MSR_TEMPERATURE_TARGET, MSR_THERM_STATUS};
use crate::protocol::{Derived, Quality, SensorKind, SensorSample};

pub const THERM_VALID_BIT: u64 = 0x8000_0000;
pub const THERM_DELTA_MASK: u64 = 0x007F_0000;
pub const THERM_DELTA_SHIFT: u32 = 16;

pub const FALLBACK_TJMAX: u8 = 100;
const SUSPICIOUS_HIGH_C: f64 = 120.0;
const SUSPICIOUS_LOW_C: f64 = 0.0;

#[derive(Debug, Clone)]
pub struct IntelDecoder {
    pub tj_max: u8,
}

impl IntelDecoder {
    /// Reads the temperature target (MSR 0x1A2) per core; first valid value wins.
    /// Follows LHM: tjMax = (value >> 16) & 0xFF, fallback 100.
    pub fn detect_tjmax(
        access: &mut dyn HardwareAccess,
        core_first_logical: &[usize],
    ) -> Result<u8, AccessError> {
        for &cpu in core_first_logical {
            if let Ok(v) = access.read_msr(MSR_TEMPERATURE_TARGET, cpu) {
                let tj = ((v >> 16) & 0xFF) as u8;
                if tj > 0 {
                    return Ok(tj);
                }
            }
        }
        Ok(FALLBACK_TJMAX)
    }

    pub fn new(access: &mut dyn HardwareAccess, core_first_logical: &[usize]) -> Result<Self, AccessError> {
        Ok(Self {
            tj_max: Self::detect_tjmax(access, core_first_logical)?,
        })
    }

    /// Decodes the per-core MSR 0x19C value into a temperature (or None when invalid).
    pub fn decode_core_value(tj_max: u8, therm: u64) -> Option<f64> {
        if therm & THERM_VALID_BIT == 0 {
            return None;
        }
        let delta = ((therm & THERM_DELTA_MASK) >> THERM_DELTA_SHIFT) as f64;
        Some(tj_max as f64 - delta)
    }

    /// Reads all cores + package once. Returns sensors (package first, then cores)
    /// and the derived hottest/coolest/delta over core samples.
    pub fn sample(
        &mut self,
        access: &mut dyn HardwareAccess,
        core_first_logical: &[usize],
    ) -> Result<(Vec<SensorSample>, Derived), AccessError> {
        let mut sensors = Vec::with_capacity(core_first_logical.len() + 1);
        let mut core_temps: Vec<(usize, f64)> = Vec::new();

        for (i, &cpu) in core_first_logical.iter().enumerate() {
            let therm = access.read_msr(MSR_THERM_STATUS, cpu)?;
            match Self::decode_core_value(self.tj_max, therm) {
                Some(t) => {
                    core_temps.push((i, t));
                    sensors.push(SensorSample {
                        kind: SensorKind::Core,
                        index: i,
                        name: format!("Core #{}", i + 1),
                        value_c: t,
                        quality: quality_for(t),
                    });
                }
                None => sensors.push(SensorSample {
                    kind: SensorKind::Core,
                    index: i,
                    name: format!("Core #{}", i + 1),
                    value_c: f64::NAN,
                    quality: Quality::Invalid,
                }),
            }
        }

        let package = match access.read_msr(MSR_PACKAGE_THERM_STATUS, core_first_logical.first().copied().unwrap_or(0))? {
            v if v & THERM_VALID_BIT != 0 => {
                let delta = ((v & THERM_DELTA_MASK) >> THERM_DELTA_SHIFT) as f64;
                Some(self.tj_max as f64 - delta)
            }
            _ => None,
        };

        if let Some(p) = package {
            sensors.insert(
                0,
                SensorSample {
                    kind: SensorKind::Package,
                    index: 0,
                    name: "CPU Package".into(),
                    value_c: p,
                    quality: quality_for(p),
                },
            );
        }

        let (hottest_core_c, coolest_core_c, core_delta_c) = if core_temps.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let hottest = core_temps
                .iter()
                .map(|(_, t)| *t)
                .fold(f64::MIN, f64::max);
            let coolest = core_temps
                .iter()
                .map(|(_, t)| *t)
                .fold(f64::MAX, f64::min);
            (hottest, coolest, hottest - coolest)
        };

        Ok((
            sensors,
            Derived {
                hottest_core_c,
                coolest_core_c,
                core_delta_c,
            },
        ))
    }
}

pub fn quality_for(temp: f64) -> Quality {
    if !temp.is_finite() {
        return Quality::Invalid;
    }
    if !(SUSPICIOUS_LOW_C..=SUSPICIOUS_HIGH_C).contains(&temp) {
        return Quality::Suspicious;
    }
    Quality::Valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware_access::HardwareAccess;
    use crate::mock::MockHardware;

    fn mock_8_threads() -> MockHardware {
        // 4 physical cores -> 8 logicals; SMT pairs share core deltas.
        MockHardware::new(100, vec![20, 20, 22, 22, 25, 25, 30, 30], 26)
    }

    #[test]
    fn tjmax_from_msr() {
        let mut m = mock_8_threads();
        m.open().unwrap();
        assert_eq!(IntelDecoder::detect_tjmax(&mut m, &[0, 2, 4, 6]).unwrap(), 100);
    }

    #[test]
    fn tjmax_fallback_when_no_readable() {
        let mut m = MockHardware::new(0, vec![0], 0).unavailable();
        assert_eq!(IntelDecoder::detect_tjmax(&mut m, &[0]).unwrap(), FALLBACK_TJMAX);
    }

    #[test]
    fn decode_core_value() {
        // valid bit set, deltaT = 0x25 = 37
        let v = 0x8000_0000 | (37u64 << 16);
        assert_eq!(IntelDecoder::decode_core_value(100, v), Some(63.0));
        // validity bit clear -> None
        assert_eq!(IntelDecoder::decode_core_value(100, 37u64 << 16), None);
        // deltaT larger than tjmax -> negative (suspicious at quality layer)
        assert_eq!(IntelDecoder::decode_core_value(90, 0x8000_0000 | (100u64 << 16)), Some(-10.0));
    }

    #[test]
    fn sample_smoke() {
        let mut m = mock_8_threads();
        m.open().unwrap();
        let mut dec = IntelDecoder::new(&mut m, &[0, 2, 4, 6]).unwrap();
        let (sensors, derived) = dec.sample(&mut m, &[0, 2, 4, 6]).unwrap();

        assert_eq!(sensors.len(), 5); // package + 4 cores
        assert_eq!(sensors[0].kind, SensorKind::Package);
        assert_eq!(sensors[0].value_c, 74.0); // 100 - 26
        assert_eq!(sensors[1].value_c, 80.0); // 100 - 20
        assert_eq!(sensors[4].value_c, 70.0); // 100 - 30

        // hottest 80 (core0), coolest 70 (core3), delta 10
        assert_eq!(derived.hottest_core_c, 80.0);
        assert_eq!(derived.coolest_core_c, 70.0);
        assert_eq!(derived.core_delta_c, 10.0);
    }

    #[test]
    fn invalid_therm_bit_produces_invalid_quality() {
        assert_eq!(quality_for(f64::NAN), Quality::Invalid);
        assert_eq!(quality_for(130.0), Quality::Suspicious);
        assert_eq!(quality_for(-5.0), Quality::Suspicious);
        assert_eq!(quality_for(47.25), Quality::Valid);
    }
}
