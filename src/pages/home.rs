use std::{
    collections::{HashSet, VecDeque},
    marker::PhantomData,
};

use enum_dispatch::enum_dispatch;
use leptos::{html::Div, logging::log, *};
use leptos_hotkeys::{use_hotkeys, use_hotkeys_context, HotkeysContext};
use rand::seq::IteratorRandom;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, ScrollToOptions};
// use web_sys::ScrollIntoViewOptions;

use crate::{
    data::{expect_timeline_context, TimelineContext},
    types::{HasBeginning, Identifier, Identify, MyName, NonSignalPointOfInterest},
    viewstate::{expect_view_state, provide_view_state, ViewState},
};

type Queue = RwSignal<VecDeque<Choice>>;
/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let input_queue: Queue = create_rw_signal(VecDeque::new());
    let (r_ground, _w_ground) = create_signal(String::from("Ground"));
    let TimelineContext { add_poi, cats, .. } = expect_timeline_context();
    let a_cat = cats.get_untracked().keys().next().unwrap().to_owned();

    let call = Callback::new(move |s| log!("My mood is: {:?}", s));
    use_hotkeys!(("ctrl+m") => move |_| {
        input_queue
        .update(|v| {
            v.push_front(
                Choice::from(UserMayChoose::<Mood>::new(call)),
            )
        })
    });
    use_hotkeys!(("ctrl+u") => move |_| {
        add_poi(NonSignalPointOfInterest::new(a_cat.to_owned(), "Korfbajs".to_string()))
    });
    provide_context(input_queue);
    view! {
        <div class="min-h-[100svh] bg-sky-50 grid">
            <Timeline/>
            <SearchPoi/>
            <Commands/>
            <Modals/>
        </div>
    }
}

#[component]
pub fn SearchPoi() -> impl IntoView {
    let TimelineContext { pois, .. } = expect_timeline_context();
    let (term, term_w) = create_signal("".to_string());
    let autocomplete = Signal::derive(move || with!(|pois, term| pois.autocomplete(term)));
    let search = Signal::derive(move || with!(|pois, term| pois.search(term)));
    let id = Uuid::new_v4();
    let handle_input = move |t| term_w(event_target_value(&t));
    let search_results = Signal::derive(move || {
        search.with(|s| {
            s.into_iter()
                .map(|f| view! { <li>{f.to_string()}</li> })
                .collect_view()
        })
    });

    let auto_results = Signal::derive(move || {
        autocomplete.with(|s| {
            s.into_iter()
                .map(|f| view! { <li>{f.to_string()}</li> })
                .collect_view()
        })
    });
    view! {
        <Portal>
            <Dialog id>
                <div>
                    <input on:input=handle_input/>
                </div>
                <div>
                    <ol>{search_results}</ol>
                </div>
                <div>
                    <ol>{auto_results}</ol>
                </div>
            </Dialog>
        </Portal>
    }
}

#[component]
pub fn Commands() -> impl IntoView {
    view! {
        <div class="absolute bottom-4 w-full">
            <ul class="flex gap-4">
                <li>Gå</li>
                <li>Se</li>
                <li>Ny</li>
            </ul>
        </div>
    }
}

#[component]
pub fn Timeline() -> impl IntoView {
    let TimelineContext {
        cats, span, pois, ..
    } = expect_timeline_context();
    let timeline_ref = create_node_ref::<Div>();
    let ViewState {
        resolution,
        day_height,
        timeline_height,
        cursor,
    } = provide_view_state(timeline_ref);
    let (current, current_w) = create_signal::<Identifier>(Identifier::default());
    let day_css_var = Signal::derive(move || day_height.with(|h| format!("{h}px")));
    let update_current = move || {
        use IteratorRandom;
        current_w(
            pois.with(|p| {
                let mut rng = rand::thread_rng();
                p.keys().choose(&mut rng)
            })
            .clone()
            .unwrap(),
        );
    };
    use_hotkeys!(("ctrl+y") => move |_| {
        let maybe_el = document().get_element_by_id(&format!("poi-{}", current().to_string()));
        let maybe_con = document().get_element_by_id("timeline-container");
        if let Some((el, con)) = maybe_el.zip(maybe_con) {

            log!("Scroll! {:?}", el.text_content());
            center_element_in_container(el, con);
        }
        update_current();
    });
    use_hotkeys!(("ctrl+i") => move |_| {
        let tl = timeline_ref().unwrap();
        log!("y: {} x: {}",tl.scroll_top(), tl.scroll_left());
    });
    create_effect(move |_| update_current());
    // TODO gör så att tidslinjen inte överlappar med kategorietiketten
    view! {
        <div
            ref=timeline_ref
            id="timeline-container"
            class="place-self-center border-sky-100 border-2 w-9/12 aspect-video overflow-scroll relative flex gap-24"
            style=("--day", day_css_var)
        >
            <div
                id="flex"
                class="flex gap-24"
                style:height=move || format!("{}px", timeline_height.get())
            >
                <For each=move || cats.get().into_iter() key=move |(u, _)| u.clone() let:ca>
                    <div id="röd" class="border-2 border-red-200 w-[300px] h-full relative">
                        <h2 class="text-blue-600 sticky top-1">{ca.1.name()}</h2>
                        <For each=move || ca.1.pois() key=move |u| u.clone() let:poi_id>
                            <Point id=poi_id/>
                        </For>
                    </div>
                </For>
            </div>
        </div>
    }
}

