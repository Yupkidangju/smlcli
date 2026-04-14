use crate::domain::settings::PersistedSettings;
use crate::domain::permissions::{ShellPolicy, FileWritePolicy, NetworkPolicy};

#[test]
fn test_default_settings_flow() {
    let settings = PersistedSettings::default();
    assert_eq!(settings.version, 1);
    assert_eq!(settings.default_provider, "OpenRouter");
    assert_eq!(settings.default_model, "auto");
    assert_eq!(settings.shell_policy, ShellPolicy::Ask);
    assert_eq!(settings.file_write_policy, FileWritePolicy::AlwaysAsk);
    assert_eq!(settings.network_policy, NetworkPolicy::ProviderOnly);
}

#[test]
fn test_wizard_state_transitions() {
    use crate::app::state::{WizardState, WizardStep};
    let mut wizard = WizardState::new();
    assert_eq!(wizard.step, WizardStep::ProviderSelection);
    
    wizard.step = WizardStep::ApiKeyInput;
    assert_eq!(wizard.step, WizardStep::ApiKeyInput);
}
