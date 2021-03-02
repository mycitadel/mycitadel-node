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

use colored::Colorize;
use std::fs;
use std::io;
use std::path::PathBuf;

use base64::display::Base64Display;
use bitcoin::consensus::{serialize, Encodable};
use bitcoin::hashes::hex::ToHex;
use wallet::Psbt;

use super::PsbtFormat;
use crate::Error;

pub(super) fn psbt_output(
    psbt: &Psbt,
    output: Option<PathBuf>,
    format: Option<PsbtFormat>,
) -> Result<(), Error> {
    let (mut psbt_file, format) = if let Some(ref filename) = output {
        (
            Box::new(io::BufWriter::new(fs::File::create(filename)?))
                as Box<dyn io::Write>,
            format.unwrap_or(PsbtFormat::Binary),
        )
    } else {
        (
            Box::new(io::BufWriter::new(io::stdout())) as Box<dyn io::Write>,
            format.unwrap_or(PsbtFormat::Base64),
        )
    };

    if output.is_none() {
        eprint!("{} ", "PSBT:".bright_yellow());
    }
    match format {
        PsbtFormat::Binary => {
            psbt.consensus_encode(&mut psbt_file)?;
        }
        PsbtFormat::Hexadecimal => {
            psbt_file.write_all(serialize(psbt).to_hex().as_bytes())?;
        }
        PsbtFormat::Base64 => {
            psbt_file.write_all(
                Base64Display::with_config(
                    &serialize(psbt),
                    ::base64::STANDARD,
                )
                .to_string()
                .as_bytes(),
            )?;
        }
    }
    psbt_file.flush()?;
    if output.is_none() {
        eprintln!("\n");
    }

    Ok(())
}
