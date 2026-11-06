use std::sync::Arc;

use fs::Fs;
use gpui::{Action, App, IntoElement};
use i18n::t;
use project::project_settings::ProjectSettings;
use settings::{BaseKeymap, Settings, update_settings_file};
use theme::{Appearance, SystemAppearance, ThemeRegistry};
use theme_settings::{ThemeAppearanceMode, ThemeName, ThemeSelection, ThemeSettings};
use ui::{
    StatefulInteractiveElement, SwitchField, ToggleButtonGroup, ToggleButtonGroupSize,
    ToggleButtonSimple, Tooltip, prelude::*,
};

use crate::theme_preview::{ThemePreviewStyle, ThemePreviewTile};

const LIGHT_THEMES: [&str; 3] = ["Yo Light", "Catppuccin Latte", "One Light"];
const DARK_THEMES: [&str; 3] = ["Yo Dark", "Catppuccin Mocha", "One Dark"];
const FAMILY_NAMES: [SharedString; 3] = [
    SharedString::new_static("Yo"),
    SharedString::new_static("Catppuccin"),
    SharedString::new_static("One"),
];

fn get_theme_family_themes(theme_name: &str) -> Option<(&'static str, &'static str)> {
    for i in 0..LIGHT_THEMES.len() {
        if LIGHT_THEMES[i] == theme_name || DARK_THEMES[i] == theme_name {
            return Some((LIGHT_THEMES[i], DARK_THEMES[i]));
        }
    }
    None
}

