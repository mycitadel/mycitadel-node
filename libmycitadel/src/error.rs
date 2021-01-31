// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

#[derive(Debug, Display, From, Error)]
#[display(doc_comments)]
#[non_exhaustive]
pub(crate) enum RequestError {
    /// Input value is not a JSON object or JSON parse error: {0}
    #[from]
    Json(serde_json::Error),

    /// Input value is not a UTF8 string: {0}
    #[from]
    Utf8(std::str::Utf8Error),

    /// Impossible error: {0}
    #[from]
    Infallible(std::convert::Infallible),

    /// I/O error: {0}
    #[from]
    Io(std::io::Error),

    /// Input error: {0}
    #[from]
    Input(String),

    /// Internal error
    Internal,
}
