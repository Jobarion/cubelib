use std::fmt::{Display, Formatter};
use cubelib::cube::Axis;
use leptos::*;
use leptos::html::*;
use leptonic::prelude::*;
use crate::step::VariantAxis::{FB, LR, UD};
use crate::util;

#[derive(Copy, Clone)]
pub struct EOConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
    pub niss: RwSignal<NissType>,
    pub variants: RwSignal<Vec<VariantAxis>>
}

#[derive(Copy, Clone)]
pub struct RZPConfig {
   pub min: RwSignal<u8>,
   pub max: RwSignal<u8>,
   pub niss: RwSignal<NissType>,
}

#[derive(Copy, Clone)]
pub struct DRConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
    pub niss: RwSignal<NissType>,
    pub variants: RwSignal<Vec<VariantAxis>>,
    pub triggers: RwSignal<Vec<String>>,
}

#[derive(Copy, Clone)]
pub struct  HTRConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
    pub niss: RwSignal<NissType>,
    pub variants: RwSignal<Vec<VariantAxis>>
}

#[derive(Copy, Clone)]
pub struct  FRConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
    pub niss: RwSignal<NissType>,
    pub variants: RwSignal<Vec<VariantAxis>>
}

#[derive(Copy, Clone)]
pub struct FinishConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
}

impl FinishConfig {
    pub fn new(cx: Scope, enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min: create_rw_signal(cx, 0),
            max: create_rw_signal(cx, 10),
        }
    }
}

impl DRConfig {
    pub fn new(cx: Scope, enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min: create_rw_signal(cx, 0),
            max: create_rw_signal(cx, 12),
            niss: create_rw_signal(cx, NissType::Before),
            variants: create_rw_signal(cx, vec![UD, FB, LR]),
            triggers: create_rw_signal(cx, vec!["R".to_string(), "R U2 R".to_string(), "R U R".to_string(), "R U' R".to_string()]),
        }
    }
}

impl RZPConfig {
    pub fn new(cx: Scope) -> Self {
        Self {
            min: create_rw_signal(cx, 0),
            max: create_rw_signal(cx, 3),
            niss: create_rw_signal(cx, NissType::Never),
        }
    }
}

impl EOConfig {
    pub fn new(cx: Scope, enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min: create_rw_signal(cx, 0),
            max: create_rw_signal(cx, 5),
            niss: create_rw_signal(cx, NissType::Always),
            variants: create_rw_signal(cx, vec![UD, FB, LR]),
        }
    }
}

impl HTRConfig {
    pub fn new(cx: Scope, enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min: create_rw_signal(cx, 0),
            max: create_rw_signal(cx, 12),
            niss: create_rw_signal(cx, NissType::Before),
            variants: create_rw_signal(cx, vec![UD, FB, LR]),
        }
    }
}

impl FRConfig {
    pub fn new(cx: Scope, enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min: create_rw_signal(cx, 0),
            max: create_rw_signal(cx, 10),
            niss: create_rw_signal(cx, NissType::Before),
            variants: create_rw_signal(cx, vec![UD, FB, LR]),
        }
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum VariantAxis {
    UD,
    FB,
    LR
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum NissType {
    Never,
    Before,
    Always
}

impl Display for VariantAxis {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UD => write!(f, "UD"),
            Self::FB => write!(f, "FB"),
            Self::LR => write!(f, "LR"),
        }
    }
}

