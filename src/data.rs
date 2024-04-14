use std::collections::HashMap;

use chrono::{Local, NaiveDate, TimeDelta};
use indexmap::IndexMap;
use indicium::simple::{SearchIndex, SearchIndexBuilder};
use leptos::{
    create_rw_signal, expect_context, provide_context, with_current_owner, Callback, MaybeSignal,
    RwSignal, Signal, SignalGet, SignalUpdate, SignalWith, SignalWithUntracked,
};

use crate::types::{
    HasBeginning, Identifier, Identify, LevelUp, MainCategory, MyName, NonSignalPointOfInterest,
    Person, PointInTime, PointOfInterest, Timeline,
};

#[derive(Clone)]
pub struct TimelineContext {
    pub pois: RwSignal<PoIs>,
    pub cats: RwSignal<IndexMap<Identifier, ByMainCategory>>,
    pub add_poi: Callback<NonSignalPointOfInterest>,
    pub span: Signal<TimeDelta>,
    pub begins: Signal<Option<NaiveDate>>,
}

#[derive(Debug, Clone)]
pub struct PoIs {
    pois: HashMap<Identifier, PointOfInterest>,
    search: SearchIndex<Identifier>,
}

impl PoIs {
    fn from_poi_collection(pois: HashMap<Identifier, PointOfInterest>) -> Self {
        let mut search = SearchIndexBuilder::default()
            .autocomplete_type(indicium::simple::AutocompleteType::Context)
            .exclude_keywords(None)
            .build();
        pois.iter().for_each(|(k, v)| search.insert(k, v));
        Self { pois, search }
    }

    fn insert(&mut self, poi: PointOfInterest) {
        self.search.insert(&poi.identity(), &poi);
        self.pois.insert(poi.identity(), poi);
    }
    pub fn keys(&self) -> std::iter::Copied<std::collections::hash_map::Keys<'_, Identifier, PointOfInterest>> {
        self.pois.keys().copied()
    }
    pub fn search(&self, term: &str) -> Vec<Identifier> {
        self.search.search(term).into_iter().copied().collect()
    }
    pub fn autocomplete(&self, term: &str) -> Vec<String> {
        self.search.autocomplete(term)
    }
    pub fn get(&self, id: &Identifier) -> Option<&PointOfInterest> {
        self.pois.get(id)
    }
}

impl HasBeginning for PoIs {
    fn begins(&self) -> NaiveDate {
        self.pois.values().map(|o| o.begins()).min().unwrap()
    }

    fn try_begins(&self) -> Option<NaiveDate> {
        self.pois.values().map(|o| o.begins()).min()
    }
}

#[derive(Debug, Clone)]
pub struct ByMainCategory {
    category: MainCategory,
    points_of_interest: RwSignal<Vec<Identifier>>,
}

impl MyName for ByMainCategory {
    fn name(&self) -> MaybeSignal<String> {
        self.category.name()
    }
}

impl ByMainCategory {
    pub fn pois(&self) -> impl IntoIterator<Item = Identifier> {
        self.points_of_interest.get().into_iter()
    }
}

// INGRESS - load and split into categories - maintain state of active category for inserts
// Could also use with_untracked
// EGRESS - flatten categories and store list of PoIs
// TODO ägare av lifeline och vad som räknas som år noll
// TODO flera projekt
// Kolla upp en persons tidslinje och vice versa
// Lägga en särskild färg på varje kategori
// TODO integrera indicium

pub fn provide_timeline_context() {
    let starting_categories = init_example_categories();
    let person = Person::new(
        "Långben".to_string(),
        PointInTime::Day(NaiveDate::from_ymd_opt(2007, 12, 30).unwrap()),
    );
    let timeline = Timeline::new(person.identity());
    let pois = init_example_pois(&starting_categories, &timeline);

    let cats = arrange_by_category(starting_categories, &pois.values().collect::<Vec<_>>());

    let pois = create_rw_signal(PoIs::from_poi_collection(pois));
    let add_poi = create_callback_for_adding_poi(pois, cats);
    let begins = Signal::derive(move || pois.with(|p| p.try_begins())); // Kan vara tom pga inga inlagda saker än
    let today = Local::now().date_naive();
    let span = Signal::derive(move || begins.get().map_or(TimeDelta::zero(), |e| today - e));

    let ctx = TimelineContext {
        pois,
        cats,
        add_poi,
        span,
        begins,
    };
    provide_context(ctx);
}

