// MyCitadel: node, wallet library & command-line tool
// Written in 2020 by
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

//! Storage API

use super::{driver, Driver, FileDriver};
use crate::Error;

pub struct Storage {
    driver: Box<dyn Driver>,
    data: Vec<u8>,
}

impl Storage {
    pub fn with(config: &driver::Config) -> Result<Self, Error> {
        let mut driver = match config {
            driver::Config::File(fdc) => {
                Box::new(FileDriver::init(fdc)?) as Box<dyn Driver>
            }
        };
        let data = driver.load()?;
        Ok(Self { driver, data })
    }
}
