
use libc::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::borrow::Cow;
use std::ptr;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::fmt;
use openssl;
use bindings::*;
use openssl::sign::Verifier;
use openssl::pkey::PKey;
use openssl::hash::MessageDigest;
use toml;
use toml::Value;
use uuid::Uuid;


#[no_mangle]
pub fn pam_authenticate(
    pamh: *mut pam_handle_t,
    flags: c_int,
    _: c_int,
    _: *mut *const c_char,
) -> c_int {
    authenticate_via_hello(pamh).unwrap_or_else(|err| {
        if (flags & PAM_SILENT) == 0 {
            println!("WSL Hello error: {}", err);
        }
        match err {
            HelloAuthenticationError::PublicKeyFileError(ref err)
                if err.kind() == io::ErrorKind::NotFound =>
            {
                PAM_USER_UNKNOWN
            }
            HelloAuthenticationError::AuthenticatorLaunchError(_) => PAM_AUTHINFO_UNAVAIL,
            HelloAuthenticationError::AuthenticatorConnectionError(_) => PAM_AUTHINFO_UNAVAIL,
            HelloAuthenticationError::AuthenticatorSignalled => PAM_AUTHINFO_UNAVAIL,
            _ => PAM_AUTH_ERR,
        }
    })
}

fn get_user(pamh: *mut pam_handle_t, prompt: Option<&str>) -> Result<Cow<str>, i32> {
    let mut c_user: *const c_char = ptr::null();
    let c_prompt = match prompt {
        Some(prompt_str) => CString::new(prompt_str).unwrap().as_ptr(),
        None => ptr::null(),
    };
    let err;
    unsafe {
        err = pam_get_user(pamh, &mut c_user, c_prompt);
    }
    match err {
        PAM_SUCCESS => unsafe {
            let c_user_str = CStr::from_ptr(c_user);
            Ok(c_user_str.to_string_lossy())
        },
        err => Err(err),
    }
}

#[derive(Debug)]
enum ConfigError {
    Io(io::Error),
    Toml(toml::de::Error),
    MissingField(String),
    InvalidValueType(String),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::Io(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> ConfigError {
        ConfigError::Toml(err)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Io(ref ioerr) => write!(f, "{}", ioerr),
            ConfigError::Toml(_) => write!(f, "TOML format error"),
            ConfigError::MissingField(_) => write!(f, "field: 'authenticator_path' is not found"),
            ConfigError::InvalidValueType(_) => {
                write!(f, "field: 'authenticator_path' has an invalid value type")
            }
        }
    }
}

fn get_authenticator_path() -> Result<String, ConfigError> {
    let mut config_file = File::open("/etc/hello_pam/config")?;
    let mut config = String::new();
    config_file.read_to_string(&mut config)?;

    let config_value = config.parse::<Value>()?;
    let authenticator_path = config_value
        .get("authenticator_path")
        .ok_or(ConfigError::MissingField("authenticator_path".to_owned()))?
        .as_str()
        .ok_or(ConfigError::InvalidValueType(
            "authenticator_path".to_owned(),
        ))?;
    Ok(authenticator_path.to_owned())
}

#[derive(Debug)]
enum HelloAuthenticationError {
    GetUserError(i32),
    ConfigError(ConfigError),
    PublicKeyFileError(io::Error),
    Io(io::Error),
    InvalidPublicKey(openssl::error::ErrorStack),
    OpenSSLError(openssl::error::ErrorStack),
    AuthenticatorLaunchError(io::Error),
    AuthenticatorConnectionError(io::Error),
    AuthenticatorSignalled,
    HelloAuthenticationFail(String),
    SignAuthenticationFail,
}

impl From<io::Error> for HelloAuthenticationError {
    fn from(err: io::Error) -> HelloAuthenticationError {
        HelloAuthenticationError::Io(err)
    }
}

impl From<ConfigError> for HelloAuthenticationError {
    fn from(err: ConfigError) -> HelloAuthenticationError {
        HelloAuthenticationError::ConfigError(err)
    }
}

