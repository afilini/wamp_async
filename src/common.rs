use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::hash::Hash;
use std::num::NonZeroU64;
use std::pin::Pin;

use crate::error::*;

use log::*;
use serde::{Deserialize, Serialize};

pub(crate) const DEFAULT_AGENT_STR: &str =
    concat!(env!("CARGO_PKG_NAME"), "_rs-", env!("CARGO_PKG_VERSION"));

/// uri: a string URI as defined in URIs
pub type WampUri = String;

/// id: an integer ID as defined in IDs
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WampId(NonZeroU64);

impl fmt::Display for WampId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<WampId> for NonZeroU64 {
    fn from(id: WampId) -> Self {
        id.0
    }
}

impl WampId {
    /// IDs in the global scope MUST be drawn randomly from a uniform distribution over the complete
    /// range [1, 2^53]
    pub(crate) fn generate() -> Self {
        let random_id = rand::random::<u64>() & ((1 << 53) - 1);
        // Safety: since random_id is in range of [0, 2**53) and we add 1, the value is always in
        // range [1, 2^53].
        Self(unsafe { NonZeroU64::new_unchecked(random_id + 1) })
    }
}

/// integer: a non-negative integer
pub type WampInteger = usize;
/// string: a Unicode string, including the empty string
pub type WampString = String;
/// bool: a boolean value (true or false)
pub type WampBool = bool;
/// dict: a dictionary (map) where keys MUST be strings
pub type WampDict = HashMap<String, Arg>;
/// list: a list (array) where items can be of any type
pub type WampList = Vec<Arg>;
/// Unnamed WAMP argument list
pub type WampArgs = Option<WampList>;
/// Named WAMP argument map
pub type WampKwArgs = Option<WampDict>;

/// Generic enum that can hold any concrete WAMP value
#[serde(untagged)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Arg {
    /// uri: a string URI as defined in URIs
    Uri(WampUri),
    /// id: an integer ID as defined in IDs
    Id(WampId),
    /// integer: a non-negative integer
    Integer(WampInteger),
    /// string: a Unicode string, including the empty string
    String(WampString),
    /// bool: a boolean value (true or false)
    Bool(WampBool),
    /// dict: a dictionary (map) where keys MUST be strings
    Dict(WampDict),
    /// list: a list (array) where items can be again any of this enumeration
    List(WampList),
    None,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// All roles a client can be
pub enum ClientRole {
    /// Client can call RPC endpoints
    Caller,
    /// Client can register RPC endpoints
    Callee,
    /// Client can publish events to topics
    Publisher,
    /// Client can register for events on topics
    Subscriber,
}
impl ClientRole {
    /// Returns the string repesentation of the role
    pub fn to_str(&self) -> &'static str {
        match self {
            ClientRole::Caller => "caller",
            ClientRole::Callee => "callee",
            ClientRole::Publisher => "publisher",
            ClientRole::Subscriber => "subscriber",
        }
    }
}

/// All the supported roles a server can have
pub enum ServerRole {
    /// Server supports RPC calls
    Router,
    /// Server supports pub/sub
    Broker,
}
impl ServerRole {
    /// Returns the string repesentation of the role
    pub fn to_str(&self) -> &'static str {
        match self {
            ServerRole::Router => "router",
            ServerRole::Broker => "broker",
        }
    }
}

/// Returns whether a uri is valid or not (using strict rules)
pub fn is_valid_strict_uri<T: AsRef<str>>(in_uri: T) -> bool {
    let uri: &str = in_uri.as_ref();
    let mut num_chars_token: usize = 0;
    if uri.starts_with("wamp.") {
        warn!("URI '{}' cannot start with 'wamp'", uri);
        return false;
    }

    for (i, c) in uri.chars().enumerate() {
        if c == '.' {
            if num_chars_token == 0 {
                warn!(
                    "URI '{}' contains a zero length token ending @ index {}",
                    uri, i
                );
                return false;
            }
            num_chars_token = 0;
        } else {
            num_chars_token += 1;
        }

        if c == '_' {
            continue;
        }

        if !c.is_lowercase() {
            warn!(
                "URI '{}' contains a non lower case character @ index {}",
                uri, i
            );
            return false;
        }
        if !c.is_alphanumeric() {
            warn!("URI '{}' contains an invalid character @ index {}", uri, i);
            return false;
        }
    }

    true
}

/// Future that can return success or an error
pub type GenericFuture = Pin<Box<dyn Future<Output = Result<(), WampError>> + Send>>;
/// Type returned by RPC functions
pub type RpcFuture =
    Pin<Box<dyn Future<Output = Result<(WampArgs, WampKwArgs), WampError>> + Send>>;
/// Generic function that can receive RPC calls
pub type RpcFunc = Box<dyn Fn(WampArgs, WampKwArgs) -> RpcFuture + Send + Sync>;
