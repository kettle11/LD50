use crate::*;

pub fn get_ui() -> impl Widget<World, StandardContext<World>> {
    let ui = stack((
        conditional(
            |world: &mut World, _| {
                world.get_singleton::<GameState>().game_mode == GameMode::GameOver
            },
            padding(stack((
                fill_pass_through(|_, _, _| Color::BLACK),
                center(
                    text("The ancient worm gnaws your bones for infinity... \nBut perhaps you can try again...?")
                        .with_size(|_, _, _| 50.),
                ),
            ))),
        ),
         conditional(
            |world: &mut World, _| {
                world.get_singleton::<GameState>().victory
            },
            stack((
                fill_pass_through(|_, _, _| Color::BLACK.with_alpha(0.2)),
                padding_with_amount(|_| 100., center(
                    text("Nevi is impressed by your bravery and has granted you the blessing of being spat into infinity instead of chewed for infinity.\n\n This is a great honor!\n\n Thanks for playing!")
                        .with_size(|_, _, _| 40.),
                ),
            ))),
        ),
        conditional(
            |world: &mut World, _| {
                world.get_singleton::<GameState>().game_mode == GameMode::Title
            },
            center(text("Last of the Sky Folk").with_size(|_, _, _| 100.).with_color(|_, _, _| Color::BLACK)),
        ),
        conditional(
            |world: &mut World, _| {
                world.get_singleton::<GameState>().game_mode == GameMode::Game
            },
            align(Alignment::Start, Alignment::End, padding(text(|world: &mut World| {
                (|game_state: &mut GameState| {
                    if game_state.victory {
                        return String::new();
                    }
                    match game_state.game_mode {
                        GameMode::Game => {
                            let y = game_state.player_max_height;
                            let messages = [
                                (46.0, "The Eternal Tower crumbles. Press space to jump"),
                                (48.0, "All have fled our home. Press space again to multi-jump."),
                                (50.0, "Click to use your grapple when your cursor turns white!"),
                                (60.0, "Collect the yellow orbs to increase your grapple length!"),
                                (150.0, "Alas, you must climb now! The worm has come..."),
                                (200.0, ""),
                                (400.0, "All others have fled or drowned in the Sea of Despair..."),
                                (600.0, ""),
                                (800.0, "Why did you return?"),
                                (1000.0, ""),
                                (1200.0, "Do you not fear the worm's endless gnaw?"),
                                (1600.0, ""),
                                (2800.0, "What is this?"),
                                (3000.0, "It appears the Above Ones have left a gift for you..."),
                                (3100.0, "Grab it and right-click to fire at the worm."),
                                (4000.0, ""),
                            ];
                            for message in messages.iter().rev() {
                                if message.0 < y {
                                    return message.1.into()
                                }
                            }
                            return String::new();
                        }
                        _ => String::new()
                    }

                })
                .try_run(world).unwrap_or_else(|_| String::new())
            })
                .with_size(|_, _, _| 52.)
                .with_color(|_, _, _| Color::BLACK.with_lightness(0.15))))
        ),
        conditional(
            |world: &mut World, _| {
                world.get_singleton::<GameState>().game_mode == GameMode::Game
            },
            stack((
                center(stack((
                    rectangle(Vec2::fill(4.0)),
                    fill(|world: &mut World, _, _| {
                        if world.get_singleton::<GameState>().can_grapple {
                            Color::WHITE
                        } else {
                            Color::BLACK
                        }
                    }),
                ))),
                align(
                    Alignment::End,
                    Alignment::End,
                    padding(
                        text(|world: &mut World| {
                            use num_format::{Locale, WriteFormatted};
                            // Cargo fmt, why don't you work? :( 

                                if let Ok(player_position) =
                                    (|player_transform: (&Transform, &CharacterController)| {
                                        player_transform.0.position
                                    })
                                    .try_run(world) {
                                        if player_position.y > 90_000.0 {
                                            "∞ m".into()
                                        } else {
                            // if player_position.y > 0.0 {
                                let mut writer = String::new();
                                let _ = writer.write_formatted(
                                    &(player_position.y.floor() as i32),
                                    &Locale::en,
                                );
                                format!("{} m", writer)
                            }
                            } else {
                                String::new()
                            }
                        })
                        .with_size(|_, _, _| 50.)
                        .with_color(|_, _, _| Color::BLACK.with_lightness(0.15)),
                    ),
                ),
            )),
        ),
    ));
    ui
}
