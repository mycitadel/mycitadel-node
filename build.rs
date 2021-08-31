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

#[macro_use]
extern crate amplify;

use clap::IntoApp;
use clap_generate::{generate_to, generators::*};

pub mod shared {
    include!("src/shared.rs");
}

pub mod cli {
    include!("src/cli/opts.rs");
}
pub mod daemon {
    include!("src/daemon.rs");
}
pub mod embedded {
    include!("src/embedded.rs");
}

fn main() -> Result<(), configure_me_codegen::Error> {
    let outdir = "./shell";

    for app in [
        daemon::Opts::into_app(),
        cli::Opts::into_app(),
        embedded::Opts::into_app(),
    ]
    .iter_mut()
    {
        let name = app.get_name().to_string();
        generate_to::<Bash, _, _>(app, &name, &outdir)?;
        generate_to::<PowerShell, _, _>(app, &name, &outdir)?;
        generate_to::<Zsh, _, _>(app, &name, &outdir)?;
    }

    configure_me_codegen::build_script_auto()
}
