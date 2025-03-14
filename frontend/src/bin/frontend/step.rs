use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use cubelib::defs::NissSwitchType;
use cubelib::steps::util::{DR_SUBSETS, expand_subset_name};
use cubelib::cube::*;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::IoIcon;

use crate::SettingsState;
use crate::util::{RwSignalTup, use_local_storage};

#[derive(Clone)]
pub struct EOConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min_abs: RwSignalTup<u8>,
    pub max_abs: RwSignalTup<u8>,
    pub niss: RwSignalTup<NissSwitchType>,
    pub variants: RwSignalTup<Vec<CubeAxis>>
}

#[derive(Clone)]
pub struct RZPConfig {
    pub min_abs: RwSignalTup<u8>,
    pub max_abs: RwSignalTup<u8>,
    pub min_rel: RwSignalTup<u8>,
    pub max_rel: RwSignalTup<u8>,
    pub niss: RwSignalTup<NissSwitchType>,
}

#[derive(Clone)]
pub struct DRConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min_abs: RwSignalTup<u8>,
    pub max_abs: RwSignalTup<u8>,
    pub min_rel: RwSignalTup<u8>,
    pub max_rel: RwSignalTup<u8>,
    pub niss: RwSignalTup<NissSwitchType>,
    pub variants: RwSignalTup<Vec<CubeAxis>>,
    pub triggers: RwSignalTup<Vec<String>>,
    pub subsets: RwSignalTup<Vec<String>>,
    pub enforce_triggers: RwSignalTup<bool>,
}

#[derive(Clone)]
pub struct HTRConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min_abs: RwSignalTup<u8>,
    pub max_abs: RwSignalTup<u8>,
    pub min_rel: RwSignalTup<u8>,
    pub max_rel: RwSignalTup<u8>,
    pub niss: RwSignalTup<NissSwitchType>,
    pub variants: RwSignalTup<Vec<CubeAxis>>,
}

#[derive(Clone)]
pub struct FRConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min_abs: RwSignalTup<u8>,
    pub max_abs: RwSignalTup<u8>,
    pub min_rel: RwSignalTup<u8>,
    pub max_rel: RwSignalTup<u8>,
    pub niss: RwSignalTup<NissSwitchType>,
    pub variants: RwSignalTup<Vec<CubeAxis>>
}

#[derive(Clone)]
pub struct FinishConfig {
    pub enabled: (Signal<bool>, Callback<bool>),
    pub min_abs: RwSignalTup<u8>,
    pub max_abs: RwSignalTup<u8>,
    pub min_rel: RwSignalTup<u8>,
    pub max_rel: RwSignalTup<u8>,
    pub leave_slice: RwSignalTup<bool>,
}

impl EOConfig {
    pub fn from_local_storage(enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min_abs: use_local_storage("eo-min-abs", 0),
            max_abs: use_local_storage("eo-max-abs", 5),
            niss: use_local_storage("eo-niss", NissSwitchType::Always),
            variants: use_local_storage("eo-variants", vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR]),
        }
    }

    pub fn reset(&self) {
        self.min_abs.1.set(0);
        self.max_abs.1.set(5);
        self.niss.1.set(NissSwitchType::Always);
        self.variants.1.set(vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR]);
        self.min_abs.2();
        self.max_abs.2();
        self.niss.2();
        self.variants.2();
    }
}

impl DRConfig {
    pub fn from_local_storage(enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min_rel: use_local_storage("dr-min-rel", 0),
            max_rel: use_local_storage("dr-max-rel", 12),
            min_abs: use_local_storage("dr-min-abs", 0),
            max_abs: use_local_storage("dr-max-abs", 14),
            niss: use_local_storage("dr-niss", NissSwitchType::Before),
            variants: use_local_storage("dr-variants", vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR]),
            triggers: use_local_storage("dr-triggers", vec!["R".to_string(), "R U2 R".to_string(), "R F2 R".to_string(), "R U R".to_string(), "R U' R".to_string()]),
            subsets: use_local_storage("htr-subsets", vec![]), // Legacy name
            enforce_triggers: use_local_storage("dr-use-triggers", true)
        }
    }

    pub fn reset(&self) {
        self.min_rel.1.set(0);
        self.max_rel.1.set(12);
        self.min_abs.1.set(0);
        self.max_abs.1.set(14);
        self.niss.1.set(NissSwitchType::Before);
        self.variants.1.set(vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR]);
        self.triggers.1.set(vec!["R".to_string(), "R U2 R".to_string(), "R F2 R".to_string(), "R U R".to_string(), "R U' R".to_string()]);
        self.min_abs.2();
        self.max_abs.2();
        self.min_rel.2();
        self.max_rel.2();
        self.niss.2();
        self.variants.2();
        self.triggers.2();
        self.subsets.2();
        self.enforce_triggers.2();
    }
}

