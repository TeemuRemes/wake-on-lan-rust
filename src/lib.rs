//! A library for creating and sending Wake-on-LAN magic packets.
//!
//! # Usage
//!
//! ```
//! use wake_on_lan;
//!
//! // The MAC address of the target device
//! let mac_address: [u8; 6] = [0x0F, 0x1E, 0x2D, 0x3C, 0x4B, 0x5A];
//!
//! // Create a magic packet (but don't send it yet)
//! let magic_packet = wake_on_lan::MagicPacket::new(&mac_address);
//!
//! // Send the magic packet via UDP to the broadcast address 255.255.255.255:9 from 0.0.0.0:0
//! magic_packet.send()?;
//! ```
//!
//! To choose the source and destination IPs and ports, use `send_to()`. If you want to access the
//! contents of the magic packet, use `magic_bytes()`.

use std::convert::TryFrom;
use std::convert::TryInto;
use std::net::{Ipv4Addr, ToSocketAddrs, UdpSocket};

/// A Wake-on-LAN magic packet.
pub struct MagicPacket {
    magic_bytes: [u8; 102]
}

impl MagicPacket {
    
    /// Creates a new `MagicPacket` intended for `mac_address` (but doesn't send it yet).
    pub fn new(mac_address: &[u8; 6]) -> MagicPacket {
        let mut magic_bytes: [u8; 102];
        
        // We use `unsafe` code to skip unnecessary array initialization and bounds checking.
        unsafe {
            magic_bytes = std::mem::uninitialized();
            
            // Copy the header to the beginning.
            let mut src: *const u8 = &MAGIC_BYTES_HEADER[0];
            let mut dst: *mut u8 = &mut magic_bytes[0];
            dst.copy_from_nonoverlapping(src, 6);
            
            // Copy the MAC address once from the argument.
            src = &mac_address[0];
            dst = dst.offset(6);
            dst.copy_from_nonoverlapping(src, 6);

            // Repeat the MAC.
            let src: *const u8 = dst; // src points to magic_bytes[6]
            dst = dst.offset(6);
            dst.copy_from_nonoverlapping(src, 6);
            
            dst = dst.offset(6);
            dst.copy_from_nonoverlapping(src, 12);
            
            dst = dst.offset(12);
            dst.copy_from_nonoverlapping(src, 24);
            
            dst = dst.offset(24);
            dst.copy_from_nonoverlapping(src, 48);
        }
        
        MagicPacket { magic_bytes }
    }
    
    /// Sends the magic packet via UDP to the broadcast address `255.255.255.255:9`.
    /// Lets the operating system choose the source port and network interface.
    pub fn send(&self) -> std::io::Result<()> {
        self.send_to(
            (Ipv4Addr::new(255, 255, 255, 255), 9),
            (Ipv4Addr::new(0, 0, 0, 0), 0)
        )
    }
    
    /// Sends the magic packet via UDP to/from an IP address and port number of your choosing.
    pub fn send_to<A: ToSocketAddrs>(&self, to_addr: A, from_addr: A) -> std::io::Result<()> {
        let socket = UdpSocket::bind(from_addr)?;
        socket.set_broadcast(true)?;
        socket.send_to(&self.magic_bytes, to_addr)?;
        
        Ok(())
    }
    
    /// Returns the magic packet's payload (6 repetitions of `0xFF` and 16 repetitions of the 
    /// target device's MAC address). Send these bytes yourself over the network if you want to do 
    /// something more advanced (like reuse a single UDP socket when sending a large number of 
    /// magic packets).
    pub fn magic_bytes(&self) -> &[u8; 102] {
        &self.magic_bytes
    }
}

const MAGIC_BYTES_HEADER: [u8; 6] = [0xFF; 6];

impl TryFrom<&str> for MagicPacket {
    type Error = &'static str;

    fn try_from(mac_string: &str) -> Result<MagicPacket, Self::Error> {
        let mac_address = mac_string
            .split(":")
            .flat_map(|hex| u8::from_str_radix(hex, 16))
            .collect::<Vec<u8>>()
            .try_into()
            .map_err(|_| "Unable to parse MAC address")?;

        Ok(Self::new(&mac_address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_from_valid() {
        let magic_packet: Result<MagicPacket, &'static str> = "00:11:22:33:44:AA".try_into();
        assert!(magic_packet.is_ok());
    }

    #[test]
    fn try_from_too_short() {
        let magic_packet: Result<MagicPacket, &'static str> = "00:11:22:33:44".try_into();
        assert!(magic_packet.is_err());
    }

    #[test]
    fn try_from_invalid_u8() {
        let magic_packet: Result<MagicPacket, &'static str> = "00:11:22:33:44:XX".try_into();
        assert!(magic_packet.is_err());
    }

    #[test]
    fn try_from_lowercase() {
        let magic_packet: Result<MagicPacket, &'static str> = "00:11:22:33:44:aa".try_into();
        assert!(magic_packet.is_ok());
    }
}
