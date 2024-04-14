use leptos::{
    create_rw_signal, expect_context, html::ElementDescriptor, provide_context, with, NodeRef, RwSignal, Signal
};
use leptos_use::{use_element_size, UseElementSizeReturn};

use crate::{
    data::{expect_timeline_context, TimelineContext},
    types::{Identifier, PointInTime},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct ViewState {
    pub resolution: RwSignal<Resolution>,
    pub day_height: Signal<f64>,
    pub timeline_height: Signal<f64>,
    pub cursor: ViewCursor,
}

#[derive(Default)]
pub struct Resolution {
    days: i64, // How many to show in screen at the same time
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum ViewCursor {
    SpecificPoI(Identifier),
    CategoryPiT {
        cat: Identifier,
        pot: PointInTime,
    },
    #[default]
    Neutral,
}

pub fn provide_view_state<T: ElementDescriptor + 'static + Clone>(target: NodeRef<T>) -> ViewState {
    let TimelineContext { span, .. } = expect_timeline_context();
    // Height of timeline container
    let UseElementSizeReturn { height, .. } = use_element_size(target);
    let resolution = create_rw_signal(Resolution { days: 365 });

    // Höjden av en sida delat på Så här många dagar ska få plats
    // Ger höjden av en dag
    let day_height =
        Signal::derive(move || with!(|height, resolution| height / resolution.days as f64));

    // Så här många pixlar vill hela tidslinjen ha
    let timeline_height =
        Signal::derive(move || with!(|day_height, span| span.num_days() as f64 * day_height));

    let vs = ViewState {
        resolution,
        day_height,
        timeline_height,
        ..Default::default()
    };
    provide_context(vs);
    vs.clone()
}

pub fn expect_view_state() -> ViewState {
    expect_context::<ViewState>()
}

