// MyCitadel: node, wallet library & command-line tool
// Written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@mycitadel.io>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the AGPL License
// along with this software.
// If not, see <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::{fs, io};

use lnpbp::strict_encoding::{StrictDecode, StrictEncode};
use microservices::FileFormat;

use super::Cache;
use crate::cache::Error;
use crate::server::opts::MYCITADEL_CACHE_FILE;

#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    StrictEncode,
    StrictDecode,
)]
#[serde(crate = "serde_crate")]
pub struct FileConfig {
    pub location: String,
    pub format: FileFormat,
}

impl FileConfig {
    pub fn filename(&self) -> PathBuf {
        let mut filename = PathBuf::from(self.location.clone());
        filename.push(MYCITADEL_CACHE_FILE);
        filename.set_extension(self.format.extension());
        filename
    }
}

#[derive(Debug)]
pub struct FileDriver {
    fd: fs::File,
    config: FileConfig,
    pub(super) cache: Cache,
}

impl FileDriver {
    pub fn with(config: FileConfig) -> Result<Self, Error> {
        info!("Initializing file driver for cache in {}", &config.location);
        fs::create_dir_all(&config.location)?;

        let filename = config.filename();
        let exists = filename.exists();
        let fd = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(!exists)
            .open(&filename)?;
        let mut me = Self {
            fd,
            config: config.clone(),
            cache: none!(),
        };
        if !exists {
            warn!(
                "Cache file `{:?}` does not exist: initializing empty citadel cache",
                filename
            );
            me.store()?;
        }
        Ok(me)
    }

    fn load(&mut self) -> Result<(), Error> {
        debug!("Loading cache from `{:?}`", self.config.filename());
        self.fd.seek(io::SeekFrom::Start(0))?;
        trace!("Parsing cache (expected format {})", self.config.format);
        self.cache = match self.config.format {
            FileFormat::StrictEncode => Cache::strict_decode(&mut self.fd)?,
            #[cfg(feature = "serde_yaml")]
            FileFormat::Yaml => serde_yaml::from_reader(&mut self.fd)?,
            #[cfg(feature = "toml")]
            FileFormat::Toml => {
                let mut data: Vec<u8> = vec![];
                self.fd.read_to_end(&mut data)?;
                toml::from_slice(&data)?
            }
            #[cfg(feature = "serde_json")]
            FileFormat::Json => serde_json::from_reader(&mut self.fd)?,
            _ => unimplemented!(),
        };
        trace!("Cache loaded from storage");
        Ok(())
    }

    fn store(&mut self) -> Result<(), Error> {
        debug!(
            "Storing cache to the file `{:?}` in {} format",
            self.config.filename(),
            self.config.format
        );
        self.fd.seek(io::SeekFrom::Start(0))?;
        self.fd.set_len(0)?;
        match self.config.format {
            FileFormat::StrictEncode => {
                self.cache.strict_encode(&mut self.fd)?;
            }
            #[cfg(feature = "serde_yaml")]
            FileFormat::Yaml => {
                serde_yaml::to_writer(&mut self.fd, &self.cache)?;
            }
            #[cfg(feature = "toml")]
            FileFormat::Toml => {
                let data = toml::to_vec(&self.cache)?;
                self.fd.write_all(&data)?;
            }
            #[cfg(feature = "serde_json")]
            FileFormat::Json => {
                serde_json::to_writer(&mut self.fd, &self.cache)?;
            }
            _ => unimplemented!(),
        };
        trace!("Cache stored");
        Ok(())
    }
}
