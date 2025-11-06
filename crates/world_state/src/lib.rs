use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::hash_map::DefaultHasher, fmt, hash::Hasher, num::ParseIntError, str::FromStr,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntitySnapshot {
    pub id: u64,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub size: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Checksum(u64);

impl Checksum {
    pub const ZERO: Checksum = Checksum(0);

    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    pub fn to_u64(self) -> u64 {
        self.0
    }

    pub fn to_hex(self) -> String {
        format!("{:016x}", self.0)
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl FromStr for Checksum {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(s, 16).map(Checksum)
    }
}

impl Serialize for Checksum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Checksum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Checksum::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    pub tick: u64,
    pub checksum: Checksum,
    pub entities: Vec<EntitySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Diff {
    pub tick: u64,
    pub base: Checksum,
    pub checksum: Checksum,
    pub added: Vec<EntitySnapshot>,
    pub removed: Vec<u64>,
    pub changed: Vec<EntitySnapshot>,
}

pub fn checksum_for_state(tick: u64, entities: &[EntitySnapshot]) -> Checksum {
    let mut hasher = DefaultHasher::new();
    hasher.write_u64(tick);
    for entity in entities {
        hasher.write_u64(entity.id);
        for value in entity.pos {
            hasher.write_u32(value.to_bits());
        }
        for value in entity.vel {
            hasher.write_u32(value.to_bits());
        }
        for value in entity.size {
            hasher.write_u32(value.to_bits());
        }
    }
    Checksum(hasher.finish())
}
