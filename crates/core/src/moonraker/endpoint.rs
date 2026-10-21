//! Adresse eines Moonraker-Servers, aus der WebSocket- und HTTP-URL abgeleitet werden.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    secure: bool,
    authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid Moonraker address '{0}', expected e.g. http://printer.local:7125")]
pub struct InvalidEndpoint(String);

impl Endpoint {
    /// Akzeptiert `host[:port]`, `http(s)://host[:port]` und `ws(s)://host[:port]/websocket`.
    pub fn parse(address: &str) -> Result<Self, InvalidEndpoint> {
        let trimmed = address.trim();
        let (secure, rest) = match trimmed.split_once("://") {
            Some(("http" | "ws", rest)) => (false, rest),
            Some(("https" | "wss", rest)) => (true, rest),
            Some(_) => return Err(InvalidEndpoint(address.to_owned())),
            None => (false, trimmed),
        };
        let authority = rest.split('/').next().unwrap_or_default();
        if authority.is_empty() || authority.contains(char::is_whitespace) {
            return Err(InvalidEndpoint(address.to_owned()));
        }
        Ok(Self { secure, authority: authority.to_owned() })
    }

    pub fn websocket_url(&self) -> String {
        let scheme = if self.secure { "wss" } else { "ws" };
        format!("{scheme}://{}/websocket", self.authority)
    }

    pub fn http_url(&self, path: &str) -> String {
        let scheme = if self.secure { "https" } else { "http" };
        format!("{scheme}://{}/{}", self.authority, path.trim_start_matches('/'))
    }

    /// Kanonische Schreibweise für Anzeige und Einstellungen.
    pub fn display(&self) -> String {
        self.http_url("").trim_end_matches('/').to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_urls_from_plain_host() {
        let endpoint = Endpoint::parse("printer.local:7125").unwrap();

        assert_eq!(endpoint.websocket_url(), "ws://printer.local:7125/websocket");
        assert_eq!(
            endpoint.http_url("/server/files/gcodes/a.png"),
            "http://printer.local:7125/server/files/gcodes/a.png"
        );
        assert_eq!(endpoint.display(), "http://printer.local:7125");
    }

    #[test]
    fn accepts_websocket_and_secure_urls() {
        assert_eq!(Endpoint::parse("ws://10.0.0.2/websocket").unwrap().display(), "http://10.0.0.2");
        assert_eq!(Endpoint::parse("https://pi").unwrap().websocket_url(), "wss://pi/websocket");
    }

    #[test]
    fn rejects_unknown_scheme_and_empty_host() {
        assert!(Endpoint::parse("ftp://pi").is_err());
        assert!(Endpoint::parse("http://").is_err());
        assert!(Endpoint::parse("  ").is_err());
    }
}
