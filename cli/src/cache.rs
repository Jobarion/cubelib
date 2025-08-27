use crate::get_cache_file;
use chrono::{DateTime, Utc};
use log::warn;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheData {
    #[serde(default = "default_last_update_check")]
    pub last_update_check: DateTime<Utc>,
}

pub struct Cache {
    data: Option<CacheData>,
}

impl Default for Cache {
    fn default() -> Self {
        Self { data: None }
    }
}

pub struct CacheGuard<'a>(&'a mut CacheData);

impl Cache {
    pub fn get_data(&mut self) -> &CacheData {
        self.get_mut_data()
    }

    fn get_mut_data(&mut self) -> &mut CacheData {
        if let Some(data) = self.data.take() {
            self.data.insert(data)
        } else {
            let new_data = match fs::read_to_string(get_cache_file()) {
                Ok(s) => Ok(s),
                Err(e) if e.kind() == ErrorKind::NotFound => Ok("".to_string()),
                Err(e) => Err(e),
            }
            .map_err(|e| e.to_string())
            .and_then(|s| toml::from_str::<CacheData>(s.as_str()).map_err(|e| e.to_string()))
            .unwrap_or_else(|e| {
                warn!("Error reading cache file: {e}");
                CacheData::default()
            });
            self.data = Some(new_data);
            self.data.as_mut().unwrap()
        }
    }

    pub fn get_and_update(&mut self) -> CacheGuard<'_> {
        CacheGuard(self.get_mut_data())
    }
}

impl<'a> Drop for CacheGuard<'a> {
    fn drop(&mut self) {
        let toml = match toml::to_string(self.0) {
            Ok(toml) => toml,
            Err(e) => {
                warn!("Error serializing cache data: {e}");
                return;
            }
        };
        if let Err(e) = fs::write(get_cache_file(), toml) {
            warn!("Error saving cache data: {e}");
        }
    }
}

impl<'a> Deref for CacheGuard<'a> {
    type Target = CacheData;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> DerefMut for CacheGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl Default for CacheData {
    fn default() -> Self {
        Self {
            last_update_check: default_last_update_check(),
        }
    }
}

pub fn default_last_update_check() -> DateTime<Utc> {
    DateTime::default()
}
