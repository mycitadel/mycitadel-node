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

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
#[non_exhaustive]
pub enum Error {
    /// I/O error during storage operations. Details: {0}
    #[from]
    #[from(std::io::Error)]
    Io(amplify::IoError),

    /// item with id {0} is not found
    NotFound(String),

    /// Error in strict data encoding: {0}
    /// Make sure that the storage is not broken.
    #[from]
    StrictEncoding(strict_encoding::Error),

    /// error in YAML data encoding: {0}
    YamlEncoding(String),

    /// error in YAML data encoding
    #[from(serde_json::Error)]
    JsonEncoding,

    /// error in YAML data encoding
    #[from(toml::de::Error)]
    #[from(toml::ser::Error)]
    TomlEncoding,
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::YamlEncoding(err.to_string())
    }
}
