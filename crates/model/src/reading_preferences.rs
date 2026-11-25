/// Fixed-size, application-wide reading settings; independent of Discord accounts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadingPreferences {
	pub zoom_percent: u16,
	pub sidebar_width: u16,
	/// Member list in servers; direct and group messages keep their own choice.
	pub show_members: bool,
	pub show_members_dms: bool,
	/// Tighter gaps between message groups.
	pub compact_messages: bool,
	/// Explicit opt-in; double-clicks do not react by default.
	pub double_click_reaction_enabled: bool,
	/// Stable index into DOUBLE_CLICK_REACTIONS, independent of emoji usage rankings.
	pub double_click_reaction: u8,
	pub animate_gifs: bool,
	pub smooth_scrolling: bool,
	pub scroll_speed_percent: u16,
	pub hide_media_links: bool,
	pub confirm_external_links: bool,
}
impl Default for ReadingPreferences {
	fn default() -> Self {
		Self {
			zoom_percent: 100,
			sidebar_width: 236,
			show_members: !cfg!(target_os = "macos"),
			show_members_dms: !cfg!(target_os = "macos"),
			compact_messages: false,
			double_click_reaction_enabled: false,
			double_click_reaction: 0,
			animate_gifs: true,
			smooth_scrolling: true,
			scroll_speed_percent: 100,
			hide_media_links: true,
			confirm_external_links: true,
		}
	}
}
impl ReadingPreferences {
	pub const DOUBLE_CLICK_REACTIONS: [&'static str; 6] = ["❤️", "👍", "😂", "🎉", "😮", "😢"];

	pub fn double_click_emoji(self) -> &'static str {
		Self::DOUBLE_CLICK_REACTIONS
			.get(usize::from(self.double_click_reaction))
			.copied()
			.unwrap_or(Self::DOUBLE_CLICK_REACTIONS[0])
	}

	pub fn is_valid(self) -> bool {
		(50..=150).contains(&self.zoom_percent)
			&& (190..=360).contains(&self.sidebar_width)
			&& (25..=300).contains(&self.scroll_speed_percent)
			&& usize::from(self.double_click_reaction) < Self::DOUBLE_CLICK_REACTIONS.len()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn inclusive_zoom_and_sidebar_bounds() {
		for (zoom_percent, sidebar_width, valid) in [
			(49, 236, false),
			(50, 190, true),
			(150, 360, true),
			(151, 236, false),
			(100, 189, false),
			(100, 361, false),
		] {
			let preferences = ReadingPreferences {
				zoom_percent,
				sidebar_width,
				..ReadingPreferences::default()
			};
			assert_eq!(preferences.is_valid(), valid);
		}
	}
}
