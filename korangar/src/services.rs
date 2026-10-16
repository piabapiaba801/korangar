//! Native game services. Inventory and currency changes remain
//! server-authoritative.
use ragnarok_packets::{
    ClaimMailItemsPacket, ClaimMailZenyPacket, DeleteMailPacket, HatchPetPacket, OpenMailboxPacket, PetMenuPacket, ReadMailPacket,
    RefreshMailboxPacket, *,
};

use crate::state::inventory::InventoryPathExt;
use crate::*;

impl Client {
    pub(crate) fn handle_game_service(&mut self, action: ServiceAction) {
        if self.client_state.try_follow(this_entity()).is_none() {
            return;
        }
        let result = match action {
            ServiceAction::OpenAuction => {
                self.ensure_auction_agent();
                if self.auction_agent.is_none() {
                    self.client_state.follow_mut(client_state().chat_messages()).push(ChatMessage::new(
                        "Auction agent unavailable. Check the local HUD installation.".to_owned(),
                        MessageColor::Error,
                    ));
                    return;
                }
                let player = self.client_state.follow(this_entity().manually_asserted());
                self.networking_system
                    .cast_skill(SkillId(7007), ragnarok_packets::SkillLevel(1), player.get_entity_id())
            }
            ServiceAction::UseInventoryItem { index } => {
                let accessory = self.client_state.follow(client_state().inventory().items()).iter().any(|item| item.index.0 == index && item.item_type == 8);
                if accessory { let _ = self.networking_system.request_item_equip(InventoryIndex(index), EquipPosition::empty()); return; }
                let player = self.client_state.follow(this_entity().manually_asserted());
                self.networking_system
                    .send_game_service(UseInventoryItemPacket::new(InventoryIndex(index), player.get_entity_id()))
            }
            ServiceAction::SelectTradeItem { index } => {
                let services = self.client_state.follow_mut(client_state().services());
                services.trade_item_index = index.to_string();
                services.trade_amount = "1".to_owned();
                services.trade_notice = "Item selected. Enter a quantity and choose Add Item.".to_owned();
                return;
            }
            ServiceAction::OpenMail => {
                if self.interface.is_window_with_class_open(WindowClass::Mailbox) {
                    self.interface.close_window_with_class(WindowClass::Mailbox);
                    return;
                }
                self.interface.open_window(MailboxWindow::new(
                    self.texture_loader.create_color(
                        "freokro mailbox frame",
                        image::load_from_memory(include_bytes!("../assets/ui/skill-tree-frame-v1.png"))
                            .expect("mail frame")
                            .to_rgba8(),
                        true,
                    ),
                ));
                self.client_state.follow_mut(client_state().services()).mail_notice = "Loading mailbox...".to_owned();
                self.networking_system.send_game_service(OpenMailboxPacket::new(0, 0, 0))
            }
            ServiceAction::RefreshMail => {
                let services = self.client_state.follow_mut(client_state().services());
                services.mail_pending_zeny = false;
                services.mail_pending_items = false;
                services.mail_loaded = false;
                services.mail_selected_id = 0;
                services.mail_body.clear();
                self.networking_system.send_game_service(RefreshMailboxPacket::new(0, [0; 16]))
            }
            ServiceAction::ReadMail { inbox, id } => {
                let services = self.client_state.follow_mut(client_state().services());
                if services.mail_pending_items || services.mail_pending_zeny {
                    return;
                }
                services.mail_selected_id = id;
                services.mail_selected_inbox = inbox;
                services.mail_body = "Loading letter...".to_owned();
                services.mail_zeny = 0;
                services.mail_items = 0;
                services.mail_loaded = false;
                self.networking_system.send_game_service(ReadMailPacket::new(inbox, id))
            }
            ServiceAction::ClaimZeny => {
                let services = self.client_state.follow_mut(client_state().services());
                if !services.mail_loaded || services.mail_selected_id == 0 || services.mail_zeny == 0 || services.mail_pending_zeny {
                    return;
                }
                services.mail_pending_zeny = true;
                self.networking_system.send_game_service(ClaimMailZenyPacket::new(
                    services.mail_selected_id,
                    services.mail_selected_inbox,
                ))
            }
            ServiceAction::ClaimItems => {
                let services = self.client_state.follow_mut(client_state().services());
                if !services.mail_loaded || services.mail_selected_id == 0 || services.mail_items == 0 || services.mail_pending_items {
                    return;
                }
                services.mail_pending_items = true;
                self.networking_system.send_game_service(ClaimMailItemsPacket::new(
                    services.mail_selected_id,
                    services.mail_selected_inbox,
                ))
            }
            ServiceAction::DeleteMail => {
                let services = self.client_state.follow_mut(client_state().services());
                if !services.mail_loaded
                    || services.mail_selected_id == 0
                    || services.mail_zeny > 0
                    || services.mail_items > 0
                    || services.mail_pending_items
                    || services.mail_pending_zeny
                {
                    services.mail_notice = "Read the letter and collect all attachments before deleting it.".to_owned();
                    return;
                }
                self.networking_system
                    .send_game_service(DeleteMailPacket::new(services.mail_selected_inbox, services.mail_selected_id))
            }
            ServiceAction::MailPage { next } => {
                let services = self.client_state.follow_mut(client_state().services());
                services.mail_page = if next {
                    (services.mail_page + 1).min(services.mail.len().saturating_sub(1) / 7)
                } else {
                    services.mail_page.saturating_sub(1)
                };
                return;
            }
            ServiceAction::OpenPets => {
                if self.interface.is_window_with_class_open(WindowClass::Pets) {
                    self.interface.close_window_with_class(WindowClass::Pets);
                    return;
                }
                self.client_state.follow_mut(client_state().services()).update_pet_description();
                self.interface.open_window(PetsWindow::new(
                    self.texture_loader.create_color(
                        "freokro companion frame",
                        image::load_from_memory(include_bytes!("../assets/ui/character-frame-v1.png"))
                            .expect("pet frame")
                            .to_rgba8(),
                        true,
                    ),
                ));
                self.networking_system.send_game_service(PetMenuPacket::new(0))
            }
            ServiceAction::RenamePet => {
                let name = self.client_state.follow(client_state().services()).pet_new_name.trim().to_owned();
                if name.is_empty() || name.len() > 23 {
                    return;
                }
                self.networking_system.send_game_service(RenamePetPacket::new(name))
            }
            ServiceAction::TogglePetAutoFeed => {
                let enabled = !self.client_state.follow(client_state().services()).pet_autofeed;
                self.networking_system
                    .send_game_service(ChangeConfigurationPacket::new(2, u32::from(enabled)))
            }
            ServiceAction::PetMenu { action } => self.networking_system.send_game_service(PetMenuPacket::new(action)),
            ServiceAction::HatchPet { index } => self.networking_system.send_game_service(HatchPetPacket::new(index)),
            ServiceAction::OpenSocial => {
                self.toggle_social_window(0);
                return;
            }
            ServiceAction::OpenGuild => {
                self.toggle_social_window(1);
                let _ = self.networking_system.send_game_service(GuildPermissionsRequestPacket::new());
                let _ = self.networking_system.send_game_service(GuildInfoRequestPacket::new(0));
                self.networking_system.send_game_service(GuildInfoRequestPacket::new(1))
            }
            ServiceAction::RefreshGuild => {
                let _ = self.networking_system.send_game_service(GuildInfoRequestPacket::new(0));
                self.networking_system.send_game_service(GuildInfoRequestPacket::new(1))
            }
            ServiceAction::CreateParty => {
                let name = self.client_state.follow(client_state().services()).group_name.trim().to_owned();
                if name.is_empty() || name.len() > 23 {
                    self.social_notice("Enter a party name (1–23 bytes).");
                    return;
                }
                self.networking_system.send_game_service(CreatePartyPacket::new(name, 0, 0))
            }
            ServiceAction::InviteParty => {
                let name = self.client_state.follow(client_state().services()).social_name.trim().to_owned();
                if name.is_empty() || name.len() > 23 {
                    self.social_notice("Enter the player's name.");
                    return;
                }
                self.networking_system.send_game_service(InvitePartyPacket::new(name))
            }
            ServiceAction::ReplyParty { accept } => {
                let services = self.client_state.follow_mut(client_state().services());
                let id = std::mem::take(&mut services.party_invite_id);
                if id == 0 {
                    return;
                }
                self.networking_system
                    .send_game_service(ReplyPartyInvitePacket::new(id, u8::from(accept)))
            }
            ServiceAction::LeaveParty => self.networking_system.send_game_service(LeavePartyPacket::new()),
            ServiceAction::CreateGuild => {
                let name = self.client_state.follow(client_state().services()).social_name.trim().to_owned();
                if name.is_empty() || name.len() > 23 {
                    self.social_notice("Enter a guild name (1–23 bytes).");
                    return;
                }
                let account = self.saved_login_data.as_ref().unwrap().account_id;
                self.networking_system.send_game_service(CreateGuildPacket::new(account, name))
            }
            ServiceAction::InviteGuild => {
                let name = self.client_state.follow(client_state().services()).social_name.trim().to_owned();
                if name.is_empty() || name.len() > 23 {
                    self.social_notice("Enter the player's name.");
                    return;
                }
                self.networking_system.send_game_service(InviteGuildPacket::new(name))
            }
            ServiceAction::ReplyGuild { accept } => {
                let services = self.client_state.follow_mut(client_state().services());
                let id = std::mem::take(&mut services.guild_invite_id);
                if id == 0 {
                    return;
                }
                self.networking_system
                    .send_game_service(ReplyGuildInvitePacket::new(id, u32::from(accept)))
            }
            ServiceAction::LeaveGuild => {
                let services = self.client_state.follow(client_state().services());
                if services.guild_id == 0 {
                    return;
                }
                self.networking_system.send_game_service(LeaveGuildPacket::new(
                    services.guild_id,
                    self.saved_login_data.as_ref().unwrap().account_id,
                    CharacterId(services.character_id),
                    "Leaving the guild".to_owned(),
                ))
            }
            ServiceAction::GuildChat => {
                let message = self.client_state.follow(client_state().services()).guild_message.trim().to_owned();
                if message.is_empty() {
                    return;
                }
                let name = self.client_state.follow(client_state().player_name());
                let mut data = format!("{} : {}", name, message).into_bytes();
                data.push(0);
                self.client_state.follow_mut(client_state().services()).guild_message.clear();
                self.networking_system.send_game_service(GuildChatRequestPacket::new(data))
            }
            ServiceAction::OpenTrade => {
                let zeny = self.client_state.try_follow(this_player()).map(|player| player.zeny).unwrap_or(0);
                self.client_state.follow_mut(client_state().services()).trade_balance = format!("Available Zeny: {}", zeny);
                self.toggle_social_window(2);
                return;
            }
            ServiceAction::RequestTrade => {
                let name = self.client_state.follow(client_state().services()).social_name.trim().to_owned();
                let target = self
                    .client_state
                    .follow(client_state().entities())
                    .iter()
                    .find(|entity| {
                        entity.get_entity_type() == EntityType::Player
                            && entity
                                .get_details()
                                .is_some_and(|details| details.split('#').next().unwrap_or(details).eq_ignore_ascii_case(&name))
                    })
                    .map(|entity| AccountId(entity.get_entity_id().0));
                if let Some(target) = target {
                    self.networking_system.send_game_service(RequestTradePacket::new(target))
                } else {
                    self.client_state.follow_mut(client_state().services()).trade_notice =
                        "Player not found nearby. Hover them to load their name, then retry.".to_owned();
                    return;
                }
            }
            ServiceAction::ReplyTrade { accept } => {
                if !self.client_state.follow(client_state().services()).trade_invite {
                    return;
                }
                self.client_state.follow_mut(client_state().services()).trade_invite = false;
                self.networking_system
                    .send_game_service(ReplyTradePacket::new(if accept { 3 } else { 4 }))
            }
            ServiceAction::AddTradeItem | ServiceAction::AddTradeZeny => {
                let selected_index = self
                    .client_state
                    .follow(client_state().services())
                    .trade_item_index
                    .parse::<u16>()
                    .ok();
                let item_name = self
                    .client_state
                    .follow(client_state().inventory().items())
                    .iter()
                    .find(|item| Some(item.index.0 + 2) == selected_index)
                    .map(|item| {
                        (item.metadata.name.clone(), match item.details {
                            korangar_networking::InventoryItemDetails::Regular { amount, .. } => u32::from(amount),
                            _ => 1,
                        })
                    });
                let zeny = self.client_state.try_follow(this_player()).map(|player| player.zeny).unwrap_or(0);
                let services = self.client_state.follow_mut(client_state().services());
                if !services.trade_active || services.trade_self_locked || !services.trade_pending_offer.is_empty() {
                    return;
                }
                let (index, amount) = if matches!(action, ServiceAction::AddTradeZeny) {
                    let Ok(amount) = services.trade_zeny_input.trim().parse::<u32>() else {
                        services.trade_notice = "Enter a valid Zeny amount.".to_owned();
                        return;
                    };
                    if amount == 0 || amount > zeny {
                        services.trade_notice = "You do not have that much Zeny.".to_owned();
                        return;
                    }
                    services.trade_own_offer = services
                        .trade_own_offer
                        .lines()
                        .filter(|line| !line.starts_with("Zeny:"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    services.trade_own_offer.push_str(&format!("\nZeny: {}\n", amount));
                    (0, amount)
                } else {
                    let (Ok(index), Ok(amount)) = (
                        services.trade_item_index.trim().parse::<u16>(),
                        services.trade_amount.trim().parse::<u32>(),
                    ) else {
                        services.trade_notice = "Select an inventory item and quantity.".to_owned();
                        return;
                    };
                    if index < 2 || amount == 0 || amount > 30_000 {
                        return;
                    }
                    if services.trade_offered_indices.contains(&index) || services.trade_offered_indices.len() >= 10 {
                        services.trade_notice =
                            "Up to ten different items may be offered. Cancel the trade to change an existing quantity.".to_owned();
                        return;
                    }
                    let Some((name, available)) = item_name else {
                        services.trade_notice = "Select an available item.".to_owned();
                        return;
                    };
                    if amount > available {
                        services.trade_notice = format!("Only {} available.", available);
                        return;
                    }
                    services.trade_pending_offer = format!("{} x{}", name, amount);
                    services.trade_pending_index = index;
                    (index, amount)
                };
                self.networking_system.send_game_service(AddTradeItemPacket::new(index, amount))
            }
            ServiceAction::LockTrade => {
                let services = self.client_state.follow(client_state().services());
                if !services.trade_active || services.trade_self_locked || !services.trade_pending_offer.is_empty() {
                    return;
                }
                self.networking_system.send_game_service(LockTradePacket::new())
            }
            ServiceAction::ConfirmTrade => {
                let services = self.client_state.follow_mut(client_state().services());
                if !services.trade_active || !services.trade_self_locked || !services.trade_other_locked {
                    services.trade_notice = "Both players must lock their offers before confirming.".to_owned();
                    return;
                }
                self.networking_system.send_game_service(ConfirmTradePacket::new())
            }
            ServiceAction::CancelTrade => self.networking_system.send_game_service(CancelTradePacket::new()),
        };
        if result.is_err() {
            let services = self.client_state.follow_mut(client_state().services());
            services.mail_pending_zeny = false;
            services.mail_pending_items = false;
            services.mail_notice = "Disconnected. Reconnect to use game services.".to_owned();
        }
    }

    fn clear_guild(&mut self) {
        let services = self.client_state.follow_mut(client_state().services());
        services.guild_id = 0;
        services.guild_name.clear();
        services.guild_roster.clear();
        services.guild_description = "You do not belong to a guild.".to_owned();
    }

    fn guild_member_departed(&mut self, character_id: u32) {
        if self.client_state.follow(client_state().services()).character_id == character_id {
            self.clear_guild();
        } else {
            let _ = self.networking_system.send_game_service(GuildInfoRequestPacket::new(1));
        }
    }

    fn social_notice(&mut self, text: &str) {
        self.client_state.follow_mut(client_state().services()).social_notice = text.to_owned();
    }

    pub(crate) fn ensure_auction_agent(&mut self) {
        if self.auction_agent_attempted {
            return;
        }
        self.auction_agent_attempted = true;
        let Ok(executable) = std::env::current_exe() else {
            return;
        };
        let Some(game) = executable.parent() else {
            return;
        };
        let root = game.parent().and_then(|p| p.parent()).unwrap_or(game);
        let candidates = [
            game.join("FreokRO-Auction-HUD.exe"),
            root.join("leilao-hud/app/FreokRO-Auction-HUD.exe"),
        ];
        let Some(agent) = candidates.into_iter().find(|path| path.is_file()) else {
            return;
        };
        let mut command = std::process::Command::new(&agent);
        command
            .arg("--agent")
            .current_dir(agent.parent().unwrap())
            .env("FREOKRO_AUCTION_RUNTIME", root.join("servidor/client/auction-ui/runtime"));
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        self.auction_agent = command.spawn().ok();
    }

    fn toggle_social_window(&mut self, kind: u8) {
        let class = match kind {
            0 => WindowClass::Social,
            1 => WindowClass::Guild,
            _ => WindowClass::Trade,
        };
        if self.interface.is_window_with_class_open(class) {
            self.interface.close_window_with_class(class);
            return;
        }
        self.show_social_window(kind);
    }

    fn show_social_window(&mut self, kind: u8) {
        let class = match kind {
            0 => WindowClass::Social,
            1 => WindowClass::Guild,
            _ => WindowClass::Trade,
        };
        if self.interface.is_window_with_class_open(class) {
            return;
        }
        let frame = self.texture_loader.create_color(
            "freokro social services frame",
            image::load_from_memory(include_bytes!("../assets/ui/skill-tree-frame-v1.png"))
                .expect("social frame")
                .to_rgba8(),
            true,
        );
        match kind {
            0 => self.interface.open_window(SocialWindow::new(frame)),
            1 => self.interface.open_window(GuildWindow::new(frame)),
            _ => self.interface.open_window(TradeWindow::new(frame)),
        }
    }

    pub(crate) fn handle_social_event(&mut self, event: SocialEvent) {
        match event {
            SocialEvent::PartyCreated(p) => self.social_notice(match p.result {
                0 => "Party created.",
                1 => "That party name is taken.",
                2 => "You already belong to a party.",
                _ => "Party creation is unavailable here.",
            }),
            SocialEvent::PartyInvite(p) => {
                let services = self.client_state.follow_mut(client_state().services());
                services.party_invite_id = p.party_id.0;
                services.social_notice = format!("Party invitation: {}", p.party_name);
                self.show_social_window(0);
            }
            SocialEvent::PartyInviteResult(p) => self.social_notice(&format!("Party invitation to {}: {}", p.name, match p.result {
                2 => "accepted",
                1 => "declined",
                3 => "party full",
                7 => "player offline",
                _ => "unavailable or blocked",
            })),
            SocialEvent::GuildMemberLeft(p) => self.guild_member_departed(p.character_id.0),
            SocialEvent::GuildMemberExpelled(p) => self.guild_member_departed(p.character_id.0),
            SocialEvent::GuildMemberOnline(_) => {
                let _ = self.networking_system.send_game_service(GuildInfoRequestPacket::new(1));
            }
            SocialEvent::GuildBroken(p) => {
                if p.result == 0 {
                    self.clear_guild();
                }
            }
            SocialEvent::GuildPermissions(_) => {}
            SocialEvent::GuildMembership(p) => {
                let services = self.client_state.follow_mut(client_state().services());
                services.guild_id = p.guild_id;
                services.guild_name = p.name.clone();
                services.guild_description = p.name;
            }
            SocialEvent::GuildCreated(p) => {
                self.social_notice(if p.result == 0 {
                    "Guild created."
                } else {
                    "Cannot create guild. Check name, Emperium, and membership."
                });
                let _ = self.networking_system.send_game_service(GuildInfoRequestPacket::new(0));
            }
            SocialEvent::GuildInviteResult(p) => self.social_notice(match p.result {
                0 => "That player already belongs to a guild.",
                1 => "Guild invitation declined.",
                2 => "Guild invitation accepted.",
                3 => "Guild is full.",
                _ => "Guild invitation unavailable.",
            }),
            SocialEvent::GuildInvite(p) => {
                let services = self.client_state.follow_mut(client_state().services());
                services.guild_invite_id = p.guild_id;
                services.social_notice = format!("Guild invitation: {}", p.name);
                self.show_social_window(1);
            }
            SocialEvent::GuildNotice(p) => self.social_notice(&format!("{}\n{}", p.subject, p.notice)),
            SocialEvent::GuildInformation(p) => {
                let services = self.client_state.follow_mut(client_state().services());
                services.guild_id = p.guild_id;
                services.guild_name = p.name.clone();
                services.guild_description = format!(
                    "{}  |  Level {}  |  Online {}/{}\nLeader: {}",
                    p.name, p.level, p.online, p.capacity, p.master_name
                );
            }
            SocialEvent::GuildMembers(p) => {
                let mut roster = String::new();
                for member in p.data.chunks_exact(58) {
                    let name = String::from_utf8_lossy(&member[34..58]).trim_end_matches('\0').to_owned();
                    if name.is_empty() {
                        continue;
                    }
                    let level = u16::from_le_bytes([member[16], member[17]]);
                    let online = u32::from_le_bytes(member[22..26].try_into().unwrap()) != 0;
                    roster.push_str(&format!(
                        "{}  |  Lv {}  |  {}\n",
                        name,
                        level,
                        if online { "Online" } else { "Offline" }
                    ));
                }
                self.client_state.follow_mut(client_state().services()).guild_roster = roster;
            }
            SocialEvent::GuildChat(p) => self.client_state.follow_mut(client_state().chat_messages()).push(ChatMessage::new(
                format!("[Guild] {}", String::from_utf8_lossy(&p.data).trim_end_matches('\0')),
                MessageColor::Information,
            )),
            SocialEvent::TradeInvite(p) => {
                let services = self.client_state.follow_mut(client_state().services());
                services.trade_invite = true;
                services.trade_notice = format!("{} wants to trade. Accept or decline.", p.name);
                self.show_social_window(2);
            }
            SocialEvent::TradeReply(p) => {
                let services = self.client_state.follow_mut(client_state().services());
                services.trade_active = p.result == 3;
                services.trade_invite = false;
                services.trade_self_locked = false;
                services.trade_other_locked = false;
                services.trade_own_offer.clear();
                services.trade_offered_indices.clear();
                services.trade_other_offer.clear();
                services.trade_pending_offer.clear();
                services.trade_notice = if p.result == 3 {
                    "Trade open. Review both offers, lock, then confirm."
                } else {
                    "Trade declined, busy, or player out of range."
                }
                .to_owned();
                if p.result == 3 {
                    self.show_social_window(2);
                }
            }
            SocialEvent::TradeItemAck(p) => {
                let services = self.client_state.follow_mut(client_state().services());
                let offer = std::mem::take(&mut services.trade_pending_offer);
                if p.result == 0 {
                    if offer.is_empty() {
                        return;
                    }
                    services.trade_offered_indices.push(services.trade_pending_index);
                    if p.index == 0 {
                        services.trade_own_offer = services
                            .trade_own_offer
                            .lines()
                            .filter(|line| !line.starts_with("Zeny:"))
                            .collect::<Vec<_>>()
                            .join("\n");
                    }
                    services.trade_own_offer.push_str(&format!("{}\n", offer));
                    services.trade_notice = "Offer updated by server.".to_owned();
                } else {
                    services.trade_notice = "Cannot add that offer. Check inventory, capacity, and trade restrictions.".to_owned();
                }
            }
            SocialEvent::TradeOffer(p) => {
                let options = p.options.chunks_exact(5).filter_map(|raw| {
                    crate::interface::windows::option_text(&ItemOptions {
                        index: u16::from_le_bytes([raw[0], raw[1]]),
                        value: u16::from_le_bytes([raw[2], raw[3]]), parameter: raw[4],
                    })
                }).collect::<Vec<_>>().join(", ");
                let name = self
                    .library
                    .get::<ItemName>(ItemNameKey {
                        item_id: ItemId(p.item_id),
                        is_identified: p.identified != 0,
                    })
                    .to_string();
                let cards = p
                    .cards
                    .iter()
                    .filter(|id| **id != 0)
                    .map(|id| {
                        self.library
                            .get::<ItemName>(ItemNameKey {
                                item_id: ItemId(*id),
                                is_identified: true,
                            })
                            .to_string()
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let services = self.client_state.follow_mut(client_state().services());
                if p.item_id == 0 {
                    services.trade_other_offer = services
                        .trade_other_offer
                        .lines()
                        .filter(|line| !line.starts_with("Zeny:"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    services.trade_other_offer.push_str(&format!("Zeny: {}\n", p.amount));
                } else {
                    services.trade_other_offer.push_str(&format!(
                        "+{} {} x{}{}{} | Grade {}{}{}{}{}\n",
                        p.refine,
                        name,
                        p.amount,
                        if cards.is_empty() { "" } else { " | " },
                        cards, p.grade,
                        if options.is_empty() { "" } else { " | " }, options,
                        if p.identified == 0 { " | Unidentified" } else { "" },
                        if p.damaged != 0 { " | Broken" } else { "" }
                    ));
                }
            }
            SocialEvent::TradeLock(p) => {
                let services = self.client_state.follow_mut(client_state().services());
                if p.who == 0 {
                    services.trade_self_locked = true;
                } else {
                    services.trade_other_locked = true;
                }
                services.trade_notice = if services.trade_self_locked && services.trade_other_locked {
                    "Both offers locked. Confirm only after reviewing their contents."
                } else {
                    "Offer locked. Waiting for the other player."
                }
                .to_owned();
            }
            SocialEvent::TradeCanceled => {
                let services = self.client_state.follow_mut(client_state().services());
                services.trade_active = false;
                services.trade_invite = false;
                services.trade_pending_offer.clear();
                services.trade_notice = "Trade canceled. Nothing was exchanged.".to_owned();
            }
            SocialEvent::TradeCompleted(p) => {
                let services = self.client_state.follow_mut(client_state().services());
                services.trade_active = false;
                services.trade_pending_offer.clear();
                services.trade_notice = if p.result == 0 {
                    "Trade completed."
                } else {
                    "Trade failed. Check the server's inventory updates."
                }
                .to_owned();
            }
        }
    }
}
