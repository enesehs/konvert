use konvert::config::Config;

#[test]
fn test_config_regression_profile_preservation_under_json_update() {
    let raw_initial_toml = r#"
[general]
notification_enabled = true
language = "tr"

[context_menu]
enabled_actions = ["convert_webp", "convert_png"]

[image]
quality = 85
lossless = false

[video]
crf = 23
preset = "medium"
lossless = false
hwaccel = "auto"

[audio]
bitrate = "320k"

[document]
pdf_dpi = 300

[profiles.discord_preset]
quality = 70
bitrate = "96k"
lossless = false

[profiles.web_opt_extreme]
quality = 65
crf = 30
preset = "veryfast"
lossless = false

[custom_thirdparty_section]
feature_flag = true
version = 2
"#;

    // 1. Parse initial TOML containing custom profiles and unknown sections
    let mut config: Config =
        toml::from_str(raw_initial_toml).expect("Failed to deserialize initial configuration");

    assert!(config.profiles.contains_key("discord_preset"));
    assert!(config.profiles.contains_key("web_opt_extreme"));
    assert!(config.extra.contains_key("custom_thirdparty_section"));

    // 2. Simulate GUI payload saving via JSON
    let gui_save_payload = r#"{
        "general": {
            "notification_enabled": false,
            "language": "en"
        },
        "image": {
            "quality": 95,
            "lossless": true
        },
        "video": {
            "crf": 18,
            "preset": "slow",
            "lossless": false,
            "hwaccel": "nvenc"
        },
        "context_menu": {
            "enabled_actions": ["convert_avif", "convert_jxl"]
        }
    }"#;

    config
        .update_from_json(gui_save_payload)
        .expect("Failed to apply GUI update payload");

    // 3. Serialize back to TOML
    let serialized_toml =
        toml::to_string_pretty(&config).expect("Failed to serialize updated configuration");

    // 4. Reload from serialized TOML and assert zero data loss
    let reloaded: Config =
        toml::from_str(&serialized_toml).expect("Failed to parse serialized TOML");

    // Assert GUI changes took effect
    assert_eq!(reloaded.general.notification_enabled, false);
    assert_eq!(reloaded.general.language.as_deref(), Some("en"));
    assert_eq!(reloaded.image.quality, 95);
    assert_eq!(reloaded.image.lossless, true);
    assert_eq!(reloaded.video.crf, 18);
    assert_eq!(reloaded.video.preset, "slow");
    assert_eq!(reloaded.video.hwaccel, "nvenc");
    assert_eq!(
        reloaded.context_menu.enabled_actions,
        vec!["convert_avif", "convert_jxl"]
    );

    // CRITICAL: Assert profiles and external sections were NOT destroyed
    assert!(
        reloaded.profiles.contains_key("discord_preset"),
        "REGRESSION: discord_preset was deleted!"
    );
    assert_eq!(reloaded.profiles["discord_preset"].quality, Some(70));
    assert_eq!(
        reloaded.profiles["discord_preset"].bitrate.as_deref(),
        Some("96k")
    );

    assert!(
        reloaded.profiles.contains_key("web_opt_extreme"),
        "REGRESSION: web_opt_extreme was deleted!"
    );
    assert_eq!(reloaded.profiles["web_opt_extreme"].quality, Some(65));
    assert_eq!(reloaded.profiles["web_opt_extreme"].crf, Some(30));

    assert!(
        reloaded.extra.contains_key("custom_thirdparty_section"),
        "REGRESSION: custom_thirdparty_section was deleted!"
    );
}

#[test]
fn test_config_regression_cli_set_preserves_profiles() {
    let raw_initial_toml = r#"
[general]
notification_enabled = true

[context_menu]
enabled_actions = []

[image]
quality = 85
lossless = false

[video]
crf = 23
preset = "medium"
lossless = false

[audio]
bitrate = "320k"

[document]
pdf_dpi = 300

[profiles.my_saved_preset]
quality = 45
lossless = true
"#;

    let mut config: Config = toml::from_str(raw_initial_toml).unwrap();
    assert!(config.profiles.contains_key("my_saved_preset"));

    // Mutate via CLI set_key_value
    config.set_key_value("image.quality", "50").unwrap();
    config.set_key_value("video.hwaccel", "vaapi").unwrap();

    let serialized = toml::to_string_pretty(&config).unwrap();
    let reloaded: Config = toml::from_str(&serialized).unwrap();

    assert_eq!(reloaded.image.quality, 50);
    assert_eq!(reloaded.video.hwaccel, "vaapi");
    assert!(
        reloaded.profiles.contains_key("my_saved_preset"),
        "REGRESSION: CLI set deleted my_saved_preset!"
    );
    assert_eq!(reloaded.profiles["my_saved_preset"].quality, Some(45));
}
