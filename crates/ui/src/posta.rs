use crate::{MessagingUi, design, icons};
use client_core::{Command, State};
use egui::{Button, Color32, RichText, Vec2};

pub(super) const ENABLED: bool = cfg!(target_os = "macos");
pub(super) const SERVER_WIDTH: f32 = 150.0;

impl MessagingUi {
	pub(super) fn posta_toolbar(
		&mut self,
		ui: &mut egui::Ui,
		state: &mut State,
		commands: &mut Vec<Command>,
	) {
		let language = self.language;
		let colors = design::palette(ui);
		if let Some(channel) = state.selected.and_then(|id| state.channel(id)) {
			if let Some(guild) = channel.guild {
				self.posta_last_server = Some(guild);
			} else if matches!(channel.kind, 1 | 3) {
				self.posta_last_dm = Some(channel.id);
			}
		}
		let friends = self.guild.is_none() && state.selected.is_none();
		let requests = if crate::interface_tweaks::on(
			ui.ctx(),
			extensions::InterfaceTweak::HiddenRequestCounts,
		) {
			(0, 0)
		} else {
			state.home_request_parts()
		};
		egui::Panel::top("posta-toolbar")
			.exact_size(48.0)
			.show_separator_line(true)
			.frame(
				egui::Frame::new()
					.fill(design::section_surface(
						ui,
						design::window_palette(ui).sidebar,
						design::ImageSection::TopBar,
					))
					.inner_margin(egui::Margin::symmetric(12, 0)),
			)
			.show(ui, |ui| {
				ui.horizontal_centered(|ui| {
					icons::inline(ui, icons::Icon::Serein, 23.0, colors.accent);
					if ui.available_width() >= 530.0 {
						ui.label(design::semibold(ui, "Wiredusk", 16.0).color(colors.text_strong));
						ui.add_space(12.0);
					}
					let messages = tab(
						ui,
						counted(language.text("posta-messages"), requests.1),
						!friends && self.guild.is_none(),
					);
					messages.context_menu(|ui| self.server_menu.all_read_item(ui, state));
					if messages.clicked() {
						self.guild = None;
						let last = self
							.posta_last_dm
							.or(state.last_viewed_dm)
							.filter(|id| {
								state.channel(*id).is_some_and(|channel| {
									channel.guild.is_none() && matches!(channel.kind, 1 | 3)
								}) && state.can_view(*id)
							})
							.or_else(|| {
								state
									.channels
									.iter()
									.filter(|channel| {
										channel.guild.is_none() && matches!(channel.kind, 1 | 3)
									})
									.filter(|channel| {
										state.can_view(channel.id)
											&& !state.spam_direct(channel.id) && !state
											.message_request(channel.id)
									})
									.max_by_key(|channel| {
										channel.last_message.unwrap_or(channel.id)
									})
									.map(|channel| channel.id)
							});
						if let Some(command) = match last {
							Some(id) if state.selected != Some(id) => state.select(id),
							Some(_) => None,
							None => state.open_messages(),
						} {
							commands.push(command);
						}
					}
					if tab(ui, language.text("posta-servers"), self.guild.is_some()).clicked()
						&& self.guild.is_none()
					{
						let target = self
							.posta_last_server
							.filter(|id| state.guild(*id).is_some())
							.or_else(|| state.guilds.first().map(|guild| guild.id));
						if let Some(guild) = target {
							self.guild = Some(guild);
							self.search.open = false;
							if state.guild_target(guild).is_some() {
								commands.extend(state.select_guild(guild));
							} else {
								state.open_home();
								self.navigation_channel = None;
							}
						} else {
							self.join_server.open_picker(state.generation);
						}
					}
					if tab(ui, counted(language.text("friends"), requests.0), friends).clicked() {
						self.guild = None;
						self.search.open = false;
						state.open_home();
					}
					ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
						let label = language.text("find-conversation");
						let search = if ui.available_width() >= 160.0 {
							ui.add(Button::new(&label).min_size(Vec2::new(138.0, 30.0)))
						} else {
							icons::button(ui, icons::Icon::Search, 30.0, &label)
						};
						if search
							.on_hover_text(language.text("find-conversation-tooltip"))
							.clicked() && !self.ime_active
						{
							self.switcher.open(ui.ctx());
						}
					});
				});
			});
	}

	pub(super) fn posta_servers(
		&mut self,
		ui: &mut egui::Ui,
		state: &mut State,
		commands: &mut Vec<Command>,
	) {
		self.rail_cache.sync(state);
		egui::Panel::left("posta-servers")
			.exact_size(SERVER_WIDTH)
			.show_separator_line(true)
			.frame(
				egui::Frame::new()
					.fill(design::section_surface(
						ui,
						design::window_palette(ui).base,
						design::ImageSection::ServerList,
					))
					.inner_margin(egui::Margin::same(8)),
			)
			.show(ui, |ui| {
				ui.spacing_mut().item_spacing.y = 8.0;
				self.scroll
					.attach(
						ui,
						"posta-server-list",
						egui::ScrollArea::vertical().auto_shrink([false, false]),
					)
					.show(ui, |ui| {
						self.server_folders(ui, state, commands, true);
						ui.horizontal(|ui| {
							self.add_server_tile(ui, state, 32.0);
							ui.add(
								egui::Label::new(
									self.language
										.text("notifications-notification-rail-add-a-server"),
								)
								.wrap(),
							);
						});
					});
			});
	}
}

fn counted(label: String, count: u32) -> String {
	if count > 0 {
		format!("{label} · {count}")
	} else {
		label
	}
}

fn tab(ui: &mut egui::Ui, label: String, selected: bool) -> egui::Response {
	let colors = design::palette(ui);
	let text = RichText::new(&label).size(13.0).color(if selected {
		colors.text_strong
	} else {
		colors.text
	});
	let button = Button::new(text)
		.min_size(Vec2::new(0.0, 30.0))
		.corner_radius(8)
		.frame_when_inactive(false);
	let response = ui.add(if selected {
		button.fill(colors.selected).stroke(egui::Stroke::NONE)
	} else {
		button
	});
	response.widget_info(|| {
		egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), selected, &label)
	});
	response
}

pub(super) fn server_label(
	ui: &mut egui::Ui,
	avatar: egui::Response,
	name: &str,
	selected: bool,
) -> egui::Response {
	let rect = egui::Rect::from_min_max(
		avatar.rect.right_top(),
		egui::pos2(ui.max_rect().right(), avatar.rect.bottom()),
	);
	let label = ui.interact(
		rect,
		ui.scope_id().with("server-label"),
		egui::Sense::click_and_drag(),
	);
	let colors = design::palette(ui);
	if selected || label.hovered() || label.has_focus() {
		ui.painter().rect_filled(
			rect,
			7,
			if selected {
				colors.selected
			} else {
				colors.hover
			},
		);
	}
	ui.expand_to_include_rect(rect);
	let mut child = ui.new_child(
		egui::UiBuilder::new()
			.max_rect(rect.shrink2(Vec2::new(4.0, 0.0)))
			.layout(egui::Layout::left_to_right(egui::Align::Center)),
	);
	child.add(
		egui::Label::new(design::medium(&child, name, 12.0).color(if selected {
			colors.text_strong
		} else {
			colors.text
		}))
		.truncate()
		.selectable(false),
	);
	label.widget_info(|| {
		egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), selected, name)
	});
	avatar.union(label)
}

pub(super) const CALL_SURFACE: Color32 = Color32::from_rgb(0x28, 0x24, 0x30);
