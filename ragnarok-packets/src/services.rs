//! Native PACKETVER 20220406 mail and companion packets, matched to rAthena.
use super::*;

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x0AC0)]
pub struct OpenMailboxPacket {
    pub character_cursor: u64,
    pub returned_cursor: u64,
    pub account_cursor: u64,
}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x0AC1)]
pub struct RefreshMailboxPacket {
    pub cursor: u64,
    pub reserved: [u8; 16],
}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x09EA)]
pub struct ReadMailPacket {
    pub inbox: u8,
    pub mail_id: u64,
}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x09F1)]
pub struct ClaimMailZenyPacket {
    pub mail_id: u64,
    pub inbox: u8,
}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x09F3)]
pub struct ClaimMailItemsPacket {
    pub mail_id: u64,
    pub inbox: u8,
}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x09F5)]
pub struct DeleteMailPacket {
    pub inbox: u8,
    pub mail_id: u64,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x0AC2)]
#[variable_length]
pub struct MailboxListPacket {
    pub complete: u8,
    #[repeating_remaining]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x0B63)]
#[variable_length]
pub struct MailBodyPacket {
    pub inbox: u8,
    pub mail_id: u64,
    pub text_length: u16,
    pub zeny: u64,
    pub item_count: u8,
    #[repeating_remaining]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x09F2)]
pub struct MailZenyResultPacket {
    pub mail_id: u64,
    pub inbox: u8,
    pub result: u8,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x09F4)]
pub struct MailItemsResultPacket {
    pub mail_id: u64,
    pub inbox: u8,
    pub result: u8,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x09F6)]
pub struct MailDeletedPacket {
    pub inbox: u8,
    pub mail_id: u64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
pub struct MailSummary {
    pub inbox: u8,
    pub id: u64,
    pub read: bool,
    pub attachment_flags: u8,
    pub sender: String,
    pub title: String,
}

impl MailboxListPacket {
    pub fn messages(&self) -> Result<Vec<MailSummary>, &'static str> {
        let mut data = self.data.as_slice();
        let mut messages = Vec::new();
        while !data.is_empty() {
            if data.len() < 41 {
                return Err("Incomplete mailbox entry");
            }
            let length = u16::from_le_bytes([data[39], data[40]]) as usize;
            if length == 0 || length > 256 || data.len() < 41 + length {
                return Err("Invalid mail title length");
            }
            let text = |bytes: &[u8]| String::from_utf8_lossy(bytes.split(|b| *b == 0).next().unwrap_or_default()).into_owned();
            messages.push(MailSummary {
                inbox: data[0],
                id: u64::from_le_bytes(data[1..9].try_into().unwrap()),
                read: data[9] != 0,
                attachment_flags: data[10],
                sender: text(&data[11..35]),
                title: text(&data[41..41 + length]),
            });
            data = &data[41 + length..];
        }
        Ok(messages)
    }
}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x01A1)]
pub struct PetMenuPacket {
    pub action: u8,
}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x01A7)]
pub struct HatchPetPacket {
    pub inventory_index: u16,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x01A2)]
pub struct PetInformationPacket {
    #[length(24)]
    pub name: String,
    pub renamed: u8,
    pub level: u16,
    pub hunger: u16,
    pub intimacy: u16,
    pub accessory: u16,
    pub job: u16,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x01A4)]
pub struct PetUpdatePacket {
    pub kind: u8,
    pub entity: EntityId,
    pub value: u32,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x01A6)]
#[variable_length]
pub struct PetEggsPacket {
    #[repeating_remaining]
    pub indices: Vec<u16>,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x01A3)]
pub struct PetFeedResultPacket {
    pub success: u8,
    pub item_id: u32,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x01A0)]
pub struct PetCaptureResultPacket {
    pub success: u8,
}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x0439)]
pub struct UseInventoryItemPacket {
    pub index: InventoryIndex,
    pub target: EntityId,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x01C8)]
pub struct InventoryItemUsedPacket {
    pub index: InventoryIndex,
    pub item_id: u32,
    pub entity: EntityId,
    pub remaining: u16,
    pub success: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mailbox_decodes_multiple_variable_titles_without_losing_entry_boundaries() {
        let mut data = Vec::new();
        for (id, title) in [(12u64, "Auction payout"), (15, "Returned item")] {
            data.push(0);
            data.extend_from_slice(&id.to_le_bytes());
            data.extend_from_slice(&[0, 3]);
            let mut sender = [0; 24];
            sender[..7].copy_from_slice(b"Auction");
            data.extend_from_slice(&sender);
            data.extend_from_slice(&3600u32.to_le_bytes());
            data.extend_from_slice(&((title.len() + 1) as u16).to_le_bytes());
            data.extend_from_slice(title.as_bytes());
            data.push(0);
        }
        let list = MailboxListPacket::new(1, data);
        let messages = list.messages().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[1].id, 15);
        assert_eq!(messages[1].title, "Returned item");
        let mut broken = list.clone();
        broken.data.pop();
        assert!(broken.messages().is_err());
    }
    #[test]
    fn attachment_requests_preserve_64_bit_mail_id_and_wire_field_order() {
        let mut writer = ByteWriter::new();
        ClaimMailItemsPacket::new(0x0102030405060708, 2)
            .packet_to_bytes(&mut writer)
            .unwrap();
        assert_eq!(writer.as_slice(), &[0xF3, 0x09, 8, 7, 6, 5, 4, 3, 2, 1, 2]);
    }
}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x01A5)]
pub struct RenamePetPacket {
    #[length(24)]
    pub name: String,
}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x02D8)]
pub struct ChangeConfigurationPacket {
    pub config_type: u32,
    pub value: u32,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x019E)]
pub struct BeginPetCapturePacket {}

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x019F)]
pub struct CapturePetPacket {
    pub target: EntityId,
}