#[component]
pub fn StepsComponent(cx: Scope) -> impl IntoView {
    let dr_enabled = use_context::<DRConfig>(cx).expect("DR context required").enabled;
    let htr_enabled = use_context::<HTRConfig>(cx).expect("HTR context required").enabled;
    let fr_enabled = use_context::<FRConfig>(cx).expect("FR context required").enabled;
    let fin_enabled = use_context::<FinishConfig>(cx).expect("Finish context required").enabled;
    view! {cx,
        <Tabs mount=Mount::Once>
            <Tab name="eo" label="EO">
                <EOParameters/>
            </Tab>
            <Tab name="dr" label=move || view! {cx, <span style:opacity={if dr_enabled.0.get() { 1.0 } else { 0.5 }}>"DR"</span>}>
                <StepEnableComponent state=dr_enabled.0 set_state=dr_enabled.1 />
                <div class:grayed-out=move || !dr_enabled.0.get()>
                    <DRParameters/>
                </div>
            </Tab>
            <Tab name="htr" label=move || view! {cx, <span style:opacity={if htr_enabled.0.get() { 1.0 } else { 0.5 }}>"HTR"</span>}>
                <StepEnableComponent state=htr_enabled.0 set_state=htr_enabled.1 />
                <div class:grayed-out=move || !htr_enabled.0.get()>
                    <HTRParameters/>
                </div>
            </Tab>
            <Tab name="fr" label=move || view! {cx, <span style:opacity={if fr_enabled.0.get() { 1.0 } else { 0.5 }}>"FR"</span>}>
                <StepEnableComponent state=fr_enabled.0 set_state=fr_enabled.1 />
                <div class:grayed-out=move || !fr_enabled.0.get()>
                    <FRParameters/>
                </div>
            </Tab>
            <Tab name="fin" label=move || view! {cx, <span style:opacity={if fin_enabled.0.get() { 1.0 } else { 0.5 }}>"Finish"</span>}>
                <StepEnableComponent state=fin_enabled.0 set_state=fin_enabled.1 />
                <div class:grayed-out=move || !fin_enabled.0.get()>
                    <FinishParameters/>
                </div>
            </Tab>
        </Tabs>
    }
}

#[component]
pub fn EOParameters(cx: Scope) -> impl IntoView {

    let eo_config = use_context::<EOConfig>(cx).expect("EO context required");

    view! {cx,
        <DefaultStepParameters
            niss_default=eo_config.niss
            min=eo_config.min
            max=eo_config.max
            set_min=eo_config.min
            set_max=eo_config.max
            total_max=8
            variants=eo_config.variants
        />
    }
}

#[component]
pub fn DRParameters(cx: Scope) -> impl IntoView {
    let dr_config = use_context::<DRConfig>(cx).expect("DR context required");
    let rzp_config = use_context::<RZPConfig>(cx).expect("RZP context required");

    view! {cx,
        <DefaultStepParameters
            niss_default=dr_config.niss
            min=dr_config.min
            max=dr_config.max
            set_min=dr_config.min
            set_max=dr_config.max
            total_max=12
            variants=dr_config.variants
        />
        <h4>"Triggers"</h4>
        <Multiselect
            options=vec!["R".to_string(), "R U2 R".to_string(), "R U R".to_string(), "R U' R".to_string(), "R L".to_string()]
            search_text_provider=create_callback(cx, move |o| format!("{o}"))
            render_option=create_callback(cx, move |(_cx, o)| format!("{o}"))
            selected=dr_config.triggers
            set_selected=create_callback(cx, move |v| dr_config.triggers.set(v))
        />
        <div class:grayed-out=move ||dr_config.triggers.get().is_empty()>
            <h2>"RZP"</h2>
            <h4>"Step length"</h4>
            <StepLengthComponent
                min=rzp_config.min
                max=rzp_config.max
                set_min=rzp_config.min
                set_max=rzp_config.max
                total_max=5
            />
            <h4>"NISS"</h4>
            <NissSettingsComponent niss_default=rzp_config.niss/>
        </div>
    }
}

#[component]
pub fn HTRParameters(cx: Scope) -> impl IntoView {
    let htr_config = use_context::<HTRConfig>(cx).expect("HTR context required");

    view! {cx,

        <DefaultStepParameters
            niss_default=htr_config.niss
            min=htr_config.min
            max=htr_config.max
            set_min=htr_config.min
            set_max=htr_config.max
            total_max=14
            variants=htr_config.variants
        />
    }
}