fn render_theme_section(tab_index: &mut isize, cx: &mut App) -> impl IntoElement {
    let theme_selection = ThemeSettings::get_global(cx).theme.clone();
    let system_appearance = theme::SystemAppearance::global(cx);

    let theme_mode = theme_selection
        .mode()
        .unwrap_or_else(|| match *system_appearance {
            Appearance::Light => ThemeAppearanceMode::Light,
            Appearance::Dark => ThemeAppearanceMode::Dark,
        });

    return v_flex()
        .gap_2()
        .child(
            h_flex()
                .justify_between()
                .child(Label::new(t!("Theme")))
                .child(
                    ToggleButtonGroup::single_row(
                        "theme-selector-onboarding-dark-light",
                        [
                            ThemeAppearanceMode::Light,
                            ThemeAppearanceMode::Dark,
                            ThemeAppearanceMode::System,
                        ]
                        .map(|mode| {
                            let mode_name = match mode {
                                ThemeAppearanceMode::Light => t!("Light"),
                                ThemeAppearanceMode::Dark => t!("Dark"),
                                ThemeAppearanceMode::System => t!("System"),
                            };
                            ToggleButtonSimple::new(mode_name, move |_, _, cx| {
                                write_mode_change(mode, cx);
                            })
                        }),
                    )
                    .size(ToggleButtonGroupSize::Medium)
                    .tab_index(tab_index)
                    .selected_index(theme_mode as usize)
                    .style(ui::ToggleButtonGroupStyle::Outlined)
                    .width(rems_from_px(3.0_f32 * 64.0_f32)),
                ),
        )
        .child(
            h_flex()
                .gap_2()
                .justify_between()
                .children(render_theme_previews(tab_index, &theme_selection, cx)),
        );

    fn render_theme_previews(
        tab_index: &mut isize,
        theme_selection: &ThemeSelection,
        cx: &mut App,
    ) -> [impl IntoElement; 3] {
        let system_appearance = SystemAppearance::global(cx);
        let theme_registry = ThemeRegistry::global(cx);

        let theme_seed = 0xBEEF as f32;
        let theme_mode = theme_selection
            .mode()
            .unwrap_or_else(|| match *system_appearance {
                Appearance::Light => ThemeAppearanceMode::Light,
                Appearance::Dark => ThemeAppearanceMode::Dark,
            });
        let appearance = match theme_mode {
            ThemeAppearanceMode::Light => Appearance::Light,
            ThemeAppearanceMode::Dark => Appearance::Dark,
            ThemeAppearanceMode::System => *system_appearance,
        };
        let current_theme_name: SharedString = theme_selection.name(appearance).0.into();

        let theme_names = match appearance {
            Appearance::Light => LIGHT_THEMES,
            Appearance::Dark => DARK_THEMES,
        };

        let themes = theme_names.map(|theme| theme_registry.get(theme).unwrap());

        [0, 1, 2].map(|index| {
            let theme = &themes[index];
            let is_selected = theme.name == current_theme_name;
            let name = theme.name.clone();
            let colors = cx.theme().colors();

            v_flex()
                .w_full()
                .items_center()
                .gap_1()
                .child(
                    h_flex()
                        .id(name)
                        .relative()
                        .w_full()
                        .border_2()
                        .border_color(colors.border_transparent)
                        .rounded(ThemePreviewTile::ROOT_RADIUS)
                        .map(|this| {
                            if is_selected {
                                this.border_color(colors.border_selected)
                            } else {
                                this.opacity(0.8).hover(|s| s.border_color(colors.border))
                            }
                        })
                        .tab_index({
                            *tab_index += 1;
                            *tab_index - 1
                        })
                        .focus(|mut style| {
                            style.border_color = Some(colors.border_focused);
                            style
                        })
                        .on_click({
                            let theme_name = theme.name.clone();

                            move |_, _, cx| {
                                write_theme_change(theme_name.clone(), theme_mode, cx);
                            }
                        })
                        .map(|this| {
                            if theme_mode == ThemeAppearanceMode::System {
                                let (light, dark) = (
                                    theme_registry.get(LIGHT_THEMES[index]).unwrap(),
                                    theme_registry.get(DARK_THEMES[index]).unwrap(),
                                );
                                this.child(
                                    ThemePreviewTile::new(light, theme_seed)
                                        .style(ThemePreviewStyle::SideBySide(dark)),
                                )
                            } else {
                                this.child(
                                    ThemePreviewTile::new(theme.clone(), theme_seed)
                                        .style(ThemePreviewStyle::Bordered),
                                )
                            }
                        }),
                )
                .child(
                    Label::new(FAMILY_NAMES[index].clone())
                        .color(Color::Muted)
                        .size(LabelSize::Small),
                )
        })
    }

    fn write_mode_change(mode: ThemeAppearanceMode, cx: &mut App) {
        let fs = <dyn Fs>::global(cx);
        update_settings_file(fs, cx, move |settings, _cx| {
            theme_settings::set_mode(settings, mode);
        });
    }

    fn write_theme_change(
        theme: impl Into<Arc<str>>,
        theme_mode: ThemeAppearanceMode,
        cx: &mut App,
    ) {
        let fs = <dyn Fs>::global(cx);
        let theme = theme.into();
        update_settings_file(fs, cx, move |settings, cx| match theme_mode {
            ThemeAppearanceMode::System => {
                let (light_theme, dark_theme) =
                    get_theme_family_themes(&theme).unwrap_or((theme.as_ref(), theme.as_ref()));

                settings.theme.theme = Some(settings::ThemeSelection::Dynamic {
                    mode: ThemeAppearanceMode::System,
                    light: ThemeName(light_theme.into()),
                    dark: ThemeName(dark_theme.into()),
                });
            }
            ThemeAppearanceMode::Light => theme_settings::set_theme(
                settings,
                theme,
                Appearance::Light,
                *SystemAppearance::global(cx),
            ),
            ThemeAppearanceMode::Dark => theme_settings::set_theme(
                settings,
                theme,
                Appearance::Dark,
                *SystemAppearance::global(cx),
            ),
        });
    }
}

