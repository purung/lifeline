#![allow(dead_code)]

use chrono::{NaiveDate, NaiveDateTime};
use leptos::{MaybeSignal, RwSignal, SignalGet};
use uuid::Uuid;

pub struct Scale;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Identifier(Uuid);

impl Default for Identifier {
    fn default() -> Self {
        Identifier(Uuid::new_v4())
    }
}

impl ToString for Identifier {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct PointOfInterest {
    id: Identifier,
    parent: Identifier,
    timeline: Identifier,
    starts: RwSignal<PointInTime>,
    ends: RwSignal<Ending>,
    name: RwSignal<String>,
    context: RwSignal<String>,
}

#[derive(Default, Debug, Clone)]
pub struct NonSignalPointOfInterest {
    id: Identifier,
    parent: Identifier,
    timeline: Identifier,
    starts: PointInTime,
    ends: Ending,
    name: String,
    context: String,
}

impl NonSignalPointOfInterest {
    pub fn new(parent: Identifier, name: String) -> Self {
        Self {
            parent,
            name,
            ..Default::default()
        }
    }
}

impl From<NonSignalPointOfInterest> for PointOfInterest {
    fn from(value: NonSignalPointOfInterest) -> Self {
        Self {
            id: value.id,
            parent: value.parent,
            timeline: value.timeline,
            starts: value.starts.into(),
            ends: value.ends.into(),
            name: value.name.into(),
            context: value.context.into(),
        }
    }
}

pub trait LevelUp {
    fn parent(&self) -> Identifier;
}

#[derive(Default, Debug, Hash)]
pub struct Timeline {
    id: Identifier,
    person: Identifier,
}

impl Timeline {
    pub fn new(person: Identifier) -> Self {
        Self {
            person,
            ..Default::default()
        }
    }
}

impl Identify for Timeline {
    fn identity(&self) -> Identifier {
        self.id
    }
}

impl LevelUp for Timeline {
    fn parent(&self) -> Identifier {
        self.person
    }
}

impl LevelUp for PointOfInterest {
    fn parent(&self) -> Identifier {
        self.parent
    }
}

impl Identify for PointOfInterest {
    fn identity(&self) -> Identifier {
        self.id
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct MainCategory {
    id: Identifier,
    name: RwSignal<String>,
}

pub trait HasBeginning {
    fn begins(&self) -> NaiveDate;
}

impl HasBeginning for PointOfInterest {
    fn begins(&self) -> NaiveDate {
        self.starts.get().begins()
    }
}

impl HasBeginning for PointInTime {
    fn begins(&self) -> NaiveDate {
        match self {
            // PointInTime::Approximated(_) => todo!(),
            // PointInTime::ChronologicalAge(_, _) => todo!(),
            PointInTime::Time(r) => r.date(),
            PointInTime::Day(d) => d.clone(),
            PointInTime::Month(y, m) => NaiveDate::from_ymd_opt(*y, *m as u32, 1).unwrap(), // Check this later
            PointInTime::Season(y, s) => match s {
                Season::Winter => NaiveDate::from_ymd_opt(*y, 12, 1).unwrap(),
                Season::Spring => NaiveDate::from_ymd_opt(*y, 3, 1).unwrap(),
                Season::Summer => NaiveDate::from_ymd_opt(*y, 6, 1).unwrap(),
                Season::Fall => NaiveDate::from_ymd_opt(*y, 9, 1).unwrap(),
            },
            PointInTime::Year(y) => NaiveDate::from_ymd_opt(*y, 1, 1).unwrap(),
            PointInTime::Undetermined => todo!(),
            _ => unreachable!(),
        }
    }
}

pub trait MyName {
    fn name(&self) -> MaybeSignal<String>;
}

impl MyName for PointOfInterest {
    fn name(&self) -> MaybeSignal<String> {
        self.name.into()
    }
}

impl MyName for MainCategory {
    fn name(&self) -> MaybeSignal<String> {
        self.name.into()
    }
}

impl Identify for MainCategory {
    fn identity(&self) -> Identifier {
        self.id
    }
}

pub trait Identify {
    fn identity(&self) -> Identifier;
}

impl MainCategory {
    pub fn new(name: String) -> Self {
        Self {
            id: Identifier::default(),
            name: RwSignal::new(name),
        }
    }
}

pub trait LookupIdentifier {
    fn lookup();
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Ending {
    At(PointInTime),
    After, // duration
    Upon,  // med länk till en PoT
    #[default]
    Undetermined,
}

// TODO Worry later about non-person entities
#[derive(Default, Debug, Clone)]
pub struct Person {
    id: Identifier,
    name: String,
    origo: PointInTime,
}

impl Identify for Person {
    fn identity(&self) -> Identifier {
        self.id
    }
}

impl HasBeginning for Person {
    fn begins(&self) -> NaiveDate {
        self.origo.begins()
    }
}

impl MyName for Person {
    fn name(&self) -> MaybeSignal<String> {
        self.name.clone().into()
    }
}

impl Person {
    pub fn new(name: String, origo: PointInTime) -> Self {
        Self {
            name,
            origo,
            ..Default::default()
        }
    }
}

pub struct Place {}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Approximated {
    after: Option<()>,
    before: Option<()>,
    during: Option<()>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Season {
    Winter,
    Spring,
    #[default]
    Summer,
    Fall,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    #[default]
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl From<u8> for Month {
    fn from(value: u8) -> Self {
        match value {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => Month::January,
        }
    }
}

// Samma höst (eller annan upplösning) som ...

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum PointInTime {
    Approximated(Approximated),
    // TimeAfter, // duration och PoI
    ChronologicalAge(i8, Identifier), // ålder och person
    Time(NaiveDateTime),
    Day(NaiveDate),
    Month(i32, Month),
    Season(i32, Season),
    // TermOf,
    Year(i32),
    // Grade,
    // Stadium,
    // Period,
    #[default]
    Undetermined,
}

impl From<NaiveDate> for PointInTime {
    fn from(value: NaiveDate) -> Self {
        PointInTime::Day(value)
    }
}

impl From<(i32, u8)> for PointInTime {
    fn from((y, m): (i32, u8)) -> Self {
        PointInTime::Month(y, Month::from(m))
    }
}

impl From<i32> for PointInTime {
    fn from(y: i32) -> Self {
        PointInTime::Year(y)
    }
}

impl PointInTime {
    // fn grade() // and so on
}

impl PointOfInterest {
    pub fn new_barebones(parent: Identifier, name: String) -> Self {
        Self {
            parent,
            name: RwSignal::new(name),
            ..Default::default()
        }
    }

    pub fn new_bare_with_start(parent: Identifier, timeline: Identifier, name: String, starts: PointInTime) -> Self {
        Self {
            parent,
            timeline,
            name: name.into(),
            starts: starts.into(),
            ..Default::default()
        }
    }
    // fn starts();
    // fn duration();
    // fn identity();
    // fn title();
    // fn context();
    // fn color();
    // fn connected<T>;
}

pub enum Relevance {}

pub struct PeriodOfTime<T> {
    precision: T,
}

pub enum CaptureMode {}

pub struct Impact;
pub struct Calibration;
pub struct NamedEntity;
// struct NamedEntity<T>;
pub struct Tag;
pub struct RelationshipQuality;
pub struct Personality;
// enum Precision<T> {

// }

//

