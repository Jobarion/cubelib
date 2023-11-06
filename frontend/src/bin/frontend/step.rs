use std::fmt::{Display, Formatter};

use cubelib::cube::Axis;
use cubelib::defs::NissSwitchType;
use leptonic::prelude::*;
use leptos::*;

#[derive(Copy, Clone)]
pub struct EOConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
    pub niss: RwSignal<NissSwitchType>,
    pub variants: RwSignal<Vec<Axis>>
}

#[derive(Copy, Clone)]
pub struct RZPConfig {
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
    pub niss: RwSignal<NissSwitchType>,
}

#[derive(Copy, Clone)]
pub struct DRConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
    pub niss: RwSignal<NissSwitchType>,
    pub variants: RwSignal<Vec<Axis>>,
    pub triggers: RwSignal<Vec<String>>,
}

#[derive(Copy, Clone)]
pub struct  HTRConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
    pub niss: RwSignal<NissSwitchType>,
    pub variants: RwSignal<Vec<Axis>>
}

#[derive(Copy, Clone)]
pub struct  FRConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
    pub niss: RwSignal<NissSwitchType>,
    pub variants: RwSignal<Vec<Axis>>
}

#[derive(Copy, Clone)]
pub struct FinishConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min: RwSignal<u8>,
    pub max: RwSignal<u8>,
    pub leave_slice: RwSignal<bool>,
}

impl FinishConfig {
    pub fn new(enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min: create_rw_signal(0),
            max: create_rw_signal(10),
            leave_slice: create_rw_signal(false),
        }
    }
}

impl DRConfig {
    pub fn new(enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min: create_rw_signal(0),
            max: create_rw_signal(12),
            niss: create_rw_signal(NissSwitchType::Before),
            variants: create_rw_signal(vec![Axis::UD, Axis::FB, Axis::LR]),
            triggers: create_rw_signal(vec!["R".to_string(), "R U2 R".to_string(), "R F2 R".to_string(), "R U R".to_string(), "R U' R".to_string()]),
        }
    }
}

impl RZPConfig {
    pub fn new() -> Self {
        Self {
            min: create_rw_signal(0),
            max: create_rw_signal(3),
            niss: create_rw_signal(NissSwitchType::Never),
        }
    }
}

impl EOConfig {
    pub fn new(enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min: create_rw_signal(0),
            max: create_rw_signal(5),
            niss: create_rw_signal(NissSwitchType::Always),
            variants: create_rw_signal(vec![Axis::UD, Axis::FB, Axis::LR]),
        }
    }
}

impl HTRConfig {
    pub fn new(enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min: create_rw_signal(0),
            max: create_rw_signal(12),
            niss: create_rw_signal(NissSwitchType::Before),
            variants: create_rw_signal(vec![Axis::UD, Axis::FB, Axis::LR]),
        }
    }
}

impl FRConfig {
    pub fn new(enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min: create_rw_signal(0),
            max: create_rw_signal(10),
            niss: create_rw_signal(NissSwitchType::Before),
            variants: create_rw_signal(vec![Axis::UD, Axis::FB, Axis::LR]),
        }
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum SelectableAxis {
    UD,
    FB,
    LR
}

impl Into<Axis> for SelectableAxis {
    fn into(self) -> Axis {
        match self {
            Self::UD => Axis::UD,
            Self::FB => Axis::FB,
            Self::LR => Axis::LR,
        }
    }
}

impl From<Axis> for SelectableAxis {
    fn from(value: Axis) -> Self {
        match value {
            Axis::UD => Self::UD,
            Axis::FB => Self::FB,
            Axis::LR => Self::LR,
        }
    }
}

impl Display for SelectableAxis {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UD => write!(f, "UD"),
            Self::FB => write!(f, "FB"),
            Self::LR => write!(f, "LR"),
        }
    }
}

#[component]
pub fn StepsComponent() -> impl IntoView {
    let dr_enabled = use_context::<DRConfig>().expect("DR context required").enabled;
    let htr_enabled = use_context::<HTRConfig>().expect("HTR context required").enabled;
    let fr_enabled = use_context::<FRConfig>().expect("FR context required").enabled;
    let fin_enabled = use_context::<FinishConfig>().expect("Finish context required").enabled;
    view! {
        <Tabs mount=Mount::Once>
            <Tab name="eo" label="EO".into_view()>
                <EOParameters/>
            </Tab>
            <Tab name="dr" label=view! {<span style:opacity=move||{if dr_enabled.0.get() { 1.0 } else { 0.5 }}>"DR"</span>}.into_view()>
                <StepEnableComponent state=dr_enabled.0 set_state=dr_enabled.1 />
                <div class:grayed-out=move || !dr_enabled.0.get()>
                    <DRParameters/>
                </div>
            </Tab>
            <Tab name="htr" label=view! {<span style:opacity=move||{if htr_enabled.0.get() { 1.0 } else { 0.5 }}>"HTR"</span>}.into_view()>
                <StepEnableComponent state=htr_enabled.0 set_state=htr_enabled.1 />
                <div class:grayed-out=move || !htr_enabled.0.get()>
                    <HTRParameters/>
                </div>
            </Tab>
            <Tab name="fr" label=view! {<span style:opacity=move||{if fr_enabled.0.get() { 1.0 } else { 0.5 }}>"FR"</span>}.into_view()>
                <StepEnableComponent state=fr_enabled.0 set_state=fr_enabled.1 />
                <div class:grayed-out=move || !fr_enabled.0.get()>
                    <FRParameters/>
                </div>
            </Tab>
            <Tab name="fin" label=view! {<span style:opacity=move||{if fin_enabled.0.get() { 1.0 } else { 0.5 }}>"Finish"</span>}.into_view()>
                <StepEnableComponent state=fin_enabled.0 set_state=fin_enabled.1 />
                <div class:grayed-out=move || !fin_enabled.0.get()>
                    <FinishParameters/>
                </div>
            </Tab>
        </Tabs>
    }
}