#[component]
pub fn FRParameters(cx: Scope) -> impl IntoView {
    let fr_config = use_context::<FRConfig>(cx).expect("FR context required");
    view! {cx,

        <DefaultStepParameters
            niss_default=fr_config.niss
            min=fr_config.min
            max=fr_config.max
            set_min=fr_config.min
            set_max=fr_config.max
            total_max=10
            variants=fr_config.variants
        />
    }
}

#[component]
pub fn FinishParameters(cx: Scope) -> impl IntoView {

    let fin_config = use_context::<FinishConfig>(cx).expect("Finish context required");
    view! {cx,
        <StepLengthComponent
            min=fin_config.min
            max=fin_config.max
            set_min=fin_config.min
            set_max=fin_config.max
            total_max=10
        />
    }
}

#[component]
pub fn StepEnableComponent(cx: Scope, #[prop(into)] state: MaybeSignal<bool>, #[prop(into)] set_state: Out<bool>) -> impl IntoView {
    view! {cx,
        <div style="display: flex; align-items: center;">
            <label style="margin-right: 10px;">"Enable step:"</label>
            <Toggle state=state set_state=set_state />
        </div>
    }
}

#[component]
pub fn StepLengthComponent(
    cx: Scope,
    total_max: u8,
    min: RwSignal<u8>,
    max: RwSignal<u8>,
    set_min: RwSignal<u8>,
    set_max: RwSignal<u8>,
) -> impl IntoView {
    view! {cx,
        <RangeSlider
            value_a=Signal::derive(cx, move||min.get() as f64)
            value_b=Signal::derive(cx, move||max.get() as f64)
            set_value_a=Callback::new(cx, move|x|set_min.set(x as u8))
            set_value_b=Callback::new(cx, move|x|set_max.set(x as u8))
            min=0.0
            max={total_max as f64}
            step=1.0
            marks=SliderMarks::Automatic { create_names: true }
            value_display=create_callback(cx, move |v| format!("{v:.0}"))
        />
    }
}

#[component]
pub fn NissSettingsComponent(cx: Scope, niss_default: RwSignal<NissType>) -> impl IntoView {
    let niss_1 = Signal::derive(cx, move || niss_default.get() != NissType::Never);
    let niss_2 = Signal::derive(cx, move || niss_default.get() == NissType::Always);
    view! {cx,
        <div style="display: flex; align-items: center; margin-bottom: 5px;">
            <label style="margin-right: 10px;">"Allow switching before step:"</label>
            <Toggle
                state=niss_1
                set_state=Callback::new(cx, move |s| if s { niss_default.set(NissType::Before) } else { niss_default.set(NissType::Never)})
            />
        </div>
        <div style="display: flex; align-items: center;" class:grayed-out=move || !niss_1.get()>
            <label style="margin-right: 10px;">"Allow switching during step:"</label>
            <Toggle
                state=niss_2
                set_state=Callback::new(cx, move |s| if s { niss_default.set(NissType::Always) } else { niss_default.set(NissType::Before)})
            />
        </div>
    }
}

#[component]
pub fn DefaultStepParameters(cx: Scope,
                             total_max: u8,
                             min: RwSignal<u8>,
                             max: RwSignal<u8>,
                             set_min: RwSignal<u8>,
                             set_max: RwSignal<u8>,
                             niss_default: RwSignal<NissType>,
                             variants: RwSignal<Vec<VariantAxis>>
                             // #[prop(into, optional)] set_niss_type: OptionalMaybeSignal<NissType>,
) -> impl IntoView {
    view! {cx,
        <h4>"Step length"</h4>
        <StepLengthComponent min=min max=max set_min=set_min set_max=set_max total_max=total_max/>
        <h4>"Variations"</h4>
        <Multiselect
            options=vec![VariantAxis::UD, VariantAxis::FB, VariantAxis::LR]
            search_text_provider=create_callback(cx, move |o| format!("{o}"))
            render_option=create_callback(cx, move |(_cx, o)| format!("{o:?}"))
            selected=variants
            set_selected=create_callback(cx, move |v| variants.set(v))
        />
        <h4>"NISS"</h4>
        <NissSettingsComponent niss_default=niss_default/>
    }
}