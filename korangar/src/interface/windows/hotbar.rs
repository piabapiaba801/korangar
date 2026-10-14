use korangar_components::skill_box;
use korangar_interface::element::Element;
use korangar_interface::window::{CustomWindow, Window};
use ragnarok_packets::HotbarSlot;
use rust_state::{ArrayLookupExt, OptionExt, Path};

use crate::interface::resource::SkillSource;
use crate::interface::windows::WindowClass;
use crate::state::hotbar::{HOTBAR_SLOTS, HOTBAR_SLOTS_PER_ROW};
use crate::state::skills::{LearnableSkill, LearnedSkill, LearnedSkillPath};
use crate::state::theme::InterfaceThemeType;
use crate::state::ClientState;

pub struct HotbarWindow<A, B> {
    hotbar_path: A,
    skills_path: B,
}

impl<A, B> HotbarWindow<A, B> {
    pub fn new(hotbar_path: A, skills_path: B) -> Self {
        Self { hotbar_path, skills_path }
    }
}

fn hotbar_row<A, B>(hotbar_path: A, skills_path: B, row: usize) -> impl Element<ClientState>
where
    A: Path<ClientState, [Option<LearnableSkill>; HOTBAR_SLOTS]>,
    B: Path<ClientState, Vec<LearnedSkill>>,
{
    use korangar_interface::prelude::*;

    split! {
        gaps: theme().window().gaps(),
        children: std::array::from_fn::<_, HOTBAR_SLOTS_PER_ROW, _>(|index| {
            let slot = row * HOTBAR_SLOTS_PER_ROW + index;
            let learnable_skill_path = hotbar_path.array_index(slot).unwrapped();
            let learned_skill_path = LearnedSkillPath::new(learnable_skill_path, skills_path);

            skill_box! {
                learnable_skill_path,
                learned_skill_path,
                source: SkillSource::Hotbar { slot: HotbarSlot(slot as u16) },
            }
        }),
    }
}

impl<A, B> CustomWindow<ClientState> for HotbarWindow<A, B>
where
    A: Path<ClientState, [Option<LearnableSkill>; HOTBAR_SLOTS]>,
    B: Path<ClientState, Vec<LearnedSkill>>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Hotbar)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: "Habilidades",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            elements: (
                hotbar_row(self.hotbar_path, self.skills_path, 1),
                hotbar_row(self.hotbar_path, self.skills_path, 0),
            )
        }
    }
}