fn create_callback_for_adding_poi(
    pois: RwSignal<PoIs>,
    cats: RwSignal<IndexMap<Identifier, ByMainCategory>>,
) -> Callback<NonSignalPointOfInterest> {
    let add_poi = move |p: NonSignalPointOfInterest| {
        let p: PointOfInterest = p.into();
        pois.update(|ps| {
            ps.insert(p);
        });
        cats.with_untracked(|c| {
            let cat = c.get(&p.parent()).unwrap();
            cat.points_of_interest
                .update(|pois| pois.push(p.identity()));
        });
    };
    Callback::new(with_current_owner(add_poi))
}

pub fn expect_timeline_context() -> TimelineContext {
    expect_context::<TimelineContext>()
}

fn arrange_by_category(
    starting_categories: Vec<MainCategory>,
    pois: &Vec<&PointOfInterest>,
) -> RwSignal<IndexMap<Identifier, ByMainCategory>> {
    create_rw_signal::<IndexMap<Identifier, ByMainCategory>>(IndexMap::from_iter(
        starting_categories.into_iter().map(|category| {
            let id = category.identity();
            (
                id,
                ByMainCategory {
                    category,
                    points_of_interest: pois
                        .iter()
                        .filter(|u| u.parent() == category.identity())
                        .map(|p| p.identity())
                        .collect::<Vec<_>>()
                        .into(),
                },
            )
        }),
    ))
}

fn init_example_pois(
    starting_categories: &Vec<MainCategory>,
    timeline: &Timeline,
) -> HashMap<Identifier, PointOfInterest> {
    let pois = vec![
        vec![(
            "Födsel",
            PointInTime::from(NaiveDate::from_ymd_opt(2007, 12, 30).unwrap()),
        )],
        vec![
            ("Gick", PointInTime::from(2008)),
            ("Talade", PointInTime::from(2008)),
            ("Cyklade", PointInTime::from(2013)),
            ("Simmade", PointInTime::from(2014)),
        ],
        vec![
            ("Hammarstad", PointInTime::from(2007)),
            ("Räveby", PointInTime::from(2009)),
            ("Ingalunda", PointInTime::from(2014)),
        ],
        vec![
            ("Förskola", PointInTime::from(2009)),
            ("Lågstadiet på Gladskolan", PointInTime::from(2014)),
            ("Mellanstadiet på snejipan", PointInTime::from(2017)),
        ],
    ];
    // Categories -> HAshmap<Identifier, Signal<Vec<Identifier>>>
    // Insertions: set_untracked på Hashmap<uuid, poi>, with_untracked på Categories, set på signal<vec, uuid>
    // expect context för
    // problemet med ordningen på kategorierna TODO
    starting_categories
        .iter()
        .map(|v| v.identity())
        .zip(pois.into_iter())
        .map(|(p, ps)| {
            ps.into_iter()
                .map(|(poi, st)| {
                    PointOfInterest::new_bare_with_start(
                        p,
                        timeline.identity(),
                        poi.to_string(),
                        st,
                    )
                })
                .collect::<Vec<_>>()
        })
        .flatten()
        .map(|p| (p.identity(), p))
        .collect::<HashMap<Identifier, PointOfInterest>>()
}

fn init_example_categories() -> Vec<MainCategory> {
    let starting_categories = vec![
        MainCategory::new("Life events".to_string()),
        MainCategory::new("Developmental steps".to_string()),
        MainCategory::new("Places lived".to_string()),
        MainCategory::new("Schooling".to_string()),
    ];
    starting_categories
}