fn center_element_in_container(el: Element, con: Element) {
    let el = el.dyn_into::<HtmlElement>().unwrap();
    let con = con.dyn_into::<HtmlElement>().unwrap();

    let poi = el.inner_text();
    let offset_top = el.offset_top();
    let con_height = con.offset_height();
    let scroll_y = offset_top - (con_height / 2);
    // log!("Working with: {poi}. OffTop: {}. Con height: {}. YGoal: {}", offset_top, con_height, scroll_y);

    let el_offset_width = el.offset_width();
    let offset_left = el
        .offset_parent()
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap()
        .offset_left();
    let con_width = con.offset_width();
    let offset_left = offset_left + (el_offset_width / 2) - (con_width / 2);
    let scroll_h = offset_left;
    let scroll_h = scroll_h.max(0);

    // log!(
    //     "Working with: {poi}. OffLeft: {}. Con width: {}. HGoal: {}",
    //     offset_left,
    //     con_width,
    //     scroll_h
    // );

    let mut scroll_opts = ScrollToOptions::new();
    scroll_opts.top(scroll_y as f64);
    scroll_opts.left(scroll_h as f64);
    scroll_opts.behavior(web_sys::ScrollBehavior::Smooth);
    con.scroll_to_with_scroll_to_options(&scroll_opts);
}

#[component]
pub fn Point(id: Identifier) -> impl IntoView {
    let TimelineContext { pois, begins, .. } = expect_timeline_context();
    let ViewState { day_height, .. } = expect_view_state();
    let poi = pois.with_untracked(|p| p.get(&id).copied().unwrap());
    let origin_distance = Signal::derive(move || begins.with(|b| poi.begins() - b.unwrap()));
    let pixel_top = Signal::derive(move || {
        let px =
            with!(|origin_distance, day_height| origin_distance.num_days() as f64 * day_height);
        format!("{px}px")
    });
    let id = format!("poi-{}", poi.identity().to_string());

    view! {
        <div id=id class="border-2 relative border-blue-800" style:top=pixel_top>
            {poi.name()}
        </div>
    }
}

#[enum_dispatch(Choice)]
trait PresentsChoices {
    fn proffer(&self) -> View;
}

#[enum_dispatch(Choice)]
trait Identifies {
    fn identify(&self) -> Uuid;
    fn child_of(&self) -> Option<Uuid>;
    fn current(&self) -> View;
}

type ChooseMood = UserMayChoose<Mood>;
type ChooseString = UserMayChoose<String>;

#[enum_dispatch]
#[derive(Clone)]
pub(crate) enum Choice {
    ChooseMood,
    ChooseString,
}

#[derive(Clone, Debug)]
pub(crate) struct UserMayChoose<T>
where
    T: 'static + IntoView + Clone,
{
    id: Uuid,
    parent: Option<Uuid>,
    starting: Option<T>,
    callback: Callback<T>,
}

#[derive(Clone, Debug)]
pub(crate) struct BecomeChoice<T> {
    id: Option<Uuid>,
    of: PhantomData<T>,
}

impl BecomeChoice<String> {
    fn with_starting(&self, starting: String, callme: Callback<String>) -> Choice {
        let mut choice = UserMayChoose::<String>::new(callme);
        choice.starting = Some(starting);
        Choice::from(choice)
    }
}

impl<T: IntoView + Clone> UserMayChoose<T> {
    fn new(callback: impl Into<Callback<T>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            starting: None,
            callback: callback.into(),
            parent: None,
        }
    }
}

impl<T: IntoView + Clone> Identifies for UserMayChoose<T> {
    fn identify(&self) -> Uuid {
        self.id
    }

    fn child_of(&self) -> Option<Uuid> {
        self.parent
    }

    fn current(&self) -> View {
        self.starting.clone().into_view()
    }
}

