use base64::write::EncoderWriter;
use std::collections::HashMap;
use std::io::Write;
use sysinfo::{ProcessExt, System, SystemExt};

use crate::error::ApiError;

#[derive(Debug)]
pub struct Auth {
    port: String,
    key: String,
}

const PORT_ARG_NAME: &str = "app-port";
const KEY_ARG_NAME: &str = "remoting-auth-token";

impl Auth {
    pub fn new() -> Result<Self, ApiError> {
        let os = Os::get_os();
        let proc_name = os.get_proc_name();
        let system = System::new_all();
        let proc = system
            .get_processes()
            .iter()
            .find_map(|(_pid, proc)| {
                if proc_name == proc.name() {
                    Some(proc)
                } else {
                    None
                }
            })
            .ok_or(ApiError::NotRunning)?;
        let args_map = Self::parse_cmd(proc.cmd());
        Ok(Self {
            port: args_map
                .get(PORT_ARG_NAME)
                .expect("could not find port in arg list")
                .to_string(),
            key: args_map
                .get(KEY_ARG_NAME)
                .expect("could not find key in arg list")
                .to_string(),
        })
    }

    pub fn basic_auth_token(&self) -> String {
        let mut token = b"Basic ".to_vec();
        {
            let mut encoder = EncoderWriter::new(&mut token, base64::STANDARD);
            write!(encoder, "riot:{}", self.key).expect("could not write to b64 encoder");
        }
        String::from_utf8(token).expect("invalid utf8")
    }

    pub fn port(&self) -> String {
        self.port.clone()
    }

    fn parse_cmd(args: &[String]) -> HashMap<&str, &str> {
        let mut args_map = HashMap::new();
        for arg in args {
            if arg.contains('=') {
                let mut parts = arg.trim_start_matches('-').split('=');
                args_map.insert(
                    parts.next().expect("could not find cmd arg key"),
                    parts.next().expect("could not find cmd arg value"),
                );
            }
        }
        args_map
    }
}

#[derive(Debug)]
enum Os {
    Windows,
    Macos,
}

const WINDOWS_PROC_NAME: &str = "LeagueClientUx.exe";
const MACOS_PROC_NAME: &str = "LeagueClientUx";

impl Os {
    fn get_os() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            panic!("unsupported os")
        }
    }

    fn get_proc_name(&self) -> &str {
        match self {
            Self::Macos => MACOS_PROC_NAME,
            Self::Windows => WINDOWS_PROC_NAME,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn get_os() {
        use super::Os;
        println!("{:?}", Os::get_os());
    }

    #[test]
    fn new_auth() {
        use super::Auth;
        println!("{:?}", Auth::new().unwrap());
    }
}
