use super::packet::PacketFrame;
use anyhow::{Context, Result};
use pcap_file::{
    DataLink,
    pcapng::{
        PcapNgWriter,
        blocks::{
            enhanced_packet::EnhancedPacketBlock,
            interface_description::{InterfaceDescriptionBlock, InterfaceDescriptionOption},
        },
    },
};
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::Path,
};

pub struct Recording<W: Write> {
    writer: PcapNgWriter<W>,
    interfaces: HashMap<(u32, bool), u32>,
}

pub fn create(path: &Path) -> Result<Recording<BufWriter<std::fs::File>>> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path).with_context(|| {
        format!(
            "cannot create capture file {} (existing files are never overwritten)",
            path.display()
        )
    })?;
    Recording::new(BufWriter::new(file))
}

impl<W: Write> Recording<W> {
    pub fn new(writer: W) -> Result<Self> {
        Ok(Self {
            writer: PcapNgWriter::new(writer)?,
            interfaces: HashMap::new(),
        })
    }

    pub fn register_interface(&mut self, index: u32, name: &str, raw_ip: bool) -> Result<u32> {
        let key = (index, raw_ip);
        if let Some(&id) = self.interfaces.get(&key) {
            return Ok(id);
        }
        let id = u32::try_from(self.interfaces.len()).context("too many recording interfaces")?;
        self.writer.write_pcapng_block(InterfaceDescriptionBlock {
            linktype: if raw_ip {
                DataLink::RAW
            } else {
                DataLink::ETHERNET
            },
            // Use an explicit limit for libpcap readers that reject zero snaplen.
            snaplen: u32::MAX,
            options: vec![
                InterfaceDescriptionOption::IfName(Cow::Borrowed(name)),
                // pcap-file encodes EnhancedPacketBlock timestamps as nanoseconds.
                InterfaceDescriptionOption::IfTsResol(9),
            ],
        })?;
        self.interfaces.insert(key, id);
        Ok(id)
    }

    pub fn write_packet(&mut self, packet: &PacketFrame) -> Result<()> {
        let interface_id =
            self.register_interface(packet.if_index, &packet.if_name, packet.raw_ip)?;
        self.writer.write_pcapng_block(EnhancedPacketBlock {
            interface_id,
            timestamp: packet.captured_at,
            original_len: u32::try_from(packet.raw_data.len())
                .context("capture packet exceeds PCAPNG size limit")?,
            data: Cow::Borrowed(&packet.raw_data),
            options: vec![],
        })?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<W> {
        self.writer
            .get_mut()
            .flush()
            .context("failed to flush capture file")?;
        Ok(self.writer.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcap_file::pcapng::{Block, PcapNgReader};
    #[test]
    fn recording_round_trips_interfaces_timestamps_and_exact_packet_bytes() {
        let mut recording = Recording::new(Vec::new()).unwrap();
        for id in [1, 2, 1] {
            recording
                .write_packet(&PacketFrame {
                    if_index: id,
                    if_name: format!("test{id}"),
                    raw_ip: id == 2,
                    raw_data: bytes::Bytes::from_static(&[0x45, 0, 0, 3, 1]),
                    captured_at: std::time::Duration::new(1_700_000_000, 123_456_789),
                    ..Default::default()
                })
                .unwrap();
        }
        let data = recording.finish().unwrap();
        let mut reader = PcapNgReader::new(data.as_slice()).unwrap();
        let mut interfaces = vec![];
        let mut packets = vec![];
        while let Some(block) = reader.next_block() {
            match block.unwrap() {
                Block::InterfaceDescription(interface) => {
                    assert!(interface.snaplen > 0);
                    assert!(
                        interface
                            .options
                            .contains(&InterfaceDescriptionOption::IfTsResol(9))
                    );
                    interfaces.push(interface.linktype);
                }
                Block::EnhancedPacket(packet) => {
                    assert_eq!(
                        packet.timestamp,
                        std::time::Duration::new(1_700_000_000, 123_456_789)
                    );
                    assert_eq!(packet.data.as_ref(), &[0x45, 0, 0, 3, 1]);
                    packets.push(packet.interface_id);
                }
                _ => {}
            }
        }
        assert_eq!(interfaces, [DataLink::ETHERNET, DataLink::RAW]);
        assert_eq!(packets, [0, 1, 0]);
    }
    #[test]
    fn write_failures_are_reported() {
        struct FailedWriter;
        impl Write for FailedWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("disk full"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        assert!(Recording::new(FailedWriter).is_err());
    }
    #[test]
    fn flush_failures_are_reported() {
        struct FlushFailure;
        impl Write for FlushFailure {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::other("flush failed"))
            }
        }
        assert!(Recording::new(FlushFailure).unwrap().finish().is_err());
    }

    #[test]
    fn existing_files_are_never_overwritten() {
        let path = std::env::temp_dir().join(format!(
            "ntap-recording-test-{}-{}.pcapng",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut original = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        original.write_all(b"preserve existing capture").unwrap();
        drop(original);
        assert!(create(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"preserve existing capture");
        std::fs::remove_file(path).unwrap();
    }
}
