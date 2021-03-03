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

use invoice::Invoice;
use lnpbp::seals::OutpointReveal;
use lnpbp::strict_encoding::{StrictDecode, StrictEncode};
use microservices::FileFormat;

use super::{Driver, Error};
use crate::model::{
    Citadel, Contract, ContractId, Operation, Policy, TweakedOutput,
};
use crate::rpc::message::{IdentityInfo, SignerAccountInfo};
use crate::server::opts::MYCITADEL_STORAGE_FILE;

#[derive(Debug)]
pub struct FileDriver {
    fd: fs::File,
    config: FileConfig,
    data: Citadel,
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
            FileFormat::StrictEncode => Citadel::strict_decode(&mut self.fd)?,
            FileFormat::Yaml => serde_yaml::from_reader(&mut self.fd)?,
            FileFormat::Toml => {
                let mut data: Vec<u8> = vec![];
                self.fd.read_to_end(&mut data)?;
                toml::from_slice(&data)?
            }
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
            FileFormat::Yaml => {
                serde_yaml::to_writer(&mut self.fd, &self.data)?;
            }
            FileFormat::Toml => {
                let data = toml::to_vec(&self.data)?;
                self.fd.write_all(&data)?;
            }
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

    fn contract_ref(
        &self,
        contract_id: ContractId,
    ) -> Result<&Contract, Error> {
        self.data
            .contracts
            .get(&contract_id)
            .ok_or(Error::ContractNotFound(contract_id))
    }

    fn add_contract(&mut self, contract: Contract) -> Result<Contract, Error> {
        self.data.contracts.insert(*contract.id(), contract.clone());
        self.store()?;
        Ok(contract)
    }

    fn rename_contract(
        &mut self,
        contract_id: ContractId,
        new_name: String,
    ) -> Result<(), Error> {
        self.data
            .contracts
            .get_mut(&contract_id)
            .ok_or(Error::ContractNotFound(contract_id))?
            .name = new_name;
        self.store()?;
        Ok(())
    }

    fn delete_contract(
        &mut self,
        contract_id: ContractId,
    ) -> Result<(), Error> {
        self.data
            .contracts
            .remove(&contract_id)
            .ok_or(Error::ContractNotFound(contract_id))?;
        self.store()?;
        Ok(())
    }

    fn policy(&self, contract_id: ContractId) -> Result<&Policy, Error> {
        self.data
            .contracts
            .get(&contract_id)
            .ok_or(Error::ContractNotFound(contract_id))
            .map(Contract::policy)
    }

    fn add_invoice(
        &mut self,
        contract_id: ContractId,
        invoice: Invoice,
        reveal_info: Vec<OutpointReveal>,
    ) -> Result<(), Error> {
        let contract = self
            .data
            .contracts
            .get_mut(&contract_id)
            .ok_or(Error::ContractNotFound(contract_id))?;
        contract.add_invoice(invoice);
        for reveal in reveal_info {
            contract.add_blinding(reveal);
        }
        self.store()?;
        Ok(())
    }

    fn add_p2c_tweak(
        &mut self,
        contract_id: ContractId,
        tweak: TweakedOutput,
    ) -> Result<(), Error> {
        let contract = self
            .data
            .contracts
            .get_mut(&contract_id)
            .ok_or(Error::ContractNotFound(contract_id))?;
        contract.add_p2c_tweak(tweak);
        self.store()?;
        Ok(())
    }

    fn register_operation(
        &mut self,
        contract_id: ContractId,
        operation: Operation,
    ) -> Result<(), Error> {
        let contract = self
            .data
            .contracts
            .get_mut(&contract_id)
            .ok_or(Error::ContractNotFound(contract_id))?;
        contract.add_operation(operation);
        self.store()?;
        Ok(())
    }

    fn history(
        &self,
        contract_id: ContractId,
    ) -> Result<Vec<&Operation>, Error> {
        let contract = self
            .data
            .contracts
            .get(&contract_id)
            .ok_or(Error::ContractNotFound(contract_id))?;
        Ok(contract.history())
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