#[component]
pub fn EOParameters() -> impl IntoView {

    let eo_config = use_context::<EOConfig>().expect("EO context required");

    view! {
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
pub fn DRParameters() -> impl IntoView {
    let dr_config = use_context::<DRConfig>().expect("DR context required");
    let rzp_config = use_context::<RZPConfig>().expect("RZP context required");

    view! {
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
            options=vec!["R".to_string(), "R U2 R".to_string(), "R F2 R".to_string(), "R U R".to_string(), "R U' R".to_string(), "R L".to_string()]
            search_text_provider=move |o| format!("{o}")
            render_option=move |o| format!("{o}").into_view()
            selected=dr_config.triggers
            set_selected=move |v| dr_config.triggers.set(v)
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
pub fn HTRParameters() -> impl IntoView {
    let htr_config = use_context::<HTRConfig>().expect("HTR context required");

    view! {

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
pub fn FRParameters() -> impl IntoView {
    let fr_config = use_context::<FRConfig>().expect("FR context required");
    view! {

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
pub fn FinishParameters() -> impl IntoView {
    let fin_config = use_context::<FinishConfig>().expect("Finish context required");
    view! {
        <StepLengthComponent
            min=fin_config.min
            max=fin_config.max
            set_min=fin_config.min
            set_max=fin_config.max
            total_max=10
        />
        <div style="display: flex; align-items: center;">
            <label style="margin-right: 10px;">"Leave slice:"</label>
            <Toggle
                state=Signal::derive(move || fin_config.leave_slice.get())
                set_state=Callback::new(move |s| fin_config.leave_slice.set(s))
            />
        </div>
    }
}

#[component]
pub fn StepEnableComponent(#[prop(into)] state: MaybeSignal<bool>, #[prop(into)] set_state: Out<bool>) -> impl IntoView {
    view! {
        <div style="display: flex; align-items: center;">
            <label style="margin-right: 10px;">"Enable step:"</label>
            <Toggle state=state set_state=set_state />
        </div>
    }
}

#[component]
pub fn StepLengthComponent(
    total_max: u8,
    min: RwSignal<u8>,
    max: RwSignal<u8>,
    set_min: RwSignal<u8>,
    set_max: RwSignal<u8>,
) -> impl IntoView {
    view! {
        <RangeSlider
            value_a=Signal::derive(move||min.get() as f64)
            value_b=Signal::derive(move||max.get() as f64)
            set_value_a=Callback::new(move|x|set_min.set(x as u8))
            set_value_b=Callback::new(move|x|set_max.set(x as u8))
            min=0.0
            max={total_max as f64}
            step=1.0
            marks=SliderMarks::Automatic { create_names: true }
            value_display=move |v| format!("{v:.0}")
        />
    }
}

#[component]
pub fn NissSettingsComponent(niss_default: RwSignal<NissSwitchType>) -> impl IntoView {
    let niss_1 = Signal::derive(move || niss_default.get() != NissSwitchType::Never);
    let niss_2 = Signal::derive(move || niss_default.get() == NissSwitchType::Always);
    view! {
        <div style="display: flex; align-items: center; margin-bottom: 5px;">
            <label style="margin-right: 10px;">"Allow switching before step:"</label>
            <Toggle
                state=niss_1
                set_state=Callback::new(move |s| if s { niss_default.set(NissSwitchType::Before) } else { niss_default.set(NissSwitchType::Never)})
            />
        </div>
        <div style="display: flex; align-items: center;" class:grayed-out=move || !niss_1.get()>
            <label style="margin-right: 10px;">"Allow switching during step:"</label>
            <Toggle
                state=niss_2
                set_state=Callback::new(move |s| if s { niss_default.set(NissSwitchType::Always) } else { niss_default.set(NissSwitchType::Before)})
            />
        </div>
    }
}

#[component]
pub fn DefaultStepParameters(total_max: u8,
                             min: RwSignal<u8>,
                             max: RwSignal<u8>,
                             set_min: RwSignal<u8>,
                             set_max: RwSignal<u8>,
                             niss_default: RwSignal<NissSwitchType>,
                             variants: RwSignal<Vec<Axis>>
) -> impl IntoView {
    view! {
        <h4>"Step length"</h4>
        <StepLengthComponent min=min max=max set_min=set_min set_max=set_max total_max=total_max/>
        <h4>"Variations"</h4>
        <Multiselect
            options=vec![SelectableAxis::UD, SelectableAxis::FB, SelectableAxis::LR]
            search_text_provider=move |o| format!("{o}")
            render_option=move |o| format!("{o:?}").into_view()
            selected=Signal::derive(move || variants.get().iter().cloned().map(|v|Into::<SelectableAxis>::into(v)).collect())
            set_selected=Callback::new(move |v: Vec<SelectableAxis>| variants.set(v.iter().cloned().map(|v|Into::<Axis>::into(v)).collect()))
        />
        <h4>"NISS"</h4>
        <NissSettingsComponent niss_default=niss_default/>
    }
}