impl fmt::Display for HelloAuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HelloAuthenticationError::ConfigError(ref err) => write!(f, "config error; {}", err),
            HelloAuthenticationError::PublicKeyFileError(ref err) => match err.kind() {
                io::ErrorKind::NotFound => {
                    write!(f, "cannot find user public key")
                }
                _ => write!(f, "{}", err),
            },
            HelloAuthenticationError::Io(ref err) => write!(f, "{}", err),
            HelloAuthenticationError::InvalidPublicKey(_) => {
                write!(f, "public key is invalid")
            }
            HelloAuthenticationError::AuthenticatorLaunchError(ref err) => {
                write!(f, "cannot launch Windows Hello; {}", err)
            }
            HelloAuthenticationError::AuthenticatorConnectionError(ref err) => {
                write!(f, "cannot communicate with Windows Hello; {}", err)
            }
            HelloAuthenticationError::HelloAuthenticationFail(ref msg) => {
                write!(f, "authentication failed; {}", msg)
            }
            HelloAuthenticationError::SignAuthenticationFail => write!(
                f,
                "signature verification failed"
            ),
            ref err => write!(f, "internal error; {:?}", err),
        }
    }
}

fn authenticate_via_hello(pamh: *mut pam_handle_t) -> Result<i32, HelloAuthenticationError> {
    let user_name = get_user(pamh, None).map_err(|e| HelloAuthenticationError::GetUserError(e))?;
    let credentialFile = format!("hello_pam_{}", user_name);

    let mut hello_public_key_file =
        File::open(format!(
            "/etc/hello_pam/public_keys/{}.pem",
            credentialFile
        )).map_err(|io| HelloAuthenticationError::PublicKeyFileError(io))?;
    let mut key_str = String::new();
    hello_public_key_file.read_to_string(&mut key_str)?;
    let hello_public_key = PKey::public_key_from_pem(key_str.as_bytes())
        .map_err(|e| HelloAuthenticationError::InvalidPublicKey(e))?;

    let challenge = format!("hello_pam:{}:{}", user_name, Uuid::new_v4());

    let auth_res;
    let challenge_tmpfile_path = &format!("/tmp/{}", challenge);
    {
        let mut challenge_tmpfile = OpenOptions::new()
            .write(true)
            .read(true)
            .create_new(true)
            .open(challenge_tmpfile_path)?;
        challenge_tmpfile.write_all(challenge.as_bytes())?;
        challenge_tmpfile.seek(SeekFrom::Start(0))?;
        let challenge_tmpfile_in = unsafe { Stdio::from_raw_fd(challenge_tmpfile.as_raw_fd()) };

        let authenticator_path = get_authenticator_path()?;
        let authenticator = Command::new(&authenticator_path)
            .arg(credentialFile)
            .current_dir("/mnt/c")
            .stdin(challenge_tmpfile_in)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| HelloAuthenticationError::AuthenticatorLaunchError(e))?;

        auth_res = authenticator.wait_with_output().map_err(|e| {
            HelloAuthenticationError::AuthenticatorConnectionError(e)
        })?;
    }
    fs::remove_file(challenge_tmpfile_path)?;

    match auth_res.status.code() {
        Some(code) if code == 0 => {}
        Some(_) => {
            return Err(HelloAuthenticationError::HelloAuthenticationFail(
                String::from_utf8(auth_res.stdout).unwrap_or("invalid utf8 output".to_string()),
            ))
        }
        None => return Err(HelloAuthenticationError::AuthenticatorSignalled),
    }
    let signature = auth_res.stdout;

    let mut verifier = Verifier::new(MessageDigest::sha256(), &hello_public_key).unwrap();
    verifier
        .update(challenge.as_bytes())
        .map_err(|e| HelloAuthenticationError::OpenSSLError(e))?;

    match verifier
        .verify(&signature)
        .map_err(|e| HelloAuthenticationError::OpenSSLError(e))?
    {
        true => Ok(PAM_SUCCESS),
        false => Err(HelloAuthenticationError::SignAuthenticationFail),
    }
}
