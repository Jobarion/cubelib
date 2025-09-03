use std::collections::HashMap;
use std::str::FromStr;
use std::str::pattern::Pattern;
use semver::Version;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct GithubReleaseData {
    tag_name: String,
    assets: Vec<GithubReleaseAsset>
}

#[derive(Clone, Debug, Deserialize)]
pub struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub struct GithubRelease {
    pub version: Version,
    pub assets: HashMap<String, GithubReleaseAsset>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum UpdateError {
    ReqwestError(reqwest::Error),
    NoVersionError,
}

pub fn fetch_latest() -> Result<GithubRelease, UpdateError> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("cubelib/{}", crate::VERSION))
        .build()
        .map_err(|e|UpdateError::ReqwestError(e))?;

    let resp = client.get("https://codeberg.org/api/v1/repos/joba/cubelib/releases").send()
        .map_err(|e|UpdateError::ReqwestError(e))?;

    let resp_json: Option<Vec<GithubReleaseData>> = resp.json()
        .map_err(|e|UpdateError::ReqwestError(e))?;
    resp_json.and_then(|releases|releases.into_iter()
        .filter_map(|x|{
            "v".strip_prefix_of(x.tag_name.as_str())
                .and_then(|v|semver::Version::from_str(v).ok())
                .map(|v|{
                    GithubRelease {
                        version: v,
                        assets: x.assets.into_iter()
                            .map(|a|(a.name.clone(), a))
                            .collect()
                    }
                })
        })
        .max_by_key(|x|x.version.clone()))
        .ok_or(UpdateError::NoVersionError)
}