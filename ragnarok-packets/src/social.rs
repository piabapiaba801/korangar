//! Native social and direct-trade protocol for PACKETVER 20220406.
use super::*;
macro_rules! client_packet {
    ($name:ident, $header:literal, { $($fields:tt)* }) => {
        #[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
        #[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
        #[header($header)] pub struct $name { $($fields)* }
    };
}
macro_rules! server_packet {
    ($name:ident, $header:literal, { $($fields:tt)* }) => {
        #[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
        #[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
        #[header($header)] pub struct $name { $($fields)* }
    };
}
client_packet!(CreatePartyPacket, 0x01E8, { #[length(24)] pub name: String, pub pickup: u8, pub share: u8 });
client_packet!(InvitePartyPacket, 0x02C4, { #[length(24)] pub name: String });
client_packet!(ReplyPartyInvitePacket, 0x02C7, { pub party_id: u32, pub accept: u8 });
client_packet!(LeavePartyPacket, 0x0100, {});
server_packet!(PartyCreatedPacket, 0x00FA, { pub result: u8 });
server_packet!(PartyInviteResultPacket, 0x02C5, { #[length(24)] pub name: String, pub result: u32 });
client_packet!(GuildPermissionsRequestPacket, 0x014D, {});
client_packet!(GuildInfoRequestPacket, 0x014F, { pub kind: u32 });
client_packet!(CreateGuildPacket, 0x0165, { pub account_id: AccountId, #[length(24)] pub name: String });
client_packet!(InviteGuildPacket, 0x0916, { #[length(24)] pub name: String });
client_packet!(ReplyGuildInvitePacket, 0x016B, { pub guild_id: u32, pub accept: u32 });
client_packet!(LeaveGuildPacket, 0x0159, { pub guild_id: u32, pub account_id: AccountId, pub character_id: CharacterId, #[length(40)] pub message: String });
server_packet!(GuildPermissionsPacket, 0x014E, { pub permission: u32 });
server_packet!(GuildMembershipPacket, 0x02F7, {
    pub guild_id: u32, pub emblem_id: u32, pub mode: u32, pub master: u8, pub server_id: u32,
    #[length(24)] pub name: String, pub master_character: CharacterId
});
server_packet!(GuildCreatedPacket, 0x0167, { pub result: u8 });
server_packet!(GuildInviteResultPacket, 0x0169, { pub result: u8 });
server_packet!(GuildInvitePacket, 0x016A, { pub guild_id: u32, #[length(24)] pub name: String });
server_packet!(GuildNoticePacket, 0x016F, { #[length(60)] pub subject: String, #[length(120)] pub notice: String });
server_packet!(GuildInformationPacket, 0x0B7B, {
    pub guild_id: u32, pub level: u32, pub online: u32, pub capacity: u32, pub average_level: u32,
    pub experience: u32, pub next_experience: u32, pub points: u32, pub honor: u32, pub virtue: u32, pub emblem_id: u32,
    #[length(24)] pub name: String, #[length(16)] pub territory: String, pub zeny: u32,
    pub master_character: CharacterId, #[length(24)] pub master_name: String
});
#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x0B7D)]
#[variable_length]
pub struct GuildMembersPacket {
    #[repeating_remaining]
    pub data: Vec<u8>,
}

macro_rules! guild_aux_packet {
    ($name:ident, $header:literal) => {
        #[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
        #[cfg_attr(
            feature = "interface",
            derive(rust_state::RustState, korangar_interface::element::StateElement)
        )]
        #[header($header)]
        #[variable_length]
        pub struct $name {
            #[repeating_remaining]
            pub data: Vec<u8>,
        }
    };
}
guild_aux_packet!(GuildAlliancesPacket, 0x014C);
guild_aux_packet!(GuildPositionsPacket, 0x0166);
guild_aux_packet!(GuildCastlesPacket, 0x0B27);

#[derive(Debug, Clone, Packet, ClientPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x017E)]
#[variable_length]
pub struct GuildChatRequestPacket {
    #[repeating_remaining]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Packet, ServerPacket, MapServer)]
#[cfg_attr(feature = "interface", derive(rust_state::RustState, korangar_interface::element::StateElement))]
#[header(0x017F)]
#[variable_length]
pub struct GuildChatPacket {
    #[repeating_remaining]
    pub data: Vec<u8>,
}

client_packet!(RequestTradePacket, 0x00E4, { pub account_id: AccountId });
client_packet!(ReplyTradePacket, 0x00E6, { pub result: u8 });
client_packet!(AddTradeItemPacket, 0x00E8, { pub index: u16, pub amount: u32 });
client_packet!(LockTradePacket, 0x00EB, {});
client_packet!(CancelTradePacket, 0x00ED, {});
client_packet!(ConfirmTradePacket, 0x00EF, {});
server_packet!(GuildMemberLeftPacket, 0x0A83, { pub character_id: CharacterId, #[length(40)] pub reason: String });
server_packet!(GuildMemberExpelledPacket, 0x0A82, { pub character_id: CharacterId, #[length(40)] pub reason: String });
server_packet!(GuildMemberOnlinePacket, 0x01F2, { pub account_id: AccountId, pub character_id: CharacterId, pub online: u32, pub gender: u16, pub hair: u16, pub hair_color: u16 });
server_packet!(GuildBrokenPacket, 0x015E, { pub result: u32 });
server_packet!(TradeInvitePacket, 0x01F4, { #[length(24)] pub name: String, pub account_id: AccountId, pub level: u16 });
server_packet!(TradeReplyPacket, 0x01F5, { pub result: u8, pub account_id: AccountId, pub level: u16 });
server_packet!(TradeItemAckPacket, 0x00EA, { pub index: u16, pub result: u8 });
server_packet!(TradeLockPacket, 0x00EC, { pub who: u8 });
server_packet!(TradeCanceledPacket, 0x00EE, {});
server_packet!(TradeCompletedPacket, 0x00F0, { pub result: u8 });
server_packet!(TradeOfferPacket, 0x0B42, {
    pub item_id: u32, pub item_type: u8, pub amount: u32, pub identified: u8, pub damaged: u8,
    pub cards: [u32; 4], pub options: [u8; 25], pub location: u32, pub appearance: u16, pub refine: u8, pub grade: u8
});

#[derive(Debug, Clone)]
pub enum SocialEvent {
    PartyCreated(PartyCreatedPacket),
    PartyInvite(PartyInvitePacket),
    PartyInviteResult(PartyInviteResultPacket),
    GuildPermissions(GuildPermissionsPacket),
    GuildMembership(GuildMembershipPacket),
    GuildCreated(GuildCreatedPacket),
    GuildInviteResult(GuildInviteResultPacket),
    GuildInvite(GuildInvitePacket),
    GuildNotice(GuildNoticePacket),
    GuildInformation(GuildInformationPacket),
    GuildMembers(GuildMembersPacket),
    GuildChat(GuildChatPacket),
    GuildMemberLeft(GuildMemberLeftPacket),
    GuildMemberExpelled(GuildMemberExpelledPacket),
    GuildMemberOnline(GuildMemberOnlinePacket),
    GuildBroken(GuildBrokenPacket),
    TradeInvite(TradeInvitePacket),
    TradeReply(TradeReplyPacket),
    TradeItemAck(TradeItemAckPacket),
    TradeLock(TradeLockPacket),
    TradeCanceled,
    TradeCompleted(TradeCompletedPacket),
    TradeOffer(TradeOfferPacket),
}
