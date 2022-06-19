use std::{collections::HashMap, any::TypeId, str::FromStr, fmt::Debug};

use crate::math::{Vector2, Vector3, Vector4, v2, v3, v4};

pub struct Config {
    map: HashMap<String, String>
}

impl Config {
    pub fn from(text: &str) -> Self {
        let mut map = HashMap::new();
        for line in text.lines() {
            if let Some((i, _)) = line.match_indices("=").next() {
                let (k, v) = line.split_at(i);
                map.insert(k.to_owned(), v[1..].to_owned());
            }
        }
        Self { map }
    }

    fn parse<T>(k: &str, v: &str) -> T
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Debug
    {
        v.parse::<T>()
            .expect(&format!("failed to parse config key {} as {:?}", k, TypeId::of::<T>()))
    }
    fn get(&self, k: &str) -> &str {
        self.map.get(k).expect(&format!("no config key {}", k))
    }

    pub fn str(&self, k: &str) -> &str { self.get(k) }
    pub fn f32(&self, k: &str) -> f32 { Self::parse(k, self.get(k)) }
    pub fn usize(&self, k: &str) -> usize { Self::parse(k, self.get(k)) }

    pub fn v2(&self, k: &str) -> Vector2 {
        let mut it = self.str(k).split(",");
        let x = it.next().unwrap();
        let y = it.next().unwrap();
        v2(Self::parse(k, x), Self::parse(k, y))
    }
    pub fn v3(&self, k: &str) -> Vector3 {
        let mut it = self.str(k).split(",");
        let x = it.next().unwrap();
        let y = it.next().unwrap();
        let z = it.next().unwrap();
        v3(Self::parse(k, x), Self::parse(k, y), Self::parse(k, z))
    }
    pub fn v4(&self, k: &str) -> Vector4 {
        let mut it = self.str(k).split(",");
        let x = it.next().unwrap();
        let y = it.next().unwrap();
        let z = it.next().unwrap();
        let w = it.next().unwrap();
        v4(Self::parse(k, x), Self::parse(k, y), Self::parse(k, z), Self::parse(k, w))
    }
}
