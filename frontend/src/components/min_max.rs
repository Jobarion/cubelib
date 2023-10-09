use rand::distributions::Alphanumeric;
use rand::Rng;
use wasm_bindgen::prelude::{Closure, wasm_bindgen};
use yew::{Callback, Component, Context, Html, html, Properties};

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct MinMaxProps {
    pub min: u32,
    pub max: u32,
    pub min_selected: u32,
    pub max_selected: u32,
    pub on_set: Callback<(u32, u32)>
}

pub struct MinMaxComponent {
    on_set: Closure<dyn Fn(u32, u32)>,
    id: String,
}

impl Component for MinMaxComponent {
    type Message = ();
    type Properties = MinMaxProps;

    fn create(ctx: &Context<Self>) -> Self {
        let on_set_callback = ctx.props().on_set.clone();

        let id = rand::thread_rng().sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        Self {
            id,
            on_set: Closure::new(move |min: u32, max: u32| {
                on_set_callback.emit((min, max));
            })
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // web_sys::console::log_1(&format!("Viewing with {:?}", ctx.props()).into());
        html! {
            <div>
                <div id={self.id.clone()}></div>
                <span>{format!("Min {} Max {}", ctx.props().min_selected, ctx.props().max_selected)}</span>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        // web_sys::console::log_1(&format!("Rendering with {:?}", ctx.props()).into());
        if first_render {
            create_slider(self.id.as_str(), ctx.props().min, ctx.props().max, ctx.props().min_selected, ctx.props().max_selected, &self.on_set);
        } else {
            set_slider(self.id.as_str(), ctx.props().min_selected, ctx.props().max_selected);
        }
    }
}

#[wasm_bindgen(module = "/src/js/slider.js")]
extern "C" {
    fn create_slider(id: &str, min: u32, max: u32, min_selected: u32, max_selected: u32, on_set: &Closure<dyn Fn(u32, u32)>);
    fn set_slider(id: &str, min: u32, max: u32);
}