impl PresentsChoices for UserMayChoose<String> {
    fn proffer(&self) -> View {
        view! { <StingChoice id=self.identify() consequence=self.callback/> }.into_view()
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Mood {
    mood: String,
    comment: String,
}

impl PresentsChoices for UserMayChoose<Mood> {
    fn proffer(&self) -> View {
        view! { <MoodChoice id=self.id consequence=self.callback/> }.into_view()
    }
}

impl IntoView for Mood {
    fn into_view(self) -> View {
        view! { self.mood }.into_view()
    }
}

#[component]
pub fn MoodChoice(id: Uuid, consequence: Callback<Mood>) -> impl IntoView {
    let mood = create_rw_signal(String::from("Haha"));
    let comment = create_rw_signal(String::from("Haha"));

    // Snapshot and reset of hotkey scope
    let HotkeysContext { active_scopes, .. } = use_hotkeys_context();
    let current_scoped = store_value(active_scopes.get_untracked());
    let reset_hotkey_scopes = move || {
        log!("Cleaning up mood choice hotkey scope");
        active_scopes.set(current_scoped.get_value());
    };
    create_effect(move |_| {
        active_scopes.set_untracked(HashSet::<String>::from(["*".to_string(), id.to_string()]));
        log!("{:?}", active_scopes.get_untracked());
    });

    // Finalization
    use_hotkeys!(("ctrl+enter", id.to_string()) => move |_| {
        reset_hotkey_scopes();
        consequence(Mood { mood: mood.get_untracked(), comment: comment.get_untracked()});
    });

    view! {
        <Dialog id=id>
            <dl>
                <Definition<String> id=id title="Mood" hotkey="j" term=mood  />
                <Definition<String> id=id title="Comment" hotkey="f" term=comment  />
            </dl>
        </Dialog>
    }
}

#[component]
pub fn Definition<T>(
    id: Uuid,
    title: &'static str,
    hotkey: &'static str,
    term: RwSignal<T>,
) -> impl IntoView
where
    T: 'static + Clone + IntoView,
    Choice: From<UserMayChoose<T>>,
{
    let (choice, choice_w) = create_signal::<Option<Choice>>(None);
    let call = Callback::new(move |s| {
        term.set(s);
        choice_w(None);
    });
    use_hotkeys!((hotkey, id.to_string()) => move |_| {
        let q = UserMayChoose::<T>::new(call);
        choice_w(Some(Choice::from(q)));
    });

    view! {
        <Portal>{move || choice().map(|c| c.proffer().into_view())}</Portal>
        <div>
            <dt class="indicator p-4">
                <span class="indicator-item indicator-start kbd kbd-sm">{hotkey}</span>
                <p class="font-bold text-lg">{title}</p>
            </dt>
            <dd>{term}</dd>
        </div>
    }
}

#[component]
pub fn StingChoice(id: Uuid, consequence: Callback<String>) -> impl IntoView {
    let (r_choice, w_choice) = create_signal(String::new());
    let HotkeysContext { active_scopes, .. } = use_hotkeys_context();
    let current_scoped = store_value(active_scopes.get_untracked());
    let reset_hotkey_scopes = move || {
        log!("Cleaning up mood choice hotkey scope");
        active_scopes.set(current_scoped.get_value());
    };
    create_effect(move |_| {
        active_scopes.set_untracked(HashSet::<String>::from(["*".to_string(), id.to_string()]));
        log!("{:?}", active_scopes.get_untracked());
    });
    on_cleanup(move || reset_hotkey_scopes());
    use_hotkeys!(("ctrl+enter", id.to_string()) => move |_| {
        consequence(r_choice());
    });
    view! {
        <Dialog id=id>
            <input
                autofocus
                prop:value=r_choice
                on:input=move |e| w_choice(event_target_value(&e))
            />
        </Dialog>
    }
}
// Saker som behöver kontrolleras
// Om dialogen syns
// Om man är klar med dialogen
// Om animering in-ut ska visas
// Cleanup i modal-kön
// En view med genvägar för att ändra varje field
// Generera kod för det på något sätt
#[component]
pub fn Modals() -> impl IntoView {
    let (r_show_modal, w_show_modal) = create_signal(true);
    let queue = expect_context::<Queue>();
    view! {
        <Show when=r_show_modal fallback=|| ()>
            <Portal>
                <For each=move || queue() key=move |q| q.identify() let:choice>
                    {choice.proffer()}
                </For>
            </Portal>
        </Show>
    }
}

#[component]
pub fn Dialog(id: Uuid, children: Children) -> impl IntoView {
    view! {
        <dialog id=id.to_string() class="modal" open=true>
            <div class="modal-box shadow-md">{children()}</div>
        </dialog>
    }
}

