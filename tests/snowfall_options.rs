//! Integration tests for the Christmas snowfall option wiring.
//!
//! These do not invoke FFmpeg, they only check that the option flows from the
//! public API into the generator configuration as expected.

use slideshow_generator::{SlideshowGenerator, SlideshowOptions, SnowfallOptions};

#[test]
fn snowfall_is_disabled_by_default() {
    let options = SlideshowOptions::new();
    assert!(options.snowfall.is_none());
}

#[test]
fn with_snowfall_enabled_turns_on_the_effect() {
    let options = SlideshowOptions::new().with_snowfall_enabled();
    let snowfall = options.snowfall.expect("snowfall should be enabled");
    assert_eq!(snowfall, SnowfallOptions::new());
}

#[test]
fn with_snowfall_accepts_a_configuration() {
    let snowfall = SnowfallOptions::new().with_density(320.0).with_wind(-1.5);
    let options = SlideshowOptions::new().with_snowfall(Some(snowfall.clone()));

    assert_eq!(options.snowfall, Some(snowfall));
}

#[test]
fn with_snowfall_accepts_none_to_disable() {
    let options = SlideshowOptions::new()
        .with_snowfall_enabled()
        .with_snowfall(None);

    assert!(options.snowfall.is_none());
}

#[test]
fn snowfall_survives_the_generator() {
    let snowfall = SnowfallOptions::new().with_speed(3.0);
    let options = SlideshowOptions::new().with_snowfall(Some(snowfall.clone()));
    let generator = SlideshowGenerator::with_options(options);

    assert_eq!(generator.options().snowfall, Some(snowfall));
}

#[test]
fn snowfall_can_be_changed_at_runtime() {
    let mut generator = SlideshowGenerator::new();
    assert!(generator.options().snowfall.is_none());

    let snowfall = SnowfallOptions::new().with_density(500.0);
    generator.set_options(SlideshowOptions::new().with_snowfall(Some(snowfall.clone())));

    assert_eq!(generator.options().snowfall, Some(snowfall));
}

#[test]
fn snowfall_parameters_round_trip_through_the_builder() {
    let snowfall = SnowfallOptions::new()
        .with_density(150.0)
        .with_speed(1.75)
        .with_wind(-0.4)
        .with_flake_size(11.0)
        .with_opacity(0.6)
        .with_seed(1234);

    assert_eq!(snowfall.density, 150.0);
    assert_eq!(snowfall.speed, 1.75);
    assert_eq!(snowfall.wind, -0.4);
    assert_eq!(snowfall.flake_size, 11.0);
    assert_eq!(snowfall.opacity, 0.6);
    assert_eq!(snowfall.seed, 1234);
    assert!(snowfall.validate().is_ok());
}

#[test]
fn randomized_options_are_reproducible_from_a_seed() {
    let first = SnowfallOptions::randomized(99);
    let second = SnowfallOptions::randomized(99);

    assert_eq!(first, second);
    assert_eq!(first.seed, 99);
}

#[test]
fn randomized_options_differ_between_seeds() {
    let first = SnowfallOptions::randomized(1);
    let second = SnowfallOptions::randomized(2);

    assert_ne!(first, second);
}

#[test]
fn every_randomized_option_is_renderable() {
    // A random render must never come out invalid, which is what makes the
    // effect safe to expose as a bare toggle.
    for seed in 0..500 {
        let options = SnowfallOptions::randomized(seed);
        assert!(options.validate().is_ok(), "seed {} invalid", seed);
        assert!(options.density > 0.0, "seed {} had no snow", seed);
        assert!(options.opacity > 0.5, "seed {} snow too faint", seed);
    }
}
