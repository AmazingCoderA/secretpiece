//! Minimal blocking FTP client used by the Android bridge: one file, short
//! timeouts, passive mode. Kept here so the phone can use the same server
//! presets as the desktop without shipping a shell or an SSH binary.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

pub(crate) struct Ftp {
    stream: TcpStream,
}

impl Ftp {
    pub(crate) fn connect(
        host: &str,
        port: &str,
        user: &str,
        password: &str,
    ) -> Result<Self, String> {
        if host.is_empty() {
            return Err("host is empty".to_owned());
        }
        let address = format!("{host}:{port}");
        let socket = address
            .to_socket_addrs()
            .map_err(|error| format!("{address}: {error}"))?
            .next()
            .ok_or_else(|| format!("{address}: no address"))?;
        let stream = TcpStream::connect_timeout(&socket, Duration::from_secs(4))
            .map_err(|error| format!("{address}: {error}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(8)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(8)))
            .map_err(|e| e.to_string())?;
        let mut ftp = Self { stream };
        ftp.expect(&[220])?;
        let user = if user.is_empty() { "anonymous" } else { user };
        let password = if password.is_empty() {
            "anonymous@"
        } else {
            password
        };
        let (code, _) = ftp.command(&format!("USER {user}"))?;
        if code == 331 {
            ftp.command(&format!("PASS {password}"))?;
        }
        ftp.command("TYPE I")?;
        Ok(ftp)
    }

    fn line(&mut self) -> Result<String, String> {
        let mut line = Vec::new();
        let mut byte = [0_u8; 1];
        loop {
            let read = self.stream.read(&mut byte).map_err(|e| e.to_string())?;
            if read == 0 {
                return Err("FTP: connection closed".to_owned());
            }
            if byte[0] == b'\n' {
                break;
            }
            if byte[0] != b'\r' {
                line.push(byte[0]);
            }
        }
        Ok(String::from_utf8_lossy(&line).into_owned())
    }

    fn reply(&mut self) -> Result<(u16, String), String> {
        let mut text = String::new();
        loop {
            let line = self.line()?;
            let code = line
                .get(..3)
                .and_then(|code| code.parse::<u16>().ok())
                .ok_or_else(|| format!("FTP: {line}"))?;
            let multiline = line.as_bytes().get(3) == Some(&b'-');
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(line.trim_end());
            if !multiline {
                return Ok((code, text));
            }
        }
    }

    fn command(&mut self, command: &str) -> Result<(u16, String), String> {
        self.stream
            .write_all(format!("{command}\r\n").as_bytes())
            .map_err(|e| e.to_string())?;
        let (code, text) = self.reply()?;
        if code >= 400 {
            return Err(text);
        }
        Ok((code, text))
    }

    fn expect(&mut self, expected: &[u16]) -> Result<(), String> {
        let (code, text) = self.reply()?;
        if expected.contains(&code) {
            Ok(())
        } else {
            Err(text)
        }
    }

    fn pasv(&mut self) -> Result<SocketAddr, String> {
        use std::net::{IpAddr, Ipv4Addr};

        let (code, text) = self.command("PASV")?;
        if code != 227 {
            return Err(format!("FTP: PASV {code}"));
        }
        let start = text.find('(').ok_or_else(|| "FTP: bad PASV".to_owned())?;
        let end = text[start..]
            .find(')')
            .ok_or_else(|| "FTP: bad PASV".to_owned())?;
        let numbers: Vec<u16> = text[start + 1..start + end]
            .split(',')
            .filter_map(|part| part.trim().parse().ok())
            .collect();
        if numbers.len() != 6 {
            return Err("FTP: bad PASV".to_owned());
        }
        let octets = [
            u8::try_from(numbers[0]).map_err(|_| "FTP: bad PASV".to_owned())?,
            u8::try_from(numbers[1]).map_err(|_| "FTP: bad PASV".to_owned())?,
            u8::try_from(numbers[2]).map_err(|_| "FTP: bad PASV".to_owned())?,
            u8::try_from(numbers[3]).map_err(|_| "FTP: bad PASV".to_owned())?,
        ];
        let port = numbers[4] * 256 + numbers[5];
        Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(octets)), port))
    }

    fn data(&mut self) -> Result<TcpStream, String> {
        let address = self.pasv()?;
        let stream = TcpStream::connect_timeout(&address, Duration::from_secs(4))
            .map_err(|e| e.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(8)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(8)))
            .map_err(|e| e.to_string())?;
        Ok(stream)
    }

    pub(crate) fn stor(&mut self, remote: &str, bytes: &[u8]) -> Result<u64, String> {
        let mut data = self.data()?;
        let (code, _) = self.command(&format!("STOR {remote}"))?;
        if code != 150 && code != 125 {
            return Err(format!("FTP: STOR {code}"));
        }
        data.write_all(bytes).map_err(|e| e.to_string())?;
        data.shutdown(std::net::Shutdown::Write)
            .map_err(|e| e.to_string())?;
        drop(data);
        let (code, text) = self.reply()?;
        if code != 226 && code != 250 {
            return Err(text);
        }
        let size = self
            .command(&format!("SIZE {remote}"))
            .ok()
            .and_then(|(code, text)| {
                if code == 213 {
                    text.split_whitespace().next()?.parse().ok()
                } else {
                    None
                }
            });
        Ok(size.unwrap_or(bytes.len() as u64))
    }

    pub(crate) fn retr(&mut self, remote: &str) -> Result<Vec<u8>, String> {
        let mut data = self.data()?;
        let (code, _) = self.command(&format!("RETR {remote}"))?;
        if code != 150 && code != 125 {
            return Err(format!("FTP: RETR {code}"));
        }
        let mut bytes = Vec::new();
        data.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        drop(data);
        let (code, text) = self.reply()?;
        if code == 226 || code == 250 {
            Ok(bytes)
        } else {
            Err(text)
        }
    }
}
