use bevy::prelude::*;

use crate::{menu::ButtonColors, GameState, MarkerGameStatePlaying};

pub struct MenuPlayingPlugin;

impl Plugin for MenuPlayingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_menu);
        app.add_systems(
            Update,
            click_play_button.run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup_menu(mut commands: Commands, button_colors: Res<ButtonColors>) {
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(120.0),
                    height: Val::Px(50.0),
                    left: Val::Px(30.),
                    top: Val::Px(30.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: button_colors.normal.into(),
                ..Default::default()
            },
            MarkerGameStatePlaying,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Back",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
                MarkerGameStatePlaying,
            ));
        });
}

fn click_play_button(
    button_colors: Res<ButtonColors>,
    mut state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                state.set(GameState::Menu);
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

pub fn enter_button(mut state: ResMut<NextState<GameState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Return) || keyboard_input.just_pressed(KeyCode::Space) {
        state.set(GameState::Playing);
    }
}