fn render_editing_section(tab_index: &mut isize, cx: &mut App) -> impl IntoElement {
    let vim_enabled = cx
        .global::<settings::SettingsStore>()
        .merged_settings()
        .editor
        .vim_mode
        .unwrap_or(false);
    let fs = <dyn Fs>::global(cx);
    *tab_index += 1;
    v_flex()
        .gap_3()
        .child(
            h_flex()
                .justify_between()
                .child(Label::new(t!("Base Keymap")))
                .child(Label::new(BaseKeymap::get_global(cx).to_string()).color(Color::Muted)),
        )
        .child(Label::new(t!("You can change the keymap later in Settings.")).color(Color::Muted))
        .child(
            SwitchField::new(
                "onboarding-lightweight-vim",
                Some(t!("Enable Lightweight Vim")),
                Some(t!(
                    "Use Normal, Insert, and Visual modes with common Vim commands."
                )),
                if vim_enabled {
                    ToggleState::Selected
                } else {
                    ToggleState::Unselected
                },
                move |selection, _, cx| {
                    let enabled = match selection {
                        ToggleState::Selected => true,
                        ToggleState::Unselected => false,
                        ToggleState::Indeterminate => return,
                    };
                    update_settings_file(fs.clone(), cx, move |settings, _| {
                        settings.editor.vim_mode = Some(enabled);
                    });
                },
            )
            .tab_index(*tab_index - 1),
        )
}

fn render_worktree_auto_trust_switch(tab_index: &mut isize, cx: &mut App) -> impl IntoElement {
    let toggle_state = if ProjectSettings::get_global(cx).session.trust_all_worktrees {
        ui::ToggleState::Selected
    } else {
        ui::ToggleState::Unselected
    };

    let tooltip_description = t!(
        "Yo can only allow services like language servers, project settings, and MCP servers to run after you mark a new project as trusted."
    );

    SwitchField::new(
        "onboarding-auto-trust-worktrees",
        Some(t!("Trust All Projects By Default")),
        Some(t!(
            "Automatically mark all new projects as trusted to unlock all Yo's features"
        )),
        toggle_state,
        {
            let fs = <dyn Fs>::global(cx);
            move |&selection, _, cx| {
                let trust = match selection {
                    ToggleState::Selected => true,
                    ToggleState::Unselected => false,
                    ToggleState::Indeterminate => {
                        return;
                    }
                };
                update_settings_file(fs.clone(), cx, move |setting, _| {
                    setting.session.get_or_insert_default().trust_all_worktrees = Some(trust);
                });
            }
        },
    )
    .tab_index({
        *tab_index += 1;
        *tab_index - 1
    })
    .tooltip(Tooltip::text(tooltip_description))
}

#[cfg(target_os = "macos")]
fn render_install_cli_section(tab_index: &mut isize) -> impl IntoElement {
    *tab_index += 1;

    h_flex()
        .gap_2()
        .flex_wrap()
        .justify_between()
        .child(
            v_flex()
                .gap_0p5()
                .max_w_5_6()
                .child(Label::new(t!("Terminal Command")))
                .child(
                    Label::new(t!(
                        "Open files and folders from the terminal with the yo command"
                    ))
                    .color(Color::Muted),
                ),
        )
        .child(
            Button::new("install-cli", t!("Install CLI"))
                .style(ButtonStyle::OutlinedGhost)
                .size(ButtonSize::Medium)
                .label_size(LabelSize::Small)
                .tab_index(*tab_index - 1)
                .on_click(|_, window, cx| {
                    window.dispatch_action(install_cli::InstallCliBinary.boxed_clone(), cx);
                }),
        )
}

pub(crate) fn render_basics_page(cx: &mut App) -> impl IntoElement {
    let mut tab_index = 0;

    let page = v_flex()
        .id("basics-page")
        .gap_6()
        .child(render_theme_section(&mut tab_index, cx))
        .child(render_editing_section(&mut tab_index, cx));

    #[cfg(target_os = "macos")]
    let page = page.child(render_install_cli_section(&mut tab_index));

    page.child(render_worktree_auto_trust_switch(&mut tab_index, cx))
}
