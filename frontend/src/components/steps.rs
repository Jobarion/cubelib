use std::fmt::format;
use cubelib::cube::Axis;
use yew::{Callback, Component, function_component, Html, html, Properties, use_state};

use crate::components::min_max::MinMaxComponent;
use crate::components::niss::{NissComponent, NissData, NissSwitchType};
use crate::components::variants::{AxisVariantComponent, AxisVariantData};

#[derive(PartialEq, Properties, Clone)]
pub struct EOStepProps {
    pub step_config: EOStepConfig,
    pub on_change: Callback<EOStepConfig>
}

#[derive(PartialEq, Properties, Clone, Debug)]
pub struct EOStepConfig {
    pub min: u32,
    pub max: u32,
    pub axis_ud: bool,
    pub axis_fb: bool,
    pub axis_lr: bool,
    pub niss_type: NissSwitchType,
    pub id: usize,
}

impl EOStepConfig {
    pub fn with_min_max(&self, min: u32, max: u32) -> Self {
        Self {
            min,
            max,
            id: self.id + 1,
            ..self.clone()
        }
    }

    pub fn with_variants(&self, variants: AxisVariantData) -> Self {
        Self {
            axis_ud: variants.ud,
            axis_fb: variants.fb,
            axis_lr: variants.lr,
            id: self.id + 1,
            ..self.clone()
        }
    }

    pub fn with_niss(&self, niss_type: NissSwitchType) -> Self {
        Self {
            niss_type,
            id: self.id + 1,
            ..self.clone()
        }
    }
}

#[function_component]
pub fn EOStepComponent(props: &EOStepProps) -> Html {
    let state_handle = use_state(|| props.step_config.clone());

    let min_max_callback = {
        let parent_callback = props.on_change.clone();
        let state_handle = state_handle.clone();
        move |(min, max)| {
            let new_state = (*state_handle).with_min_max(min, max);
            web_sys::console::log_1(&format!("State UD {}", (*state_handle).id).into());
            state_handle.set(new_state.clone());
            parent_callback.clone().emit(new_state);
        }
    };

    let variants_callback = {
        let parent_callback = props.on_change.clone();
        let state_handle = state_handle.clone();
        move |variants: AxisVariantData| {
            let new_state = (*state_handle).with_variants(variants);
            web_sys::console::log_1(&format!("State UD {}", (*state_handle).id).into());
            state_handle.set(new_state.clone());
            parent_callback.clone().emit(new_state);
        }
    };

    let niss_callback = {
        let parent_callback = props.on_change.clone();
        let state_handle = state_handle.clone();
        move |niss: NissData| {
            let new_state = (*state_handle).with_niss(niss.niss_type);
            web_sys::console::log_1(&format!("State UD {}", (*state_handle).id).into());
            state_handle.set(new_state.clone());
            parent_callback.clone().emit(new_state);
        }
    };

    let state = (*state_handle).clone();

    html! {
        <div>
            <h2>{"Edge Orientation"}</h2>
            <MinMaxComponent min=0 max=8 min_selected={state.min} max_selected={state.max} on_set={min_max_callback}/>
            <AxisVariantComponent data={AxisVariantData{ud: state.axis_ud, fb: state.axis_fb, lr:state.axis_lr}} on_changed={variants_callback}/>
            <NissComponent data={NissData { niss_type: state.niss_type }} on_changed={niss_callback}/>
        </div>
    }
}
