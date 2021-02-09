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

//! File storage driver

use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::{fs, io};

use lnpbp::strict_encoding::{StrictDecode, StrictEncode};
use microservices::FileFormat;

use super::{Driver, Error};
use crate::model::{Contract, ContractId, Policy, Wallet};
use crate::rpc::message::{IdentityInfo, SignerAccountInfo};
use crate::server::opts::MYCITADEL_STORAGE_FILE;

#[derive(Debug)]
pub struct FileDriver {
    fd: fs::File,
    config: FileConfig,
    data: Wallet,
}

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
        filename.push(MYCITADEL_STORAGE_FILE);
        filename.set_extension(self.format.extension());
        filename
    }
}

impl FileDriver {
    pub fn with(config: FileConfig) -> Result<Self, Error> {
        info!("Initializing file driver for data in {}", &config.location);
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
            data: Default::default(),
        };
        if !exists {
            warn!(
                "Data file `{:?}` does not exist: initializing empty citadel storage",
                filename
            );
            me.store()?;
        } else {
            me.load()?;
        }
        Ok(me)
    }

    fn load(&mut self) -> Result<(), Error> {
        debug!("Loading data from `{:?}`", self.config.filename());
        self.fd.seek(io::SeekFrom::Start(0))?;
        trace!("Parsing data (expected format {})", self.config.format);
        self.data = match self.config.format {
            FileFormat::StrictEncode => Wallet::strict_decode(&mut self.fd)?,
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
        trace!("Data loaded from storage");
        Ok(())
    }

    fn store(&mut self) -> Result<(), Error> {
        debug!(
            "Storing data to the file `{:?}` in {} format",
            self.config.filename(),
            self.config.format
        );
        self.fd.seek(io::SeekFrom::Start(0))?;
        self.fd.set_len(0)?;
        match self.config.format {
            FileFormat::StrictEncode => {
                self.data.strict_encode(&mut self.fd)?;
            }
            #[cfg(feature = "serde_yaml")]
            FileFormat::Yaml => {
                serde_yaml::to_writer(&mut self.fd, &self.data)?;
            }
            #[cfg(feature = "toml")]
            FileFormat::Toml => {
                let data = toml::to_vec(&self.data)?;
                self.fd.write_all(&data)?;
            }
            #[cfg(feature = "serde_json")]
            FileFormat::Json => {
                serde_json::to_writer(&mut self.fd, &self.data)?;
            }
            _ => unimplemented!(),
        };
        trace!("MyCitadel data stored");
        Ok(())
    }
}

impl Driver for FileDriver {
    fn contracts(&self) -> Result<Vec<Contract>, Error> {
        Ok(self.data.contracts.values().cloned().collect())
    }

    fn add_contract(&mut self, contract: Contract) -> Result<Contract, Error> {
        self.data.contracts.insert(*contract.id(), contract.clone());
        self.store()?;
        Ok(contract)
    }

    fn policy(&self, contract_id: ContractId) -> Result<&Policy, Error> {
        self.data
            .contracts
            .get(&contract_id)
            .ok_or(Error::ContractNotFound(contract_id))
            .map(Contract::policy)
    }

    fn signers(&self) -> Result<Vec<SignerAccountInfo>, Error> {
        unimplemented!()
    }

    fn add_signer(
        &mut self,
        _account: SignerAccountInfo,
    ) -> Result<SignerAccountInfo, Error> {
        unimplemented!()
    }

    fn identities(&self) -> Result<Vec<IdentityInfo>, Error> {
        unimplemented!()
    }

    fn add_identity(
        &mut self,
        _identity: IdentityInfo,
    ) -> Result<IdentityInfo, Error> {
        unimplemented!()
    }
}
