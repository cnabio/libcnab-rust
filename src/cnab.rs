use std::collections::btree_map::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Bundle implements a CNAB bundle descriptor
///
/// Bundle descriptors describe the properties of a bundle, including which images
/// are associated, what parameters and credentials are configurable, and whether there
/// are any additional target actions that can be executed on this bundle.
///
/// The fields here are in canonical order.
#[derive(Debug, Serialize, Deserialize)]
pub struct Bundle {
    /// The list of additional actions that this bundle can perform.
    ///
    /// 'install', 'upgrade', and 'uninstall' are default actions, but additional actions
    /// may be defined here.
    pub actions: Option<BTreeMap<String, Action>>,
    /// The list of configurable credentials.
    ///
    /// Credentials are injected into the bundle's invocation image at startup time.
    pub credentials: Option<BTreeMap<String, Credential>>,
    /// This field allows for additional data to described in the bundle.
    ///
    /// This data should be stored in key/value pairs, where the value is undefined by
    /// the specification (but must be representable as JSON).
    pub custom: Option<BTreeMap<String, serde_json::Value>>,
    /// description is a short description of this bundle
    pub description: Option<String>,
    /// The list of images that comprise this bundle.
    ///
    /// Each image here is considered a constituent of the application described by this
    /// bundle.
    pub images: Option<BTreeMap<String, Image>>,
    /// invocation_images is the list of available bootstrapping images for this bundle
    ///
    /// Only one ought to be executed.
    #[serde(rename = "invocationImages")]
    pub invocation_images: Vec<Image>,
    /// keywords is a list of keywords describing this bundle
    pub keywords: Option<Vec<String>>,
    /// license is the license of this bundle
    pub license: Option<String>,
    /// maintainers is a list of maintainers responsible for this bundle
    pub maintainers: Option<Vec<Maintainer>>,
    /// name is the name of the bundle
    pub name: String,
    /// The collection of parameters that can be passed into this bundle.
    ///
    /// Parameters can be injected into a bundle during startup time.
    pub parameters: Option<BTreeMap<String, Parameter>>,
    /// schema_version is the version of the CNAB specification used to describe this
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    /// version is the version of the bundle
    pub version: String,
}

/// Represents a bundle.
impl Bundle {
    /// A convenience function to open and deserialize a [`bundle.json`](https://github.com/deislabs/cnab-spec/blob/master/101-bundle-json.md) file.
    ///
    /// ```
    /// use libcnab::Bundle;
    ///
    /// let bundle = Bundle::from_file("testdata/bundle.json").unwrap();
    /// assert_eq!(bundle.name, "helloworld");
    /// ```
    pub fn from_file(file_path: &str) -> Result<Self, BundleParseError> {
        let file = File::open(Path::new(&file_path))?;
        let res: Result<Bundle, canonical_json::Error> = canonical_json::from_reader(file);
        match res {
            Ok(b) => Ok(b),
            Err(err) => {
                match err {
                    // the canonical JSON parser errors out if the input JSON is not canonical
                    // for now, we should accept non-canonical JSON as input
                    canonical_json::Error::Syntax(
                        canonical_json::SyntaxError::UnexpectedWhitespace,
                        _,
                        _,
                    ) => {
                        let file = File::open(Path::new(&file_path)).expect("file not found");
                        let b: Result<Bundle, serde_json::Error> = serde_json::from_reader(file);
                        match b {
                            Ok(b) => return Ok(b),
                            Err(err) => return Err(BundleParseError::from(err)),
                        };
                    }
                    // TODO - Radu M - check other error types generated because JSON is not canonical
                    _ => (),
                }
                Err(BundleParseError::from(err))
            }
        }
    }
}

impl FromStr for Bundle {
    type Err = BundleParseError;

    fn from_str(json_data: &str) -> Result<Bundle, BundleParseError> {
        let bundle: Result<Bundle, canonical_json::Error> = canonical_json::from_str(json_data);
        match bundle {
            Ok(b) => Ok(b),
            Err(err) => {
                match err {
                    // the canonical JSON parser errors out if the input JSON is not canonical
                    // for now, we should accept non-canonical JSON as input
                    canonical_json::Error::Syntax(
                        canonical_json::SyntaxError::UnexpectedWhitespace,
                        _,
                        _,
                    ) => {
                        let b: Result<Bundle, serde_json::Error> = serde_json::from_str(json_data);
                        match b {
                            Ok(b) => return Ok(b),
                            Err(err) => return Err(BundleParseError::from(err)),
                        };
                    }
                    // TODO - Radu M - check other error types generated because JSON is not canonical
                    _ => (),
                }
                Err(BundleParseError::from(err))
            }
        }
    }
}

/// Represents an error parsing a bundle descriptor
///
/// This captures the various errors that may bubble up when a bundle descriptor
/// fails to parse.
#[derive(Debug)]
pub enum BundleParseError {
    SerdeJSONError(serde_json::Error),
    CanonicalJSONError(canonical_json::Error),
    IoError(std::io::Error),
}

impl From<std::io::Error> for BundleParseError {
    fn from(error: std::io::Error) -> Self {
        BundleParseError::IoError(error)
    }
}

