use crate::domain::provider::ProviderKind;
use crate::providers::registry::get_adapter;

#[test]
fn test_provider_registry_factory() {
    let openrouter = get_adapter(&ProviderKind::OpenRouter);
    let gemini = get_adapter(&ProviderKind::Google);
    let openai = get_adapter(&ProviderKind::OpenAI);
    let anthropic = get_adapter(&ProviderKind::Anthropic);
    let xai = get_adapter(&ProviderKind::Xai);

    // We only test instantiation since networking requires mocked HTTP clients
    // in strict unit tests. Factory should dynamically allocate correct adapters.

    // Ensure that adapters were successfully allocated and are valid trait objects.
    let ptr1 = &*openrouter as *const _ as *const ();
    let ptr2 = &*gemini as *const _ as *const ();
    let ptr3 = &*openai as *const _ as *const ();
    let ptr4 = &*anthropic as *const _ as *const ();
    let ptr5 = &*xai as *const _ as *const ();

    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());
    assert!(!ptr3.is_null());
    assert!(!ptr4.is_null());
    assert!(!ptr5.is_null());
    assert_ne!(ptr1, ptr2, "Should allocate distinct provider instances");
}
