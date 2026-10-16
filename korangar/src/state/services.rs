use korangar_interface::element::StateElement;
use ragnarok_packets::MailSummary;
use rust_state::RustState;

#[derive(Clone, Debug)]
pub enum ServiceAction {
    UseInventoryItem { index: u16 },
    SelectTradeItem { index: u16 },
    OpenAuction,
    OpenMail,
    RefreshMail,
    ReadMail { inbox: u8, id: u64 },
    ClaimZeny,
    ClaimItems,
    DeleteMail,
    MailPage { next: bool },
    OpenPets,
    RenamePet,
    TogglePetAutoFeed,
    PetMenu { action: u8 },
    HatchPet { index: u16 },
    OpenSocial,
    CreateParty,
    InviteParty,
    ReplyParty { accept: bool },
    LeaveParty,
    OpenGuild,
    RefreshGuild,
    CreateGuild,
    InviteGuild,
    ReplyGuild { accept: bool },
    LeaveGuild,
    GuildChat,
    OpenTrade,
    RequestTrade,
    ReplyTrade { accept: bool },
    AddTradeItem,
    AddTradeZeny,
    LockTrade,
    ConfirmTrade,
    CancelTrade,
}

#[derive(Default, RustState, StateElement)]
pub struct ServicesState {
    pub character_id: u32,
    pub mail: Vec<MailSummary>,
    pub mail_page: usize,
    pub mail_selected_id: u64,
    pub mail_selected_inbox: u8,
    pub mail_body: String,
    pub mail_zeny: u64,
    pub mail_items: u8,
    pub mail_loaded: bool,
    pub mail_pending_zeny: bool,
    pub mail_pending_items: bool,
    pub mail_notice: String,
    pub pet_entity: u32,
    pub pet_capture_pending: bool,
    pub pet_name: String,
    pub pet_new_name: String,
    pub pet_autofeed: bool,
    pub pet_level: u16,
    pub pet_hunger: u16,
    pub pet_intimacy: u16,
    pub pet_description: String,
    pub pet_eggs: Vec<u16>,
    pub social_name: String,
    pub group_name: String,
    pub social_notice: String,
    pub party_invite_id: u32,
    pub guild_invite_id: u32,
    pub guild_id: u32,
    pub guild_name: String,
    pub guild_description: String,
    pub guild_roster: String,
    pub guild_message: String,
    pub trade_invite: bool,
    pub trade_active: bool,
    pub trade_self_locked: bool,
    pub trade_other_locked: bool,
    pub trade_item_index: String,
    pub trade_amount: String,
    pub trade_zeny_input: String,
    pub trade_own_offer: String,
    pub trade_other_offer: String,
    pub trade_pending_offer: String,
    pub trade_pending_index: u16,
    pub trade_offered_indices: Vec<u16>,
    pub trade_notice: String,
    pub trade_balance: String,
}

impl ServicesState {
    pub fn update_pet_description(&mut self) {
        self.pet_description = if self.pet_name.is_empty() {
            "No active companion. Use a Pet Incubator to hatch an egg.".to_owned()
        } else {
            format!(
                "{}  |  Level {}\nHunger: {}/100  |  Loyalty: {}/750\nAuto-feed: {}",
                self.pet_name,
                self.pet_level,
                self.pet_hunger,
                self.pet_intimacy,
                if self.pet_autofeed { "On" } else { "Off" }
            )
        };
    }
}
