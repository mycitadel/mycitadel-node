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

use microservices::shell::Exec;

use super::Command;
use crate::rpc;
use crate::Error;

impl Exec for Command {
    type Runtime = rpc::Client;
    type Error = Error;

    #[inline]
    fn exec(&self, _runtime: &mut rpc::Client) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
