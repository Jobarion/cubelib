use leptonic::prelude::*;
use leptos::*;
use leptos_icons::IoIcon;
use leptos_use::storage::use_local_storage;

#[derive(Clone)]
pub struct SettingsState {
    advanced: (Signal<bool>, WriteSignal<bool>),
    absolute_step_length: (Signal<bool>, WriteSignal<bool>),
}

impl SettingsState {
    pub fn from_local_storage() -> Self {
        let advanced = use_local_storage("settings-advanced", false);
        let abs_step_len = use_local_storage("settings-rel-step-len", true);
        Self {
            advanced: (advanced.0, advanced.1),
            absolute_step_length: (abs_step_len.0, abs_step_len.1),
        }
    }
}

#[component]
pub fn SettingsComponent(active: ReadSignal<bool>, set_active: WriteSignal<bool>) -> impl IntoView {

    let settings = use_context::<SettingsState>().expect("Settings context required");
    view! {
        <Modal show_when=active>
            <div style:position="relative">
                <button
                    on:click=move|_|set_active.set(false)
                    class="icon-button"
                    style:position="absolute"
                    style:top="-5px"
                    style:right="-5px"
                    style:opacity="40%"
                    style:font-size="28px">
                    <Icon icon=IoIcon::IoCloseOutline/>
                </button>
            </div>
            <ModalHeader><ModalTitle>"Preferences"</ModalTitle></ModalHeader>
            <ModalBody>
                <SettingsOption
                    label="Advanced mode:"
                    description="Shows advanced options that are otherwise hidden to make Mallard easier to understand"
                >
                    <Toggle
                        state=settings.advanced.0
                        set_state=settings.advanced.1
                    />
                </SettingsOption>
                <SettingsOption
                    label="Relative step length:"
                    description="By default the step length is absolute. This means RZP in 6 moves means EO and RZP together have to be in 6 moves. When enabled, RZP in 3 means RZP can take 3 moves, regardless of the EO length.">
                    <Toggle
                        state=settings.absolute_step_length.0
                        set_state=settings.absolute_step_length.1
                    />
                </SettingsOption>
            </ModalBody>
            <ModalFooter>""</ModalFooter>
        </Modal>
    }
}

#[component]
pub fn SettingsOption(
    #[prop(into)] label: String,
    #[prop(into, optional)] description: Option<String>,
    children: Children) -> impl IntoView {

    let description = description
        .map(|d| view! {<div class="settings-description">{d}</div>}.into_view())
        .unwrap_or(view!{}.into_view());

    view! {
        <div class="settings-container">
            <div class="settings-option">
                <label class="settings-label">{label}</label>
                {children()}
            </div>
            {description}
        </div>
    }
}
