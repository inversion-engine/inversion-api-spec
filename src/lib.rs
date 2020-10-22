#![forbid(unsafe_code)]
#![forbid(warnings)]
#![forbid(missing_docs)]
//! Utilities for parsing and validation of inversion-api specification documents.

use derive_more::*;
use indexmap::*;

/// Re-exported dependencies.
pub mod dependencies {
    pub use ::derive_more;
    pub use ::indexmap;
    pub use ::nanoid;
    pub use ::once_cell;
    pub use ::serde;
    pub use ::serde_json;
}

/// Newtype for a nanoid string.
#[derive(
    Debug,
    Display,
    serde::Serialize,
    serde::Deserialize,
    Deref,
    From,
    Into,
    PartialEq,
)]
pub struct NanoId(pub String);

impl Default for NanoId {
    fn default() -> Self {
        Self(nanoid::nanoid!())
    }
}

impl From<&str> for NanoId {
    fn from(o: &str) -> Self {
        Self(o.to_string())
    }
}

/// Doc wrapper is a heuristic for identifying the document type.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IApiSpecDoc {
    /// This provides the actual api spec.
    inversion_api_spec: IApiSpec,
}

/// The actual inversion api spec.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IApiSpec {
    /// nanoid spec identifier
    id: NanoId,

    /// spec title
    title: String,

    /// spec revision
    revision: u32,

    /// If a call generates an error, this type will be returned.
    error_type: String,

    /// Some(true) if a broker should only allow one implementation.
    #[serde(skip_serializing_if = "Option::is_none")]
    unique: Option<bool>,

    /// Stablized features that *must* exist in implementations (by revision).
    features: IndexMap<String, Feature>,

    /// Unstable Features:
    /// - apis may change between revisions
    /// - unstable features may be dropped
    /// - unstable features may be omitted by implementations
    unstable_features: IndexMap<String, UnstableFeature>,

    /// Types to be used in error_type or input/output of calls.
    types: IndexMap<String, Type>,

    /// A dependant binding may make requests of its owner.
    calls_out: IndexMap<String, Call>,

    /// An owner may bind an api dependency and make calls to it.
    calls_in: IndexMap<String, Call>,
}

/// Stable Feature Definition
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    /// documentation for feature
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,

    /// feature was stablized at this revision number
    stablized_revision: u32,

    /// this feature is deprecated, and may no longer be supported by implementors
    #[serde(skip_serializing_if = "Option::is_none")]
    deprecated: Option<bool>,
}

/// Unstable Feature Definition
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UnstableFeature {
    /// documentation for feature
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,
}

/// The structured types allowed by inversion api.
/// Note how you can use NamedType to refer to other types.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Type {
    /// If a call doesn't take any parameters, or doesn't return any data.
    Null {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },
    /// `true` or `false`
    Bool {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },
    /// 32 bit signed integer
    I32 {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },
    /// 32 bit unsigned integer
    U32 {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },
    /// 64 bit signed integer
    I64 {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },
    /// 64 bit unsigned integer
    U64 {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },
    /// 64 bit floating point number
    F64 {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },
    /// Binary byte array
    Bytes {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },
    /// Utf-8 string
    String {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },
    /// A subtype that is allowed to be omitted
    Optional {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
        /// the type of the optional item
        content: Box<Type>,
    },
    /// Array of identical subtypes
    Array {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
        /// content type for array items
        content: Box<Type>,
    },
    /// Structured set of subtypes
    // "Struct" values will be stored as a msgpack array.
    // The individual values will be stored at the array index
    // defined in StructContent
    Struct {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
        /// map of struct item types/names
        content: IndexMap<String, StructContent>,
    },
    /// "One of" a set of subtypes
    // "Enum" values will be stored as a msgpack array length 2.
    // The first value will be the literal number index defined in StructContent
    // The second value will be the content type defined in StructContent
    Enum {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
        /// map of enum item types/names
        content: IndexMap<String, StructContent>,
    },
    /// Reference to another defined type
    NamedType {
        /// documentation
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
        /// name of referenced named type
        content: String,
    },
}

/// Struct data definition
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StructContent {
    /// documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,
    /// The index of this struct item
    index: u32,
    /// The content type of this struct item
    content: Box<Type>,
}

/// Call data definition
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Call {
    /// documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,
    /// Which feature this call is defined in
    feature: String,
    /// The named type for the input to this call
    input: String,
    /// The named type for the output of this call
    output: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile in a parsed static IApiSpecDoc
    /// Not sure if we want to expose this or not...
    macro_rules! include_iapi_spec_doc {
        ($name:ident, $path:expr) => {
            static $name: $crate::dependencies::once_cell::sync::Lazy<
                $crate::IApiSpecDoc,
            > = $crate::dependencies::once_cell::sync::Lazy::new(|| {
                const DATA: &[u8] = ::std::include_bytes!($path);
                $crate::dependencies::serde_json::from_slice(DATA).unwrap()
            });
        };
    }

    const JSON_FIXTURE_SRC: &[u8] = include_bytes!("fixture_spec.json");
    include_iapi_spec_doc!(JSON_FIXTURE, "fixture_spec.json");

    #[test]
    fn round_trip_encode_decode() {
        let doc: IApiSpecDoc =
            serde_json::from_slice(JSON_FIXTURE_SRC).unwrap();
        assert_eq!(*JSON_FIXTURE, doc);
        {
            let spec = &doc.inversion_api_spec;
            assert_eq!("gwSMYpO3kr5yLvTNR3KR4", spec.id.as_str());
        }
        let res = serde_json::to_string_pretty(&doc).unwrap();
        assert_eq!(
            String::from_utf8_lossy(JSON_FIXTURE_SRC).trim(),
            res.as_str().trim()
        );
        let doc2: IApiSpecDoc = serde_json::from_str(&res).unwrap();
        assert_eq!(doc, doc2);
    }
}
