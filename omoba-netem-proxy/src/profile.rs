use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{NetemError, Result};

pub const RTT_BUCKET_COUNT: usize = 20;
pub const RTT_MIN_MS: u32 = 20;
pub const RTT_MAX_MS: u32 = 100;
pub const RTT_BUCKET_WIDTH_MS: u32 = 4;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelayProfile {
    pub name: String,
    pub weights: [u64; RTT_BUCKET_COUNT],
    pub total_weight: u64,
}

impl DelayProfile {
    pub fn new(name: impl Into<String>, weights: Vec<u64>) -> Result<Self> {
        let weights: [u64; RTT_BUCKET_COUNT] = weights.try_into().map_err(|values: Vec<u64>| {
            NetemError::Config(format!(
                "profile must contain {RTT_BUCKET_COUNT} weights, got {}",
                values.len()
            ))
        })?;
        let total_weight = weights.iter().try_fold(0_u64, |sum, value| {
            sum.checked_add(*value)
                .ok_or_else(|| NetemError::Config("profile weight sum overflow".into()))
        })?;
        if total_weight == 0 {
            return Err(NetemError::Config(
                "profile weight sum must be greater than zero".into(),
            ));
        }
        Ok(Self {
            name: name.into(),
            weights,
            total_weight,
        })
    }

    pub fn load_custom(path: &Path) -> Result<Self> {
        #[derive(Deserialize)]
        struct Custom {
            weights: Vec<u64>,
        }
        let bytes = fs::read(path).map_err(|error| NetemError::Config(error.to_string()))?;
        let value: Custom = serde_json::from_slice(&bytes)
            .map_err(|error| NetemError::Config(error.to_string()))?;
        Self::new("custom-20-bin", value.weights)
    }

    pub fn named(name: &str) -> Result<Self> {
        let mut weights = vec![0_u64; RTT_BUCKET_COUNT];
        match name {
            "fixed-20" => weights[0] = 1,
            "fixed-60" => weights[10] = 1,
            "fixed-100" => weights[19] = 1,
            "uniform-20-100" => weights.fill(1),
            "low-skew" => {
                for (index, weight) in weights.iter_mut().enumerate() {
                    *weight = (RTT_BUCKET_COUNT - index) as u64;
                }
            }
            "high-skew" => {
                for (index, weight) in weights.iter_mut().enumerate() {
                    *weight = (index + 1) as u64;
                }
            }
            "bimodal-20-100" => {
                weights[0] = 10;
                weights[1] = 5;
                weights[18] = 5;
                weights[19] = 10;
            }
            other => return Err(NetemError::Config(format!("unknown profile {other}"))),
        }
        Self::new(name, weights)
    }
}

pub fn bucket_bounds(index: usize) -> Result<(u32, u32)> {
    if index >= RTT_BUCKET_COUNT {
        return Err(NetemError::Config(format!("invalid RTT bucket {index}")));
    }
    let low = RTT_MIN_MS + index as u32 * RTT_BUCKET_WIDTH_MS;
    let high = if index + 1 == RTT_BUCKET_COUNT {
        RTT_MAX_MS
    } else {
        low + RTT_BUCKET_WIDTH_MS - 1
    };
    Ok((low, high))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bucket_contract_is_twenty_four_ms_ranges() {
        assert_eq!(bucket_bounds(0).unwrap(), (20, 23));
        assert_eq!(bucket_bounds(10).unwrap(), (60, 63));
        assert_eq!(bucket_bounds(19).unwrap(), (96, 100));
        assert!(bucket_bounds(20).is_err())
    }
    #[test]
    fn invalid_weight_vectors_fail_closed() {
        assert!(DelayProfile::new("short", vec![1; 19]).is_err());
        assert!(DelayProfile::new("zero", vec![0; 20]).is_err());
        let mut values = vec![0; 20];
        values[0] = u64::MAX;
        values[1] = 1;
        assert!(DelayProfile::new("overflow", values).is_err())
    }
    #[test]
    fn builtins_are_pinned() {
        for name in [
            "fixed-20",
            "fixed-60",
            "fixed-100",
            "uniform-20-100",
            "low-skew",
            "high-skew",
            "bimodal-20-100",
        ] {
            let value = DelayProfile::named(name).unwrap();
            assert_eq!(value.weights.len(), 20);
            assert!(value.total_weight > 0)
        }
        assert_eq!(DelayProfile::named("fixed-20").unwrap().weights[0], 1);
        assert_eq!(DelayProfile::named("fixed-100").unwrap().weights[19], 1)
    }
}
