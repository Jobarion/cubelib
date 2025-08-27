use crate::util::use_local_storage;
use crate::util::RwSignalTup;
use cubelib::algs::Algorithm;
use cubelib::cube::*;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::IoIcon;
use std::str::FromStr;

#[derive(Clone)]
pub struct SettingsState {
    advanced: RwSignalTup<bool>,
    relative_step_length: RwSignalTup<bool>,
    additional_triggers: RwSignalTup<Vec<Algorithm>>,
}

impl SettingsState {
    pub fn is_advanced(&self) -> bool {
        self.advanced.0.get()
    }

    pub fn advanced(&self) -> Signal<bool> {
        self.advanced.0
    }

    pub fn is_relative(&self) -> bool {
        self.relative_step_length.0.get()
    }

    pub fn relative(&self) -> Signal<bool> {
        self.relative_step_length.0
    }

    pub fn additional_triggers(&self) -> Signal<Vec<String>> {
        let triggers = self.additional_triggers.0.clone();
        Signal::derive(move || triggers.get().into_iter().map(|x| x.to_string()).collect())
    }
}

impl SettingsState {
    pub fn from_local_storage() -> Self {
        let advanced = use_local_storage("settings-advanced", false);
        let rel_step_len = use_local_storage("settings-rel-step-len", true);
        let additional_triggers =
            use_local_storage("settings-additional-triggers", Vec::<Algorithm>::new());
        Self {
            advanced,
            relative_step_length: rel_step_len,
            additional_triggers,
        }
    }
}

#[component]
pub fn SettingsComponent(active: ReadSignal<bool>, set_active: WriteSignal<bool>) -> impl IntoView {
    let settings = use_context::<SettingsState>().expect("Settings context required");
    let (cur_trig, cur_trig_set) = create_signal("".to_string());

    let is_trigger_valid = Signal::derive(move || {
        if let Ok(alg) = Algorithm::from_str(cur_trig.get().as_str()) {
            if !alg.inverse_moves.is_empty() {
                return false;
            }
            if alg.len() == 0 {
                return false;
            }
            if alg.normal_moves[0] != Turn333::R {
                return false;
            }
            let last = alg.normal_moves[alg.normal_moves.len() - 1];
            if last != Turn333::R && last != Turn333::L {
                return false;
            }
            true
        } else {
            false
        }
    });

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
                <SettingsOptionSmall
                    label="Advanced mode:"
                    description="Shows advanced options that are otherwise hidden to make Mallard easier to understand."
                >
                    <Toggle
                        state=settings.advanced.0
                        set_state=settings.advanced.1
                    />
                </SettingsOptionSmall>
                <SettingsOptionSmall
                    label="Relative step length:"
                    description="By default the step length is absolute. This means RZP in 6 moves means EO and RZP together have to be in 6 moves. When enabled, RZP in 3 means RZP can take 3 moves, regardless of the EO length.">
                    <Toggle
                        state=settings.relative_step_length.0
                        set_state=settings.relative_step_length.1
                    />
                </SettingsOptionSmall>
                <div>
                    <SettingsOptionSmall
                        label="Additional Triggers:"
                        description="Configure additional triggers to select in the DR step. Triggers are rotated and mirrored to all positions and must start with an R and end with an R or L."
                    >
                        <TextInput
                            get=cur_trig
                            set=cur_trig_set placeholder="R U L"
                            style="width: 150px"
                            class=move|| if is_trigger_valid.get() || cur_trig.get().is_empty() { "" } else { "leptonic-input-invalid" }
                        />
                        <button
                            enabled=move||false
                            on:click=move|_|{
                                if is_trigger_valid.get()  {
                                    let mut triggers = settings.additional_triggers.0.get();
                                    let alg = Algorithm::from_str(cur_trig.get().as_str()).unwrap();
                                    if !triggers.contains(&alg) {
                                        triggers.push(alg);
                                        settings.additional_triggers.1.set(triggers);
                                    }
                                    cur_trig_set.set("".to_string());
                                }
                            }
                            class="icon-button"
                            style:cursor=move||if is_trigger_valid.get() { "pointer" } else { "default" }
                            style:opacity="60%"
                            style:font-size="30px">
                            <Icon icon=IoIcon::IoAddOutline/>
                        </button>
                    </SettingsOptionSmall>
                    <TriggerList triggers=settings.additional_triggers.0 triggers_set=settings.additional_triggers.1 />
                </div>
            </ModalBody>
            <ModalFooter>""</ModalFooter>
        </Modal>
    }
}

#[component]
pub fn SettingsOptionSmall(
    #[prop(into)] label: String,
    #[prop(into, optional)] description: Option<String>,
    children: Children,
) -> impl IntoView {
    let description = description
        .map(|d| view! {<div class="settings-description">{d}</div>}.into_view())
        .unwrap_or(view! {}.into_view());

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

#[component]
fn TriggerList(
    triggers: Signal<Vec<Algorithm>>,
    triggers_set: WriteSignal<Vec<Algorithm>>,
) -> impl IntoView {
    view! {
        <div style:width="500px">
        {move || {
            triggers.get()
                .iter()
                .map(|alg| alg.to_string())
                .map(|alg| {
                    let alg_c = alg.clone();
                    view! {
                        <Chip color=ChipColor::Secondary dismissible=move |_| {
                            triggers_set.set(triggers.get()
                                .into_iter()
                                .filter(|x|!alg_c.eq(&x.to_string()))
                                .collect());
                        }>
                            {alg}
                        </Chip>
                    }
                })
                .collect_view()
        }}
        </div>
    }
}