impl RZPConfig {
    pub fn from_local_storage() -> Self {
        Self {
            min_rel: use_local_storage("rzp-min-rel", 0),
            max_rel: use_local_storage("rzp-max-rel", 3),
            min_abs: use_local_storage("rzp-min-abs", 0),
            max_abs: use_local_storage("rzp-max-abs", 6),
            niss: use_local_storage("rzp-niss", NissSwitchType::Never),
        }
    }

    pub fn reset(&self) {
        self.min_rel.1.set(0);
        self.max_rel.1.set(3);
        self.min_abs.1.set(0);
        self.max_abs.1.set(6);
        self.niss.1.set(NissSwitchType::Never);
        self.min_abs.2();
        self.max_abs.2();
        self.min_rel.2();
        self.max_rel.2();
        self.niss.2();
    }
}

impl HTRConfig {
    pub fn from_local_storage(enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min_rel: use_local_storage("htr-min-rel", 0),
            max_rel: use_local_storage("htr-max-rel", 12),
            min_abs: use_local_storage("htr-min-abs", 0),
            max_abs: use_local_storage("htr-max-abs", 20),
            niss: use_local_storage("htr-niss", NissSwitchType::Before),
            variants: use_local_storage("htr-variants", vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR]),
        }
    }

    pub fn reset(&self) {
        self.min_rel.1.set(0);
        self.max_rel.1.set(12);
        self.min_abs.1.set(0);
        self.max_abs.1.set(20);
        self.niss.1.set(NissSwitchType::Before);
        self.variants.1.set(vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR]);
        self.min_abs.2();
        self.max_abs.2();
        self.min_rel.2();
        self.max_rel.2();
        self.niss.2();
        self.variants.2();
    }
}

impl FRConfig {
    pub fn from_local_storage(enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min_rel: use_local_storage("fr-min-rel", 0),
            max_rel: use_local_storage("fr-max-rel", 10),
            min_abs: use_local_storage("fin-min-abs", 0),
            max_abs: use_local_storage("fin-max-abs", 26),
            niss: use_local_storage("fr-niss", NissSwitchType::Before),
            variants: use_local_storage("fr-variants", vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR]),
        }
    }

    pub fn reset(&self) {
        self.min_rel.1.set(0);
        self.max_rel.1.set(10);
        self.min_abs.1.set(0);
        self.max_abs.1.set(26);
        self.niss.1.set(NissSwitchType::Before);
        self.variants.1.set(vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR]);
        self.min_abs.2();
        self.max_abs.2();
        self.min_rel.2();
        self.max_rel.2();
        self.niss.2();
        self.variants.2();
    }
}

impl FinishConfig {
    pub fn from_local_storage(enabled: (Signal<bool>, Callback<bool>)) -> Self {
        Self {
            enabled,
            min_rel: use_local_storage("fin-min-rel", 0),
            max_rel: use_local_storage("fin-max-rel", 10),
            min_abs: use_local_storage("fin-min-abs", 0),
            max_abs: use_local_storage("fin-max-abs", 30),
            leave_slice: use_local_storage("fin-ls", false),
        }
    }