impl From<serde_json::Error> for BundleParseError {
    fn from(error: serde_json::Error) -> Self {
        BundleParseError::SerdeJSONError(error)
    }
}

impl From<canonical_json::Error> for BundleParseError {
    fn from(error: canonical_json::Error) -> Self {
        BundleParseError::CanonicalJSONError(error)
    }
}

/// Maintainer describes a bundle maintainer.
///
/// The name field is required, though the format of its value is unspecified.
#[derive(Debug, Serialize, Deserialize)]
pub struct Maintainer {
    /// The email address of the maintainer
    pub email: Option<String>,
    /// The name of the maintainer
    pub name: String,
    /// A URL with more information about the maintainer
    pub url: Option<String>,
}

/// Image describes a CNAB image.
///
/// Both invocation images and regular images can be described using this object.
#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    /// A digest to be used to verify the integrity of the image
    pub digest: Option<String>,
    /// The image, as a string of the form REPO/NAME:TAG@SHA
    pub image: String,
    /// The type of image. Typically, this is treated as an OCI Image
    #[serde(rename = "imageType")]
    pub image_type: Option<String>,
    /// The media type of the image
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,
    /// The platform this image may be deployed on
    pub platform: Option<Platform>,
    /// The size in bytes of the image
    pub size: Option<i64>,
}

/// Platform defines a platform as a machine architecture plus and operating system
#[derive(Debug, Serialize, Deserialize)]
pub struct Platform {
    /// The architecture
    ///
    /// Typical values are amd64, i386, and arm64
    pub arch: Option<String>,
    /// The operating system.
    ///
    /// Typical values are darwin, windows, and linux
    pub os: Option<String>,
}

/// Credential describes a particular credential that may be injected into a bundle
#[derive(Debug, Serialize, Deserialize)]
pub struct Credential {
    /// The description of this credential
    pub description: Option<String>,
    /// The name of the environment variable into which the value will be placed
    pub env: Option<String>,
    /// The fully qualified path into which the value will be placed
    pub path: Option<PathBuf>,
}

/// Parameter describes a parameter that will be put into the invocation image
///
/// Paramters are injected into the invocation image at startup time
#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    /// The actions to which this parameter applies.
    ///
    /// If unset, this parameter will be applied to all actions.
    #[serde(rename = "applyTo")]
    pub apply_to: Option<Vec<String>>,
    /// The location where this parameter will be injected in the invocation image
    pub destination: Destination,
    /// This parameter's default value
    #[serde(rename = "defaultValue")]
    pub default_value: Option<serde_json::Value>,

    /// An enumeration of allowed values
    #[serde(rename = "enum")]
    pub allowed_values: Option<Vec<serde_json::Value>>,
    /// alphabetically, this is 'enum'
    /// The exclusive maximum.
    ///
    /// If unspecified, no exclusive max is applied
    #[serde(rename = "exclusiveMaximum")]
    pub exclusive_maximum: Option<i64>,
    /// The exclusive minimum.
    ///
    /// If unspecified, no exclusive min is applied
    #[serde(rename = "exclusiveMinimum")]
    pub exclusive_minimum: Option<i64>,
    /// The maximum
    ///
    /// If unspecified, the maximum 64-bit integer value is applied
    pub maximum: Option<i64>,
    /// The maximum length of a string value
    ///
    /// If unspecified, no max is applied.
    #[serde(rename = "maxLength")]
    pub max_length: Option<i64>,
    /// Additional parameter information
    pub metadata: Option<Metadata>,
    /// The minimum integer value
    ///
    /// If unspecified, the minimum 64-bit integer value is applied
    pub minimum: Option<i64>,
    /// The minimum string length
    #[serde(rename = "minLength")]
    pub min_length: Option<i64>,
    /// A regular expression (as defined in ECMAScript)
    ///
    /// If it is not matched, a string parameter value will be rejected
    pub pattern: Option<String>,
    /// Indicate whether this parameter is required
    ///
    /// Default is false.
    #[serde(default)]
    pub required: bool,
    /// This describes the underlying type of the parameter (string, int...)
    #[serde(rename = "type")]
    pub parameter_type: String, // Should be Enum; alphabetically, this is 'type'
}

/// An Action is a custom action in an invocation image.
///
/// For example, an invocation image may provide help text by creating a 'help'
/// action that, when triggered, prints help text to STDOUT.
#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    /// Describes what this action does
    pub description: Option<String>,
    /// If true, this action modifies the deployment, and should be tracked as a release.
    #[serde(default)]
    pub modifies: bool,
    /// If true, this action does not require any state information to be injected
    ///
    /// For example, printing help text does not require an installation, credentials,
    /// or parameters.
    #[serde(default)]
    pub stateless: bool,
}

/// Describe a parameter
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    /// A description of a parameter
    pub description: Option<String>,
}

/// Destination describes where, in the invocation image, a particular parameter value should be
/// placed.
///
/// A parameter value can be placed into an environment variable (`env`) or a file at
/// a particular location on the filesystem (`path`). This is a non-exclusive or, meaning
/// that the same paramter can be written to both an env var and a path.
#[derive(Debug, Serialize, Deserialize)]
pub struct Destination {
    /// The name of the destination environment variable
    pub env: Option<String>,
    /// The fully qualified path to the destination file
    pub path: Option<PathBuf>,
}
