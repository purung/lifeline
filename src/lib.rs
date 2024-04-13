use leptos::*;
use leptos_hotkeys::{provide_hotkeys_context, scopes};
use leptos_meta::*;
use leptos_router::*;

// Modules
mod components;
mod pages;
mod types;
mod data;
mod viewstate;

use crate::data::provide_timeline_context;
// Top-Level pages
use crate::pages::home::Home;
use crate::pages::not_found::NotFound;

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    let main_ref = create_node_ref::<html::Main>();
    provide_hotkeys_context(main_ref, false, scopes!());
    provide_timeline_context();

    view! {
        <Html lang="en" dir="ltr" attr:data-theme="light"/>

        // sets the document title
        <Title text="Welcome to Leptos CSR"/>

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <main _ref=main_ref>
            <Router>
                <Routes>
                    <Route path="/" view=Home/>
                    <Route path="/*" view=NotFound/>
                </Routes>
            </Router>
        </main>
    }
}

