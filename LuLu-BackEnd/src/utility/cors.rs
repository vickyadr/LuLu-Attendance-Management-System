use actix_cors::Cors;
use actix_web::http::header;

/// Read the origins this server answers for, from the environment.
///
/// A deployment behind one hostname sets `ALLOWED_ORIGINS=https://lulu.example.com`.
/// When it is unset the server falls back to origins on private address space,
/// which covers a LAN install (the usual case) and localhost. Public origins are
/// never accepted by default: a browser on attacker.example must not be able to
/// read LuLu's responses with the operator's session token attached.
pub fn allowed_origins() -> Vec<String> {
    match std::env::var("ALLOWED_ORIGINS") {
        Ok(raw) => raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Whether `origin` is one this server serves.
///
/// An entry may be a full origin (`https://lulu.example.com`) or a bare host
/// (`192.168.1.10`, `lulu.local`, `localhost`). A bare host matches on
/// authority only, so a caller can point the SPA at any port or scheme on that
/// host without re-listing it. A full origin must match exactly, scheme included.
///
/// Private and loopback hosts are accepted without configuration: this is an
/// attendance server on an internal network, and a fixed list would otherwise
/// mean editing the environment every time the LAN address changes.
pub fn origin_allowed(origin: &str, allowed: &[String]) -> bool {
    // A browser always sends a scheme in `Origin`, so a value without one is not
    // an origin. Parsing it anyway would let `192.168.1.10:3232` - which is what
    // a hand-written allowlist entry looks like - pass as a request origin.
    let requested = Authority::parse_origin(origin);

    if allowed.is_empty() {
        return requested.is_some_and(|a| a.is_private_range());
    }

    allowed.iter().any(|entry| {
        let entry = entry.trim();
        // A configured value may carry a scheme or be a bare host, so it goes
        // through the lenient parser.
        let Some(configured) = Authority::parse(entry) else {
            return false;
        };

        if !entry.contains("://") {
            // A bare host matches the authority alone, on any port and scheme.
            // An entry that names a port narrows to that port.
            requested.as_ref().is_some_and(|r| {
                r.host == configured.host
                    && (configured.port.is_none() || r.port == configured.port)
            })
        } else {
            // A full origin has to match exactly, scheme included.
            entry.trim_end_matches('/') == origin.trim_end_matches('/')
        }
    })
}

/// The `host[:port]` part of an origin, with credentials and brackets removed.
///
/// A bare host is accepted too: that is how an operator writes an entry in
/// `ALLOWED_ORIGINS` (`192.168.1.10`, `lulu.example.com`, `[fd00::1]`).
#[derive(Debug, PartialEq, Eq)]
struct Authority {
    host: String,
    port: Option<String>,
}

impl Authority {
    /// Parse a request `Origin`: a scheme is required, because a browser sends one.
    fn parse_origin(input: &str) -> Option<Self> {
        input.split_once("://").and_then(|(scheme, _)| {
            matches!(scheme, "http" | "https").then(|| Self::parse(input))?
        })
    }

    /// Parse either a full origin or a bare `host[:port]`.
    fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        // Split the scheme off when there is one; a bare host keeps everything.
        let rest = match input.split_once("://") {
            Some((scheme, rest)) => {
                // A browser Origin is http or https, so anything else is not one
                // and must not be waved through by the range check below.
                if !matches!(scheme, "http" | "https") {
                    return None;
                }
                rest
            }
            None => input,
        };
        // Drop any path, then any credentials.
        let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
        let host_port = authority.rsplit('@').next().unwrap_or(authority);

        if let Some(rest) = host_port.strip_prefix('[') {
            // IPv6 literal: the closing bracket ends the host, and what follows
            // is the port. Splitting on ':' here would cut the address in half.
            let (host, tail) = rest.split_once(']')?;
            if host.is_empty() {
                return None;
            }
            let port = match tail.strip_prefix(':') {
                Some("") | None => None,
                Some(p) => Some(p.to_string()),
            };
            return Some(Self {
                host: host.to_ascii_lowercase(),
                port,
            });
        }

        // Not bracketed, so at most one colon can separate a port. More than one
        // would mean a bare IPv6 literal, which must be bracketed to be valid.
        match host_port.split_once(':') {
            Some((host, port)) if !host.is_empty() && !port.is_empty() => Some(Self {
                host: host.to_ascii_lowercase(),
                port: Some(port.to_string()),
            }),
            Some(_) => None,
            None if !host_port.is_empty() => Some(Self {
                host: host_port.to_ascii_lowercase(),
                port: None,
            }),
            None => None,
        }
    }

    /// Whether the host sits in loopback, link-local, or RFC1918 private space.
    fn is_private_range(&self) -> bool {
        let bare = self.host.as_str();
        if bare == "localhost" || bare.ends_with(".localhost") || bare.ends_with(".local") {
            // `.local` is reserved for mDNS by RFC 6762 and is not a public
            // suffix, so it names a machine on this network and nothing else.
            // Match on the suffix, not a substring: `x.local.evil.com` is public.
            return true;
        }
        if let Ok(v4) = bare.parse::<std::net::Ipv4Addr>() {
            return v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified();
        }
        if let Ok(v6) = bare.parse::<std::net::Ipv6Addr>() {
            return v6.is_loopback() || unique_local(&v6) || v6.is_unicast_link_local();
        }
        false
    }
}

/// fc00::/7 - the IPv6 private range, which `Ipv6Addr` has no named method for.
fn unique_local(addr: &std::net::Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xfe00) == 0xfc00
}