    pub fn reset(&self) {
        self.min_rel.1.set(0);
        self.max_rel.1.set(10);
        self.min_abs.1.set(0);
        self.max_abs.1.set(30);
        self.leave_slice.1.set(false);
        self.min_abs.2();
        self.max_abs.2();
        self.min_rel.2();
        self.max_rel.2();
        self.leave_slice.2();
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum SelectableAxis {
    UD,
    FB,
    LR
}

impl Into<CubeAxis> for SelectableAxis {
    fn into(self) -> CubeAxis {
        match self {
            Self::UD => CubeAxis::UD,
            Self::FB => CubeAxis::FB,
            Self::LR => CubeAxis::LR,
        }
    }
}

impl From<CubeAxis> for SelectableAxis {
    fn from(value: CubeAxis) -> Self {
        match value {
            CubeAxis::UD => Self::UD,
            CubeAxis::FB => Self::FB,
            CubeAxis::LR => Self::LR,
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
            min_abs=eo_config.min_abs
            max_abs=eo_config.max_abs
            total_max=8
            variants=eo_config.variants
        />
    }
}

#[component]
pub fn DRParameters() -> impl IntoView {
    let dr_config = use_context::<DRConfig>().expect("DR context required");
    let rzp_config = use_context::<RZPConfig>().expect("RZP context required");
    let settings = use_context::<SettingsState>().expect("Settings context required");

    let default_triggers = vec![
        "R".to_string(),
        "R U2 F2 R".to_string(),
        "R F2 U2 R".to_string(),
        "R U2 R".to_string(),
        "R F2 R".to_string(),
        "R U R".to_string(),
        "R U' R".to_string(),
        "R L".to_string(),
        "R U L".to_string(),
        "R U' L".to_string(),
    ];

    let additional_triggers = settings.additional_triggers();
    let trigger_options = Signal::derive(move|| {
        let mut triggers = default_triggers.clone();
        let mut additional = additional_triggers.get();
        triggers.append(&mut additional);
        triggers.sort();
        triggers.dedup();
        triggers
    });

    let triggers_disabled = Signal::derive(move|| !dr_config.enforce_triggers.0.get() || dr_config.triggers.0.get().is_empty());

    view! {
        <DefaultStepParameters
            niss_default=dr_config.niss
            min_abs=dr_config.min_abs
            max_abs=dr_config.max_abs
            min_rel=dr_config.min_rel
            max_rel=dr_config.max_rel
            total_max_rel=12
            total_max=16
            variants=dr_config.variants
        />
        <h4>"Triggers"</h4>
        <div style="display: flex; align-items: center;">
            <label style="margin-right: 10px;">"Use triggers:"</label>
            <Toggle state=dr_config.enforce_triggers.0 set_state=dr_config.enforce_triggers.1 />
        </div>
        <div class:grayed-out=move|| !dr_config.enforce_triggers.0.get()>
            <Multiselect
                options=trigger_options
                search_text_provider=move |o| format!("{o}")
                render_option=move |o| format!("{o}").into_view()
                selected=dr_config.triggers.0
                set_selected=move |v| dr_config.triggers.1.set(v)
            />
        </div>
        <DRSubsetSelection />
        <div class:grayed-out=triggers_disabled>
            <h2>"RZP"</h2>
            <h4>"Step length"</h4>
            {move || {
                if settings.is_relative() {
                    view! {
                        <StepLengthComponent
                            min=rzp_config.min_rel.clone()
                            max=rzp_config.max_rel.clone()
                            total_max=5
                        />
                    }
                } else {
                    view! {
                        <StepLengthComponent
                            max=rzp_config.max_abs.clone()
                            min=rzp_config.min_abs.clone()
                            total_max=9
                        />
                    }
                }
            }}
            <h4>"NISS"</h4>
            <NissSettingsComponent niss_default=rzp_config.niss/>
        </div>
    }
}

#[component]
pub fn DRSubsetSelection() -> impl IntoView {
    let settings = use_context::<SettingsState>().expect("Settings context required");
    let dr_config = use_context::<DRConfig>().expect("DR context required");
    let (cur_subset, cur_subset_set) = create_signal("".to_string());
    let (subsets, subsets_set, _) = dr_config.subsets;

    let expanded_subset: Signal<Vec<String>> = Signal::derive(move|| expand_subset_name(cur_subset.get().as_str())
        .into_iter()
        .map(|s|s.to_string())
        .collect()
    );

    let advanced = settings.advanced();

    let placeholder = Signal::derive(move||if advanced.get() { "2c3 e6" } else { "2c3" }.to_string());

    view! {
        <h4>"Subsets"</h4>
        <TextInput
            get=cur_subset
            set=cur_subset_set placeholder=placeholder
            style="width: 150px"
            class=move|| if cur_subset.get().is_empty() || !expanded_subset.get().is_empty() { "" } else { "leptonic-input-invalid" }
        />
        <SubsetPreview
            subsets=Signal::derive(move||{
                let subsets = subsets.get();
                expanded_subset.get()
                    .into_iter()
                    .filter(|x|!subsets.contains(x))
                    .collect()
            })
            on_click=move|s: String|{
                if s.len() == 3 {
                    for subset in DR_SUBSETS {
                        let subset = subset.to_string();
                        if !subset.starts_with(&s) {
                            continue;
                        }
                        let mut subsets = subsets.get();
                        if !subsets.contains(&subset) {
                            subsets.push(subset);
                            subsets_set.set(subsets);
                        }
                    }
                } else {
                    let mut subsets = subsets.get();
                    if !subsets.contains(&s) {
                        subsets.push(s);
                        subsets_set.set(subsets);
                    }
                }
            }
            advanced=settings.advanced()
        />
        <button
            enabled=move||false
            on:click=move|_|{
                if !expanded_subset.get().is_empty()  {
                    let mut subsets = subsets.get();
                    for subset in expanded_subset.get() {
                        if !subsets.contains(&subset) {
                            subsets.push(subset);
                        }
                    }
                    subsets_set.set(subsets);
                    cur_subset_set.set("".to_string());
                }
            }
            class="icon-button"
            style:cursor=move||if !expanded_subset.get().is_empty() { "pointer" } else { "default" }
            style:opacity="60%"
            style:font-size="30px">
            <Icon icon=IoIcon::IoAddOutline/>
        </button>
        <h5>{move||if subsets.get().is_empty() { "All subsets enabled" } else { "Enabled subsets"}}</h5>
        <SubsetList subsets=subsets subsets_set=subsets_set advanced=settings.advanced() />
    }
}

#[component]
pub fn HTRParameters() -> impl IntoView {
    let htr_config = use_context::<HTRConfig>().expect("HTR context required");

    view! {

        <DefaultStepParameters
            niss_default=htr_config.niss
            min_abs=htr_config.min_abs
            max_abs=htr_config.max_abs
            min_rel=htr_config.min_rel
            max_rel=htr_config.max_rel
            total_max_rel=14
            total_max=28
            variants=htr_config.variants
        />
    }
}

#[component]
fn SubsetList(
    #[prop(into)] subsets: Signal<Vec<String>>,
    #[prop(into)] subsets_set: Out<Vec<String>>,
    advanced: Signal<bool>
) -> impl IntoView {
    view! {
        <div style:width="500px">
        {move || {
            let advanced = advanced.get();
            let all_subsets: HashSet<String> = subsets.get()
                .iter()
                .cloned()
                .map(|subset| if advanced {
                    subset
                } else {
                    subset.split_once(" ").unwrap().0.to_string()
                })
                .collect();
            all_subsets.into_iter()
                .map(|subset| {
                    let subset_c = subset.clone();
                    view! {
                        <Chip color=ChipColor::Secondary dismissible=move |_| {
                            subsets_set.set(subsets.get()
                                .into_iter()
                                .filter(|x|!x.starts_with(&subset_c))
                                .collect());
                        }>
                            {subset}
                        </Chip>
                    }.into_view()
                })
                .collect_view()
        }}
        </div>
    }
}

#[component]
fn SubsetPreview(
    #[prop(into)] subsets: Signal<Vec<String>>,
    #[prop(into)] on_click: Callback<String>,
    advanced: Signal<bool>
) -> impl IntoView {
    view! {
        <div style:width="500px">
        {move || {
            let advanced = advanced.get();
            let all_subsets: HashSet<String> = subsets.get()
                .iter()
                .cloned()
                .map(|subset| if advanced {
                    subset
                } else {
                    subset.split_once(" ").unwrap().0.to_string()
                })
                .collect();
            all_subsets
                .into_iter()
                .map(|subset| {
                    let subset_c = subset.clone();
                    view! {
                        <Chip color=ChipColor::Primary on:click=move |_| {
                            on_click.call(subset_c.clone())
                        }>
                            {subset}
                        </Chip>
                    }.into_view()
                })
                .collect_view()
        }}
        </div>
    }
}

#[component]
pub fn FRParameters() -> impl IntoView {
    let fr_config = use_context::<FRConfig>().expect("FR context required");
    view! {

        <DefaultStepParameters
            niss_default=fr_config.niss
            min_abs=fr_config.min_abs
            max_abs=fr_config.max_abs
            min_rel=fr_config.min_rel
            max_rel=fr_config.max_rel
            total_max_rel=10
            total_max=30
            variants=fr_config.variants
        />
    }
}

#[component]
pub fn FinishParameters() -> impl IntoView {
    let fin_config = use_context::<FinishConfig>().expect("Finish context required");
    view! {
        <DefaultStepParameters
            min_abs=fin_config.min_abs
            max_abs=fin_config.max_abs
            min_rel=fin_config.min_rel
            max_rel=fin_config.max_rel
            total_max_rel=10
            total_max=30
        />
        <div style="display: flex; align-items: center;">
            <label style="margin-right: 10px;">"Leave slice:"</label>
            <Toggle
                state=Signal::derive(move || fin_config.leave_slice.0.get())
                set_state=Callback::new(move |s| fin_config.leave_slice.1.set(s))
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
    min: RwSignalTup<u8>,
    max: RwSignalTup<u8>,
) -> impl IntoView {

    view! {
        <RangeSlider
            class={move||if total_max < 20 || total_max >= 30 { "" } else { "slider-reduce-mark-clutter" }}
            value_a=Signal::derive(move||min.0.get() as f64)
            value_b=Signal::derive(move||max.0.get() as f64)
            set_value_a=Callback::new(move|x|min.1.set(x as u8))
            set_value_b=Callback::new(move|x|max.1.set(x as u8))
            min=0.0
            max={total_max as f64}
            step=1.0
            marks=SliderMarks::Automatic { create_names: true }
            value_display=move |v| format!("{v:.0}")
        />
    }
}

#[component]
pub fn NissSettingsComponent(niss_default: RwSignalTup<NissSwitchType>) -> impl IntoView {
    let niss_1 = Signal::derive(move || niss_default.0.get() != NissSwitchType::Never);
    let niss_2 = Signal::derive(move || niss_default.0.get() == NissSwitchType::Always);
    view! {
        
        <div style="display: flex; align-items: center; margin-bottom: 5px;">
            <label style="margin-right: 10px;">"Allow switching before step:"</label>
            <Toggle
                state=niss_1
                set_state=Callback::new(move |s| if s { niss_default.1.set(NissSwitchType::Before) } else { niss_default.1.set(NissSwitchType::Never)})
            />
        </div>
        <div style="display: flex; align-items: center;" class:grayed-out=move || !niss_1.get()>
            <label style="margin-right: 10px;">"Allow switching during step:"</label>
            <Toggle
                state=niss_2
                set_state=Callback::new(move |s| if s { niss_default.1.set(NissSwitchType::Always) } else { niss_default.1.set(NissSwitchType::Before)})
            />
        </div>
    }
}

#[component]
pub fn DefaultStepParameters(total_max: u8,
                             min_abs: RwSignalTup<u8>,
                             max_abs: RwSignalTup<u8>,
                             #[prop(into, optional)] total_max_rel: Option<u8>,
                             #[prop(into, optional)] min_rel: Option<RwSignalTup<u8>>,
                             #[prop(into, optional)] max_rel: Option<RwSignalTup<u8>>,
                             #[prop(into, optional)] niss_default: Option<RwSignalTup<NissSwitchType>>,
                             #[prop(into, optional)] variants: Option<RwSignalTup<Vec<CubeAxis>>>
) -> impl IntoView {
    let settings = use_context::<SettingsState>().expect("Settings context required");

    let relative = settings.relative();

    let niss = niss_default.map(|niss| view! {
        <h4>"NISS"</h4>
        <NissSettingsComponent niss_default=niss/>
    }.into_view()).unwrap_or(view!{}.into_view());

    view! {
        <h4>"Step length"</h4>
        {move || if !relative.get() || (min_rel.is_none() || max_rel.is_none()) {
            view! {
                <StepLengthComponent min=min_abs.clone() max=max_abs.clone() total_max=total_max/>
            }
        } else {
            view! {
                <StepLengthComponent min=min_rel.clone().unwrap() max=max_rel.clone().unwrap() total_max=total_max_rel.unwrap()/>
            }
        }}
        {move || variants.clone().map(|var| view! {
                <h4>"Variations"</h4>
                <Multiselect
                    options=vec![SelectableAxis::UD, SelectableAxis::FB, SelectableAxis::LR]
                    search_text_provider=move |o| format!("{o}")
                    render_option=move |o| format!("{o:?}").into_view()
                    selected=Signal::derive(move || var.0.get().iter().cloned().map(|v|Into::<SelectableAxis>::into(v)).collect())
                    set_selected=Callback::new(move |v: Vec<SelectableAxis>| var.1.set(v.iter().cloned().map(|v|Into::<CubeAxis>::into(v)).collect()))
                />
            }.into_view()).filter(|_|settings.is_advanced()).unwrap_or(view!{}.into_view())
        }
        {niss}
    }
}