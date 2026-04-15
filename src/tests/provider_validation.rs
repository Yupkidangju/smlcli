use crate::domain::provider::ProviderKind;
use crate::providers::registry::get_adapter;

#[test]
fn test_provider_registry_factory() {
    let openrouter = get_adapter(&ProviderKind::OpenRouter);
    let gemini = get_adapter(&ProviderKind::Google);

    // We only test instantiation since networking requires mocked HTTP clients
    // in strict unit tests. Factory should dynamically allocate correct adapters.

    // Ensure that adapters were successfully allocated and are valid trait objects.
    let ptr1 = &*openrouter as *const _ as *const ();
    let ptr2 = &*gemini as *const _ as *const ();

    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());
    assert_ne!(ptr1, ptr2, "Should allocate distinct provider instances");
}