/// Log the effective CORS scope once, at startup.
///
/// `policy()` runs inside the per-worker factory closure, so anything it logs
/// repeats for every worker; this is where the operator actually looks.
pub fn log_scope() {
    let allowed = allowed_origins();
    if allowed.is_empty() {
        log::warn!(
            "ALLOWED_ORIGINS is unset: answering only loopback and private-range origins. \
             Set it to your public hostname to serve LuLu over the internet."
        );
    } else {
        log::info!("CORS restricted to {} configured origin(s)", allowed.len());
    }
}

/// The CORS policy the server installs.
///
/// Credentials stay enabled because the SPA authenticates with a session token,
/// which makes the origin check the only thing standing between a hostile page
/// and an authenticated session - hence the default above.
pub fn policy() -> Cors {
    let allowed = allowed_origins();

    Cors::default()
        // `HeaderValue` has no `Display`, so go through `to_str` and treat an
        // unparseable origin as refused rather than trusting it.
        .allowed_origin_fn(move |origin, _| {
            origin.to_str().is_ok_and(|o| origin_allowed(o, &allowed))
        })
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::ACCEPT_ENCODING,
            header::ACCEPT_LANGUAGE,
            header::HOST,
            header::CONNECTION,
            header::ORIGIN,
            header::REFERER,
            header::USER_AGENT,
        ])
        .expose_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
        .supports_credentials()
        .max_age(3600)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(origins: &[&str]) -> Vec<String> {
        origins.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_public_origin_is_refused_by_default() {
        // The attack this prevents: a page on another site reading LuLu's
        // responses with the operator's session token attached.
        assert!(!origin_allowed("https://attacker.example", &[]));
        assert!(!origin_allowed("http://evil.test", &[]));
    }

    #[test]
    fn lan_and_loopback_origins_serve_without_configuration() {
        assert!(origin_allowed("http://192.168.1.10:3232", &[]));
        assert!(origin_allowed("http://10.0.0.7", &[]));
        assert!(origin_allowed("http://127.0.0.1:3232", &[]));
        assert!(origin_allowed("http://localhost:3232", &[]));
        assert!(origin_allowed("https://lulu.local", &[]));
        // IPv6 loopback and unique-local.
        assert!(origin_allowed("http://[::1]:3232", &[]));
        assert!(origin_allowed("http://[fd00::1]:3232", &[]));
    }

    #[test]
    fn a_public_origin_is_refused_even_when_it_looks_local() {
        // Not a bypass: the suffix test must not let a real domain through.
        assert!(!origin_allowed("http://localhost.attacker.example", &[]));
        assert!(!origin_allowed("http://127.0.0.1.evil.test", &[]));
        assert!(!origin_allowed("https://lulu.local.evil.com", &[]));
        assert!(!origin_allowed("https://notlocal.example", &[]));
        assert!(!origin_allowed("https://localhost.evil.test", &[]));
    }

    #[test]
    fn a_configured_origin_list_replaces_the_private_default() {
        let allowed = cfg(&["https://lulu.example.com"]);
        assert!(origin_allowed("https://lulu.example.com", &allowed));
        // Explicit configuration is exhaustive: a LAN address is not on it.
        assert!(!origin_allowed("http://192.168.1.10:3232", &allowed));
        assert!(!origin_allowed("https://attacker.example", &allowed));
    }

    #[test]
    fn a_bare_host_entry_matches_any_port_or_scheme() {
        let allowed = cfg(&["lulu.example.com"]);
        assert!(origin_allowed("https://lulu.example.com", &allowed));
        assert!(origin_allowed("http://lulu.example.com:3232", &allowed));
        assert!(!origin_allowed("https://other.example.com", &allowed));
        // A bare host is not a wildcard.
        assert!(!origin_allowed("https://sub.lulu.example.com", &allowed));
    }

    #[test]
    fn a_full_origin_entry_has_to_match_exactly() {
        let allowed = cfg(&["https://lulu.example.com"]);
        assert!(origin_allowed("https://lulu.example.com", &allowed));
        // Scheme is part of the match; the port comes from the origin itself.
        assert!(!origin_allowed("http://lulu.example.com", &allowed));
    }

    #[test]
    fn a_configured_private_host_still_serves_when_listed() {
        // The LAN install case: list the address explicitly and it works.
        let allowed = cfg(&["192.168.1.10", "http://localhost:3232"]);
        assert!(origin_allowed("http://192.168.1.10:3232", &allowed));
        assert!(origin_allowed("http://localhost:3232", &allowed));
        assert!(!origin_allowed("http://192.168.1.11:3232", &allowed));
    }

    #[test]
    fn the_scheme_prefix_check_runs_before_the_split() {
        // "http://evil\n://x" and other non-schemes must not be treated as origins.
        assert!(!origin_allowed("ftp://192.168.1.10", &[]));
        assert!(!origin_allowed("192.168.1.10:3232", &[]));
        assert!(!origin_allowed("", &[]));
        assert!(!origin_allowed("null", &[]));
    }

    #[test]
    fn an_ipv6_authority_is_not_mistaken_for_a_port() {
        // Splitting on the first colon would yield host "[" and lose the address.
        assert_eq!(
            Authority::parse("http://[fd00::1]:3232"),
            Some(Authority {
                host: "fd00::1".to_string(),
                port: Some("3232".to_string())
            })
        );
        assert_eq!(
            Authority::parse("http://[fd00::1]"),
            Some(Authority {
                host: "fd00::1".to_string(),
                port: None
            })
        );
        assert!(origin_allowed("http://[fe80::1]:3232", &[]));
        assert!(!origin_allowed("http://[2001:db8::1]:3232", &[]));
    }

    #[test]
    fn the_environment_is_parsed_into_a_trimmed_list() {
        // Pure parse check: reading a variable here would race other tests.
        let raw = " https://a.example , ,https://b.example ";
        let parsed: Vec<String> = raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        assert_eq!(parsed, vec!["https://a.example", "https://b.example"]);
    }

    #[test]
    fn a_bare_ipv6_host_entry_is_configurable() {
        let allowed = cfg(&["[fd00::1]"]);
        assert!(origin_allowed("http://[fd00::1]:3232", &allowed));
    }
}
