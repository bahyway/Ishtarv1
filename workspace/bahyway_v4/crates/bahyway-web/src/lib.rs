pub mod etl_sim;
pub mod hubble;
pub mod notebook;
pub mod sc_ga;
pub mod schema_config;
pub mod story_engine;
pub mod tribes;

#[cfg(target_arch = "wasm32")]
use std::cell::{Cell, RefCell};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
use web_sys::{
    CanvasRenderingContext2d, Event, HtmlCanvasElement, HtmlSelectElement, HtmlTextAreaElement,
    InputEvent,
};

#[cfg(target_arch = "wasm32")]
use etl_sim::EtlSimState;
#[cfg(target_arch = "wasm32")]
use hubble::HubbleState;
#[cfg(target_arch = "wasm32")]
use notebook::{NotebookState, TokKind, PRESETS};
#[cfg(target_arch = "wasm32")]
use sc_ga::ScGaState;
#[cfg(target_arch = "wasm32")]
use schema_config::{SchemaConfigState, FORMAT_LAYERS, SOURCE_TRUST_LEVELS};
#[cfg(target_arch = "wasm32")]
use story_engine::{StewardState, StoryEngineState, STORIES};
#[cfg(target_arch = "wasm32")]
use tribes::{particle_pos, status_css_color, TribesState, MAX_ORBIT_PX, PARTICLES, TRIBES};

/// Live tribe stats fetched from /api/v1/tribes — overlaid on static tribe cards.
#[cfg(target_arch = "wasm32")]
struct LiveTribeStat {
    tribe_id: String,
    last_batch: String,
    golden: usize,
    fuzzy: usize,
    dead: usize,
    total: usize,
    batches: usize,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static NB:            RefCell<NotebookState>    = RefCell::new(NotebookState::new());
    static ETL:           RefCell<EtlSimState>      = RefCell::new(EtlSimState::new());
    static LIGHT_MODE:    Cell<bool>                = Cell::new(false);
    static TRIBES_STATE:  RefCell<TribesState>      = RefCell::new(TribesState::new());
    static STORY_STATE:    RefCell<StoryEngineState> = RefCell::new(StoryEngineState::new());
    static STORY_DRAG:     Cell<bool>  = Cell::new(false);
    static STORY_DRAG_X:   Cell<f64>  = Cell::new(0.0);
    static STORY_PAN_0:    Cell<f64>  = Cell::new(0.0);
    static STORY_MOVED:    Cell<bool>  = Cell::new(false);
    static STEWARD_STATE:  RefCell<StewardState> = RefCell::new(StewardState::new(16));
    static CTX_PARTICLE:   Cell<usize> = Cell::new(usize::MAX);
    static HUBBLE_STATE:   RefCell<HubbleState>  = RefCell::new(HubbleState::new());
    static SC_GA:          RefCell<ScGaState>    = RefCell::new(ScGaState::new());
    static SCHEMA_STATE:   RefCell<SchemaConfigState> = RefCell::new(SchemaConfigState::new());
    static LIVE_TRIBE:     RefCell<Option<LiveTribeStat>> = RefCell::new(None);
}

// ─── helpers ──────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn doc() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

#[cfg(target_arch = "wasm32")]
fn set_html(id: &str, html: &str) {
    if let Some(el) = doc().get_element_by_id(id) {
        el.set_inner_html(html);
    }
}

#[cfg(target_arch = "wasm32")]
fn set_text(id: &str, text: &str) {
    if let Some(el) = doc().get_element_by_id(id) {
        el.set_text_content(Some(text));
    }
}

#[cfg(target_arch = "wasm32")]
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Map TokKind to a CSS class name defined in index.html.
#[cfg(target_arch = "wasm32")]
fn tok_class(k: &TokKind) -> &'static str {
    match k {
        TokKind::Keyword => "tok-kw",
        TokKind::Field => "tok-fld",
        TokKind::Operator => "tok-op",
        TokKind::HexLit => "tok-lit",
        TokKind::NumLit => "tok-lit",
        TokKind::StrLit => "tok-str",
        TokKind::Punct => "tok-pun",
        TokKind::Unknown => "tok-unk",
    }
}

#[cfg(target_arch = "wasm32")]
fn lerp_rgb_f(a: [u8; 3], b: [u8; 3], t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let l = |a: u8, b: u8| ((a as f32 * (1.0 - t) + b as f32 * t) as u8);
    (l(a[0], b[0]), l(a[1], b[1]), l(a[2], b[2]))
}

/// Map particle status to a CSS class name defined in index.html.
#[cfg(target_arch = "wasm32")]
fn status_class(s: &str) -> &'static str {
    match s {
        "Active" => "s-active",
        "Critical" => "s-critical",
        "Warning" => "s-warning",
        "Watch" => "s-watch",
        "Pending" => "s-pending",
        "Archive" => "s-archive",
        "Dead" => "s-dead",
        "Fuzzy" => "s-fuzzy",
        _ => "",
    }
}

// ─── WASM entry point ─────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(|info| {
        web_sys::console::error_1(&JsValue::from_str(&format!("panic: {info}")));
    }));

    render_notebook();
    setup_notebook_events();

    render_pipeline_stats();
    render_etl_log();
    setup_pipeline_events();

    render_tribes_overview();
    setup_tribes_events();
    setup_story_engine_events();
    setup_steward_events();
    setup_hubble_events();

    render_probe();
    setup_probe_events();

    setup_schema_events();
    setup_live_data_events();

    setup_tabs();
    setup_theme();
    setup_tour();
    start_loops();
}

// ─── Notebook rendering ───────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn render_notebook() {
    NB.with(|nb| {
        let nb = nb.borrow();

        // Token stream — CSS classes, no inline colors
        let tok_html: String = nb
            .tokens()
            .iter()
            .map(|t| {
                let cls = tok_class(&t.kind);
                format!("<span class='{cls}'>{}</span>&#32;", html_escape(&t.value))
            })
            .collect();
        set_html("tok-stream", &tok_html);

        // Result count
        let results = nb.results();
        set_text(
            "result-count",
            &format!("{} particle(s) matched", results.len()),
        );

        // Results table — CSS classes for status and tribe
        let rows: String = results
            .iter()
            .map(|p| {
                let [r, g, b] = p.rgb;
                let sc = status_class(p.status);
                format!(
                    "<tr>\
                 <td>{}</td>\
                 <td class='tribe-id'>0x{:04X}</td>\
                 <td>{}</td>\
                 <td class='{sc}'>{}</td>\
                 <td style='color:rgb({r},{g},{b})'>({r},{g},{b})</td>\
                 <td style='color:var(--text-muted)'>{}</td>\
                 </tr>",
                    html_escape(p.name),
                    p.tribe_id,
                    p.epoch,
                    html_escape(p.status),
                    p.tags.join(", ")
                )
            })
            .collect();
        set_html("results-body", &rows);
    });
}

// ─── Notebook events ──────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn setup_notebook_events() {
    let d = doc();
    let container = match d.get_element_by_id("preset-btns") {
        Some(c) => c,
        None => return,
    };

    for &(label, query) in PRESETS {
        let btn = d.create_element("button").unwrap();
        btn.set_inner_html(label);
        btn.set_attribute("class", "preset-btn").ok();

        let q = query.to_string();
        let cb = Closure::wrap(Box::new(move |_: Event| {
            NB.with(|nb| nb.borrow_mut().set_query(&q));
            if let Some(ta) = doc()
                .get_element_by_id("nb-query")
                .and_then(|e| e.dyn_into::<HtmlTextAreaElement>().ok())
            {
                ta.set_value(&q);
            }
            render_notebook();
        }) as Box<dyn FnMut(Event)>);

        btn.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
            .ok();
        cb.forget();
        container.append_child(&btn).ok();
    }

    // textarea → live re-render
    let input_cb = Closure::wrap(Box::new(move |e: InputEvent| {
        if let Some(ta) = e
            .target()
            .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
        {
            let q = ta.value();
            NB.with(|nb| nb.borrow_mut().set_query(&q));
            render_notebook();
        }
    }) as Box<dyn FnMut(InputEvent)>);

    if let Some(ta) = d.get_element_by_id("nb-query") {
        ta.add_event_listener_with_callback("input", input_cb.as_ref().unchecked_ref())
            .ok();
    }
    input_cb.forget();
}

// ─── Pipeline rendering ───────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn render_pipeline_stats() {
    ETL.with(|etl| {
        let etl = etl.borrow();
        set_text("stat-sdb", &etl.sdb_pending().to_string());
        set_text("stat-odb", &etl.odb_active().to_string());
        set_text("stat-qdb", &etl.qdb_count().to_string());
        set_text("stat-journal", &etl.journal_entries().to_string());
    });
}

#[cfg(target_arch = "wasm32")]
fn render_etl_log() {
    ETL.with(|etl| {
        let etl = etl.borrow();
        let html: String = etl
            .log
            .iter()
            .rev()
            .map(|e| {
                let cls = match e.kind {
                    etl_sim::LogKind::Info => "log-info",
                    etl_sim::LogKind::Sweep => "log-sweep",
                    etl_sim::LogKind::Malware => "log-malware",
                };
                format!("<div class='{cls}'>{}</div>", html_escape(&e.text))
            })
            .collect();
        set_html("etl-log", &html);
    });
}

// ─── Pipeline events ──────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn setup_pipeline_events() {
    attach_click("btn-stage-valid", || {
        ETL.with(|etl| etl.borrow_mut().stage_valid());
        render_pipeline_stats();
        render_etl_log();
    });

    attach_click("btn-stage-malware", || {
        ETL.with(|etl| etl.borrow_mut().stage_malware());
        render_pipeline_stats();
        render_etl_log();
    });

    attach_click("btn-tick", || {
        ETL.with(|etl| etl.borrow_mut().do_tick());
        render_pipeline_stats();
        render_etl_log();
    });

    attach_click("btn-auto-toggle", || {
        let now = ETL.with(|etl| {
            let mut etl = etl.borrow_mut();
            etl.auto_tick = !etl.auto_tick;
            etl.auto_tick
        });
        let label = if now { "Auto: ON" } else { "Auto: OFF" };
        if let Some(btn) = doc().get_element_by_id("btn-auto-toggle") {
            btn.set_text_content(Some(label));
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn attach_click(id: &str, f: impl Fn() + 'static) {
    let cb = Closure::wrap(Box::new(move |_: Event| f()) as Box<dyn FnMut(Event)>);
    if let Some(el) = doc().get_element_by_id(id) {
        el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
            .ok();
    }
    cb.forget();
}

// ─── Tab switching ────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn setup_tabs() {
    switch_tab("notebook");
    attach_click("tab-hubble", || {
        switch_tab("hubble");
        render_hubble();
        let _ = try_draw_hubble_canvas();
    });
    attach_click("tab-dashboard", || {
        switch_tab("dashboard");
        render_dashboard();
    });
    attach_click("tab-notebook", || switch_tab("notebook"));
    attach_click("tab-pipeline", || switch_tab("pipeline"));
    attach_click("tab-probe", || {
        switch_tab("probe");
        render_probe();
        let _ = try_draw_probe_canvas();
    });
    attach_click("tab-tribes", || {
        switch_tab("tribes");
        fetch_tribe_stats();
        render_tribes_overview();
    });
    attach_click("tab-schema", || {
        switch_tab("schema");
        render_schema_view();
        let _ = try_draw_particles_model_canvas();
    });
}

#[cfg(target_arch = "wasm32")]
fn switch_tab(name: &'static str) {
    let d = doc();
    for &p in &[
        "hubble",
        "dashboard",
        "notebook",
        "pipeline",
        "probe",
        "tribes",
        "schema",
    ] {
        if let Some(el) = d.get_element_by_id(&format!("panel-{p}")) {
            let display = if p == name { "" } else { "none" };
            el.set_attribute("style", &format!("display:{display}"))
                .ok();
        }
    }
    for &t in &[
        "tab-hubble",
        "tab-dashboard",
        "tab-notebook",
        "tab-pipeline",
        "tab-probe",
        "tab-tribes",
        "tab-schema",
    ] {
        if let Some(el) = d.get_element_by_id(t) {
            let cls = if t == format!("tab-{name}") {
                "tab-btn active"
            } else {
                "tab-btn"
            };
            el.set_attribute("class", cls).ok();
        }
    }
}

// ─── Theme toggle ─────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn setup_theme() {
    attach_click("btn-theme", || {
        let is_light = LIGHT_MODE.with(|t| {
            let v = !t.get();
            t.set(v);
            v
        });
        apply_theme(is_light);
    });
}

#[cfg(target_arch = "wasm32")]
fn apply_theme(light: bool) {
    let theme = if light { "light" } else { "dark" };
    // ☀ (sun) for light mode, ☾ (crescent) for dark mode
    let icon = if light { "&#9728;" } else { "&#9790;" };

    if let Some(root) = doc().document_element() {
        root.set_attribute("data-theme", theme).ok();
    }
    if let Some(btn) = doc().get_element_by_id("btn-theme") {
        btn.set_inner_html(icon);
    }
}

// ─── Tour modal ───────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn setup_tour() {
    attach_click("btn-tour", || show_tour());
    attach_click("tour-close", || hide_tour());
    attach_click("tour-backdrop", || hide_tour());
    // Prevent modal click from bubbling up to backdrop
    if let Some(modal) = doc().get_element_by_id("tour-modal") {
        let stop_cb = Closure::wrap(Box::new(move |e: Event| {
            e.stop_propagation();
        }) as Box<dyn FnMut(Event)>);
        modal
            .add_event_listener_with_callback("click", stop_cb.as_ref().unchecked_ref())
            .ok();
        stop_cb.forget();
    }
}

#[cfg(target_arch = "wasm32")]
fn show_tour() {
    if let Some(el) = doc().get_element_by_id("tour-backdrop") {
        el.set_attribute("class", "open").ok();
    }
}

#[cfg(target_arch = "wasm32")]
fn hide_tour() {
    if let Some(el) = doc().get_element_by_id("tour-backdrop") {
        el.remove_attribute("class").ok();
    }
}

// ─── Animation and auto-tick loops ───────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn start_loops() {
    let win = web_sys::window().unwrap();

    // Pipeline canvas refresh: ~4 fps (static diagram, no need for 30 fps)
    let anim_cb = Closure::wrap(Box::new(|| {
        if doc()
            .get_element_by_id("panel-pipeline")
            .and_then(|e| e.get_attribute("style"))
            .map(|s| s.contains("none"))
            .unwrap_or(false)
        {
            return;
        }
        draw_particle_canvas();
    }) as Box<dyn FnMut()>);
    win.set_interval_with_callback_and_timeout_and_arguments_0(
        anim_cb.as_ref().unchecked_ref(),
        250,
    )
    .ok();
    anim_cb.forget();

    // Auto-tick: one tick every 800 ms
    let tick_cb = Closure::wrap(Box::new(|| {
        let should = ETL.with(|etl| etl.borrow().auto_tick);
        if should {
            ETL.with(|etl| etl.borrow_mut().do_tick());
            render_pipeline_stats();
            render_etl_log();
        }
    }) as Box<dyn FnMut()>);
    win.set_interval_with_callback_and_timeout_and_arguments_0(
        tick_cb.as_ref().unchecked_ref(),
        800,
    )
    .ok();
    tick_cb.forget();

    // Tribes canvas animation: ~30 fps
    let tribes_cb = Closure::wrap(Box::new(|| {
        // Only animate when tribes panel is visible
        if doc()
            .get_element_by_id("panel-tribes")
            .and_then(|e| e.get_attribute("style"))
            .map(|s| s.contains("none"))
            .unwrap_or(false)
        {
            return;
        }
        TRIBES_STATE.with(|ts| ts.borrow_mut().animate());
        let _ = try_draw_tribes_canvas();
    }) as Box<dyn FnMut()>);
    win.set_interval_with_callback_and_timeout_and_arguments_0(
        tribes_cb.as_ref().unchecked_ref(),
        33,
    )
    .ok();
    tribes_cb.forget();

    // Live MDM data auto-refresh: every 10 seconds when dashboard is visible
    let live_cb = Closure::wrap(Box::new(|| {
        let is_visible = doc()
            .get_element_by_id("panel-dashboard")
            .and_then(|e| e.get_attribute("style"))
            .map(|s| !s.contains("none"))
            .unwrap_or(false);
        if is_visible {
            fetch_live_stats();
        }
    }) as Box<dyn FnMut()>);
    win.set_interval_with_callback_and_timeout_and_arguments_0(
        live_cb.as_ref().unchecked_ref(),
        10_000,
    )
    .ok();
    live_cb.forget();

    // Batch history auto-refresh: every 30 seconds when tribes panel is visible
    let batches_cb = Closure::wrap(Box::new(|| {
        let is_visible = doc()
            .get_element_by_id("panel-tribes")
            .and_then(|e| e.get_attribute("style"))
            .map(|s| !s.contains("none"))
            .unwrap_or(false);
        if is_visible {
            fetch_batch_history();
        }
    }) as Box<dyn FnMut()>);
    win.set_interval_with_callback_and_timeout_and_arguments_0(
        batches_cb.as_ref().unchecked_ref(),
        30_000,
    )
    .ok();
    batches_cb.forget();

    // Tribe stats auto-refresh: every 60 seconds when tribes panel is visible
    let tribes_stats_cb = Closure::wrap(Box::new(|| {
        let is_visible = doc()
            .get_element_by_id("panel-tribes")
            .and_then(|e| e.get_attribute("style"))
            .map(|s| !s.contains("none"))
            .unwrap_or(false);
        if is_visible {
            fetch_tribe_stats();
        }
    }) as Box<dyn FnMut()>);
    win.set_interval_with_callback_and_timeout_and_arguments_0(
        tribes_stats_cb.as_ref().unchecked_ref(),
        60_000,
    )
    .ok();
    tribes_stats_cb.forget();

    // Hubble canvas animation: ~30 fps
    let hubble_cb = Closure::wrap(Box::new(|| {
        if doc()
            .get_element_by_id("panel-hubble")
            .and_then(|e| e.get_attribute("style"))
            .map(|s| s.contains("none"))
            .unwrap_or(false)
        {
            return;
        }
        let _ = try_draw_hubble_canvas();
    }) as Box<dyn FnMut()>);
    win.set_interval_with_callback_and_timeout_and_arguments_0(
        hubble_cb.as_ref().unchecked_ref(),
        33,
    )
    .ok();
    hubble_cb.forget();
}

// ─── Tribes rendering ─────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn render_tribes_overview() {
    let (selected, compare_a, compare_b) = TRIBES_STATE.with(|ts| {
        let ts = ts.borrow();
        (ts.selected, ts.compare_a, ts.compare_b)
    });

    // Read live tribe data (from last fetch_tribe_stats() call)
    let live = LIVE_TRIBE.with(|lt| {
        lt.borrow().as_ref().map(|s| {
            (
                s.tribe_id.clone(),
                s.last_batch.clone(),
                s.golden,
                s.fuzzy,
                s.dead,
                s.total,
                s.batches,
            )
        })
    });

    // Build tribe cards HTML — enhance matching card with live counts
    let cards_html: String = TRIBES.iter().enumerate().map(|(ti, tribe)| {
        let [r, g, b] = tribe.rgb;
        let particle_count = PARTICLES.iter().filter(|p| p.tribe_idx == ti).count();
        let orbit_count = tribe.orbits.len();
        let sel_class = if selected == Some(ti) { "tribe-card selected" } else { "tribe-card" };
        let tribe_hex = format!("0x{:04X}", tribe.id);

        let live_html = if let Some((ref lid, ref lbatch, lgold, lfuzz, _ldead, ltotal, lbatches)) = live {
            if *lid == tribe_hex {
                format!(
                    "<div class='tc-live-badge'>&#10004; LIVE</div>\
                     <div class='tc-live-stat'>\
                       <span style='color:var(--green)'>{lgold} Golden</span> &nbsp;\
                       <span style='color:var(--yellow)'>{lfuzz} Fuzzy</span> &nbsp;\
                       &middot; {ltotal} total &middot; {lbatches} batches\
                     </div>\
                     <div class='tc-live-stat' style='color:var(--text-muted)'>last: {lbatch}</div>"
                )
            } else { String::new() }
        } else { String::new() };

        format!(
            "<div class='{sel_class}' id='tribe-card-{ti}' style='border-left-color:rgb({r},{g},{b})'>\
             {live_html}\
             <div class='tc-name' style='color:rgb({r},{g},{b})'>{}</div>\
             <div class='tc-id'>{tribe_hex}</div>\
             <div class='tc-tag'>{}</div>\
             <div class='tc-stat'>{particle_count} particles &middot; {orbit_count} orbits</div>\
             </div>",
            tribe.name, tribe.tag
        )
    }).collect();

    // If live tribe_id doesn't match any static card, prepend a standalone live card
    let standalone_live =
        if let Some((ref lid, ref lbatch, lgold, lfuzz, _ldead, ltotal, lbatches)) = live {
            let matched = TRIBES.iter().any(|t| format!("0x{:04X}", t.id) == *lid);
            if !matched {
                format!(
                    "<div class='tribe-card' style='border-left-color:var(--green)'>\
                 <div class='tc-live-badge'>&#10004; LIVE DATA</div>\
                 <div class='tc-name' style='color:var(--green)'>Tribe {lid}</div>\
                 <div class='tc-id'>{lid}</div>\
                 <div class='tc-tag'>Real-time BeeMDM</div>\
                 <div class='tc-live-stat'>\
                   <span style='color:var(--green)'>{lgold} Golden</span> &nbsp;\
                   <span style='color:var(--yellow)'>{lfuzz} Fuzzy</span> &nbsp;\
                   &middot; {ltotal} total &middot; {lbatches} batches\
                 </div>\
                 <div class='tc-live-stat' style='color:var(--text-muted)'>last: {lbatch}</div>\
                 </div>"
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

    set_html("tribe-cards", &format!("{standalone_live}{cards_html}"));

    // Show/hide detail drawer
    if let Some(ti) = selected {
        render_tribe_detail(ti);
        if let Some(el) = doc().get_element_by_id("tribe-detail-drawer") {
            el.set_attribute("style", "display:block").ok();
        }
    } else {
        if let Some(el) = doc().get_element_by_id("tribe-detail-drawer") {
            el.set_attribute("style", "display:none").ok();
        }
    }

    // Re-attach card click handlers each render
    let d = doc();
    for ti in 0..TRIBES.len() {
        if let Some(card) = d.get_element_by_id(&format!("tribe-card-{ti}")) {
            let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
                TRIBES_STATE.with(|ts| {
                    let mut ts = ts.borrow_mut();
                    ts.selected = if ts.selected == Some(ti) {
                        None
                    } else {
                        Some(ti)
                    };
                });
                render_tribes_overview();
            }) as Box<dyn FnMut(web_sys::Event)>);
            card.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
        }
    }

    // Re-render compare buttons in case view is compare
    let _ = TRIBES_STATE.with(|ts| {
        let ts = ts.borrow();
        (ts.view, ts.compare_a, ts.compare_b)
    });
    render_compare_buttons(compare_a, compare_b);
}

#[cfg(target_arch = "wasm32")]
fn render_compare_buttons(compare_a: usize, compare_b: usize) {
    let btns_a: String = TRIBES.iter().enumerate().map(|(ti, tribe)| {
        let sel = if ti == compare_a { " sel" } else { "" };
        let [r, g, b] = tribe.rgb;
        format!(
            "<button class='cmp-tribe-btn{sel}' id='cmp-a-{ti}' style='color:rgb({r},{g},{b})'>{}</button>",
            tribe.name
        )
    }).collect();
    let btns_b: String = TRIBES.iter().enumerate().map(|(ti, tribe)| {
        let sel = if ti == compare_b { " sel" } else { "" };
        let [r, g, b] = tribe.rgb;
        format!(
            "<button class='cmp-tribe-btn{sel}' id='cmp-b-{ti}' style='color:rgb({r},{g},{b})'>{}</button>",
            tribe.name
        )
    }).collect();
    set_html("cmp-btns-a", &btns_a);
    set_html("cmp-btns-b", &btns_b);

    // Attach compare A/B button clicks
    let d = doc();
    for ti in 0..TRIBES.len() {
        if let Some(btn) = d.get_element_by_id(&format!("cmp-a-{ti}")) {
            let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
                TRIBES_STATE.with(|ts| ts.borrow_mut().compare_a = ti);
                let (ca, cb_idx) =
                    TRIBES_STATE.with(|ts| (ts.borrow().compare_a, ts.borrow().compare_b));
                render_compare_buttons(ca, cb_idx);
                render_compare_view();
            }) as Box<dyn FnMut(web_sys::Event)>);
            btn.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
        }
        if let Some(btn) = d.get_element_by_id(&format!("cmp-b-{ti}")) {
            let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
                TRIBES_STATE.with(|ts| ts.borrow_mut().compare_b = ti);
                let (ca, cb_idx) =
                    TRIBES_STATE.with(|ts| (ts.borrow().compare_a, ts.borrow().compare_b));
                render_compare_buttons(ca, cb_idx);
                render_compare_view();
            }) as Box<dyn FnMut(web_sys::Event)>);
            btn.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn render_tribe_detail(tribe_idx: usize) {
    let tribe = &TRIBES[tribe_idx];
    let [r, g, b] = tribe.rgb;
    set_html(
        "detail-title",
        &format!(
            "<span style='color:rgb({r},{g},{b})'>{}</span> &mdash; {}",
            tribe.name, tribe.tag
        ),
    );

    // Show live banner if this tribe's id matches the fetched live data
    let tribe_hex = format!("0x{:04X}", tribe.id);
    let live_banner = LIVE_TRIBE.with(|lt| {
        lt.borrow().as_ref().and_then(|s| {
            if s.tribe_id == tribe_hex {
                Some(format!(
                    "<div class='live-detail-header'>\
                     &#10004; Live BeeMDM data &nbsp;&middot;&nbsp; \
                     {g} Golden &nbsp; {f} Fuzzy &nbsp; {t} total &nbsp;\
                     &middot; {b} batches &nbsp; &middot; last: {lb}\
                     </div>",
                    g = s.golden,
                    f = s.fuzzy,
                    t = s.total,
                    b = s.batches,
                    lb = s.last_batch
                ))
            } else {
                None
            }
        })
    });
    if let Some(banner_html) = live_banner {
        set_html("detail-live-banner", &banner_html);
        if let Some(el) = doc().get_element_by_id("detail-live-banner") {
            el.set_attribute("style", "display:block").ok();
        }
    } else if let Some(el) = doc().get_element_by_id("detail-live-banner") {
        el.set_attribute("style", "display:none").ok();
    }

    let rows: String = PARTICLES
        .iter()
        .filter(|p| p.tribe_idx == tribe_idx)
        .map(|p| {
            let orbit_name = tribe.orbits[p.orbit_idx].name;
            let sc = status_class(p.status);
            let status_color = status_css_color(p.status);
            let drift_pct = (p.drift * 100.0) as u32;
            let tags_str = p.tags.join(", ");
            let type_color = p.ptype.css_color();
            format!(
                "<tr>\
                 <td>{}</td>\
                 <td style='color:{type_color}'>{}</td>\
                 <td>{orbit_name}</td>\
                 <td class='{sc}'>{}</td>\
                 <td>{}</td>\
                 <td style='color:{status_color}'>{drift_pct}%</td>\
                 <td style='color:var(--text-muted);font-size:11px'>{tags_str}</td>\
                 </tr>",
                p.name,
                p.ptype.label(),
                p.status,
                p.epoch
            )
        })
        .collect();
    set_html("detail-particles-body", &rows);
}

#[cfg(target_arch = "wasm32")]
fn setup_tribes_events() {
    // Sub-tab: Star Map
    attach_click("stab-map", || {
        if let Some(el) = doc().get_element_by_id("tribes-map-view") {
            el.set_attribute("style", "display:block").ok();
        }
        if let Some(el) = doc().get_element_by_id("tribes-compare-view") {
            el.set_attribute("style", "display:none").ok();
        }
        if let Some(el) = doc().get_element_by_id("stab-map") {
            el.set_attribute("class", "sub-tab active").ok();
        }
        if let Some(el) = doc().get_element_by_id("stab-compare") {
            el.set_attribute("class", "sub-tab").ok();
        }
        TRIBES_STATE.with(|ts| ts.borrow_mut().view = tribes::TribesView::Map);
    });

    // Sub-tab: Compare
    attach_click("stab-compare", || {
        if let Some(el) = doc().get_element_by_id("tribes-map-view") {
            el.set_attribute("style", "display:none").ok();
        }
        if let Some(el) = doc().get_element_by_id("tribes-compare-view") {
            el.set_attribute("style", "display:block").ok();
        }
        if let Some(el) = doc().get_element_by_id("stab-map") {
            el.set_attribute("class", "sub-tab").ok();
        }
        if let Some(el) = doc().get_element_by_id("stab-compare") {
            el.set_attribute("class", "sub-tab active").ok();
        }
        TRIBES_STATE.with(|ts| ts.borrow_mut().view = tribes::TribesView::Compare);
        render_compare_view();
    });
}

#[cfg(target_arch = "wasm32")]
fn render_compare_view() {
    let (compare_a, compare_b) = TRIBES_STATE.with(|ts| {
        let ts = ts.borrow();
        (ts.compare_a, ts.compare_b)
    });

    set_html("cmp-th-a", TRIBES[compare_a].name);
    set_html("cmp-th-b", TRIBES[compare_b].name);

    fn tribe_col_html(ti: usize) -> String {
        let tribe = &TRIBES[ti];
        let [r, g, b] = tribe.rgb;
        let tribe_particles: Vec<&tribes::RParticle> =
            PARTICLES.iter().filter(|p| p.tribe_idx == ti).collect();

        let ptype_counts =
            |pt: tribes::PType| tribe_particles.iter().filter(|p| p.ptype == pt).count();
        let status_count = |s: &str| tribe_particles.iter().filter(|p| p.status == s).count();

        let z = ptype_counts(tribes::PType::Zikru);
        let m = ptype_counts(tribes::PType::Musaru);
        let k = ptype_counts(tribes::PType::Kali);
        let a = ptype_counts(tribes::PType::Alu);
        let e = ptype_counts(tribes::PType::Eresh);

        let active = status_count("Active");
        let critical = status_count("Critical");
        let warning = status_count("Warning");
        let watch = status_count("Watch");
        let pending = status_count("Pending");
        let archive = status_count("Archive");

        format!(
            "<div class='cmp-col-name' style='color:rgb({r},{g},{b})'>{}</div>\
             <div class='cmp-col-tag'>{}</div>\
             <div class='cmp-ptype-row'>\
               <span class='cmp-ptype-chip'>Zikru ×{z}</span>\
               <span class='cmp-ptype-chip'>Musarû ×{m}</span>\
               <span class='cmp-ptype-chip'>Kali ×{k}</span>\
               <span class='cmp-ptype-chip'>Alu ×{a}</span>\
               <span class='cmp-ptype-chip'>Eresh ×{e}</span>\
             </div>\
             <div style='font-size:12px;color:var(--text-muted)'>\
               <span class='s-active'>Active:{active}</span> &nbsp;\
               <span class='s-critical'>Critical:{critical}</span> &nbsp;\
               <span class='s-warning'>Warning:{warning}</span> &nbsp;\
               <span class='s-watch'>Watch:{watch}</span> &nbsp;\
               <span class='s-pending'>Pending:{pending}</span> &nbsp;\
               <span class='s-archive'>Archive:{archive}</span>\
             </div>",
            tribe.name, tribe.tag
        )
    }

    set_html("cmp-col-a", &tribe_col_html(compare_a));
    set_html("cmp-col-b", &tribe_col_html(compare_b));

    // Comparison table rows
    fn tribe_stats(ti: usize) -> (usize, usize, usize, usize, f32, f32) {
        let ps: Vec<&tribes::RParticle> = PARTICLES.iter().filter(|p| p.tribe_idx == ti).collect();
        let total = ps.len();
        let active = ps.iter().filter(|p| p.status == "Active").count();
        let critical = ps.iter().filter(|p| p.status == "Critical").count();
        let orbits = TRIBES[ti].orbits.len();
        let avg_epoch = if ps.is_empty() {
            0.0
        } else {
            ps.iter().map(|p| p.epoch as f32).sum::<f32>() / ps.len() as f32
        };
        let max_drift = ps.iter().map(|p| p.drift).fold(0.0_f32, f32::max);
        (total, active, critical, orbits, avg_epoch, max_drift)
    }

    let (ta, aa, ca, oa, ea, da) = tribe_stats(compare_a);
    let (tb, ab, cb, ob, eb, db) = tribe_stats(compare_b);

    let rows = format!(
        "<tr><td>Particles</td><td>{ta}</td><td>{tb}</td></tr>\
         <tr><td>Active</td><td class='s-active'>{aa}</td><td class='s-active'>{ab}</td></tr>\
         <tr><td>Critical</td><td class='s-critical'>{ca}</td><td class='s-critical'>{cb}</td></tr>\
         <tr><td>Orbits</td><td>{oa}</td><td>{ob}</td></tr>\
         <tr><td>Avg Epoch</td><td>{ea:.1}</td><td>{eb:.1}</td></tr>\
         <tr><td>Max Drift</td><td>{:.0}%</td><td>{:.0}%</td></tr>",
        da * 100.0,
        db * 100.0
    );
    set_html("cmp-tbody", &rows);
}

#[cfg(target_arch = "wasm32")]
fn try_draw_tribes_canvas() -> Option<()> {
    use std::f64::consts::TAU;

    let canvas = doc()
        .get_element_by_id("tribes-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;

    let ctx = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;

    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    let light = LIGHT_MODE.with(|t| t.get());

    // Background
    ctx.set_fill_style_str(if light { "#eeeef8" } else { "#07070f" });
    ctx.fill_rect(0.0, 0.0, w, h);

    let (angles, selected) = TRIBES_STATE.with(|ts| {
        let ts = ts.borrow();
        (ts.angles.clone(), ts.selected)
    });

    for (ti, tribe) in TRIBES.iter().enumerate() {
        let cx = tribe.map_x as f64 * w;
        let cy = tribe.map_y as f64 * h;
        let [tr, tg, tb] = tribe.rgb;

        let (alpha_ring, alpha_nucleus, label_alpha) = if selected == Some(ti) {
            (0.35_f64, 1.0_f64, "0.9")
        } else if selected.is_some() {
            (0.08_f64, 0.3_f64, "0.3")
        } else {
            (0.22_f64, 0.9_f64, "0.7")
        };

        // Draw orbit rings
        for orbit in tribe.orbits {
            let r = orbit.radius as f64 * MAX_ORBIT_PX;
            ctx.set_stroke_style_str(&format!("rgba({tr},{tg},{tb},{alpha_ring:.2})"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.arc(cx, cy, r, 0.0, TAU).ok();
            ctx.stroke();
        }

        // Draw nucleus
        ctx.set_fill_style_str(&format!("rgba({tr},{tg},{tb},{alpha_nucleus:.2})"));
        ctx.begin_path();
        ctx.arc(cx, cy, 9.0, 0.0, TAU).ok();
        ctx.fill();

        // Selection ring for selected tribe
        if selected == Some(ti) {
            ctx.set_stroke_style_str(&format!("rgba({tr},{tg},{tb},0.60)"));
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.arc(cx, cy, 14.0, 0.0, TAU).ok();
            ctx.stroke();
        }

        // Tribe name label
        ctx.set_fill_style_str(&format!("rgba({tr},{tg},{tb},{label_alpha})"));
        ctx.set_font("11px monospace");
        ctx.set_text_align("center");
        ctx.fill_text(tribe.name, cx, cy + MAX_ORBIT_PX + 16.0).ok();
        ctx.set_text_align("left");

        // Draw particles for this tribe
        let tribe_particles: Vec<(usize, &tribes::RParticle)> = PARTICLES
            .iter()
            .enumerate()
            .filter(|(_, p)| p.tribe_idx == ti)
            .collect();

        for (pi, p) in &tribe_particles {
            let angle = angles[*pi];
            let (px, py) = particle_pos(p, angle, cx, cy);
            let [pr_b, pg_b, pb_b] = p.rgb;
            let (pr, pg, pb) = if p.drift > 0.60 {
                lerp_rgb_f(
                    [pr_b, pg_b, pb_b],
                    [122, 58, 58],
                    ((p.drift - 0.60) / 0.40).min(1.0),
                )
            } else if p.drift > 0.30 {
                lerp_rgb_f(
                    [pr_b, pg_b, pb_b],
                    [255, 184, 108],
                    ((p.drift - 0.30) / 0.30).min(1.0),
                )
            } else {
                (pr_b, pg_b, pb_b)
            };
            let dot_r = p.ptype.dot_radius();

            // Glow halo
            let halo_alpha = if selected == Some(ti) {
                "0.30"
            } else if selected.is_some() {
                "0.08"
            } else {
                "0.18"
            };
            ctx.set_fill_style_str(&format!("rgba({pr},{pg},{pb},{halo_alpha})"));
            ctx.begin_path();
            ctx.arc(px, py, dot_r * 2.5, 0.0, TAU).ok();
            ctx.fill();

            // Dot
            let dot_alpha = if selected.is_some() && selected != Some(ti) {
                "0.25"
            } else {
                "0.90"
            };
            ctx.set_fill_style_str(&format!("rgba({pr},{pg},{pb},{dot_alpha})"));
            ctx.begin_path();
            ctx.arc(px, py, dot_r, 0.0, TAU).ok();
            ctx.fill();

            // Name label for selected tribe
            if selected == Some(ti) {
                ctx.set_fill_style_str("rgba(205,214,244,0.80)");
                ctx.set_font("9px monospace");
                ctx.set_text_align("left");
                ctx.fill_text(p.name, px + dot_r + 3.0, py + 3.0).ok();
            }
        }
    }

    Some(())
}

// ─── 2D Canvas drawing ────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn draw_particle_canvas() {
    let _ = try_draw_canvas();
}

#[cfg(target_arch = "wasm32")]
fn try_draw_canvas() -> Option<()> {
    use std::f64::consts::TAU;

    let canvas = doc()
        .get_element_by_id("particle-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let ctx = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;

    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    let light = LIGHT_MODE.with(|t| t.get());

    // ── Background ────────────────────────────────────────────────────────────
    ctx.set_fill_style_str(if light { "#e8e8f8" } else { "#07070f" });
    ctx.fill_rect(0.0, 0.0, w, h);

    // ── Title bar ─────────────────────────────────────────────────────────────
    ctx.set_fill_style_str(if light {
        "rgba(30,30,58,0.35)"
    } else {
        "rgba(205,214,244,0.22)"
    });
    ctx.set_font("11px monospace");
    ctx.set_text_align("center");
    ctx.fill_text("EnkiDB  ·  Five-Tier Pipeline Architecture", w / 2.0, 14.0)
        .ok();
    ctx.set_text_align("left");

    // ── Collect pipeline state ────────────────────────────────────────────────
    let (sdb_n, odb_n, qdb_n, jnl_n) = ETL.with(|etl| {
        let e = etl.borrow();
        (
            e.sdb_pending(),
            e.odb_active(),
            e.qdb_count(),
            e.journal_entries(),
        )
    });
    let sdb_cols: Vec<[u8; 3]> = ETL.with(|etl| etl.borrow().sdb_vis.clone());
    let qdb_cols: Vec<[u8; 3]> = ETL.with(|etl| etl.borrow().qdb_vis.clone());
    let odb_cols: Vec<[u8; 3]> = ETL.with(|etl| etl.borrow().vis.iter().map(|v| v.rgb).collect());
    let (last_sweep, tick_now) = ETL.with(|etl| {
        let e = etl.borrow();
        (e.last_sweep_at, e.tick_count)
    });
    let log_entries: Vec<(String, String)> = ETL.with(|etl| {
        let e = etl.borrow();
        e.log
            .iter()
            .rev()
            .take(5)
            .map(|l| {
                let col = match l.kind {
                    etl_sim::LogKind::Info => "#8be9fd",
                    etl_sim::LogKind::Sweep => "#50fa7b",
                    etl_sim::LogKind::Malware => "#ff5555",
                };
                (col.to_string(), l.text.clone())
            })
            .collect()
    });

    let sweep_age = tick_now.saturating_sub(last_sweep);
    let sweep_fresh = sweep_age < 4;
    let sweep_glow = if sweep_fresh {
        1.0_f64
    } else if sweep_age < 10 {
        0.55
    } else {
        0.28
    };

    // ── Zone layout (960 × 300) ───────────────────────────────────────────────
    //  SDB  x=8   w=148
    //  SWP  x=164 w=112
    //  ODB  x=284 w=252
    //  QDB  x=544 w=172
    //  JNL  x=724 w=228
    let zy = 22.0_f64;
    let zh = h - zy - 8.0;

    let (sdb_x, sdb_w): (f64, f64) = (8.0, 148.0);
    let (swp_x, swp_w): (f64, f64) = (164.0, 112.0);
    let (odb_x, odb_w): (f64, f64) = (284.0, 252.0);
    let (qdb_x, qdb_w): (f64, f64) = (544.0, 172.0);
    let (jnl_x, jnl_w): (f64, f64) = (724.0, w - 724.0 - 8.0);

    // ── Helper: zone background + header strip ────────────────────────────────
    macro_rules! zone_header {
        ($x:expr, $w:expr, $bg:expr, $hdr:expr, $label:expr) => {{
            ctx.set_fill_style_str($bg);
            ctx.fill_rect($x, zy, $w, zh);
            ctx.set_fill_style_str($hdr);
            ctx.fill_rect($x, zy, $w, 22.0);
            ctx.set_fill_style_str(if $hdr == "#50fa7b" || $hdr == "#f1fa8c" {
                "#1e1e2e"
            } else {
                "#cdd6f4"
            });
            ctx.set_font("bold 10px monospace");
            ctx.set_text_align("center");
            ctx.fill_text($label, $x + $w / 2.0, zy + 14.0).ok();
            ctx.set_text_align("left");
        }};
    }

    // ══ SDB Zone ══════════════════════════════════════════════════════════════
    zone_header!(
        sdb_x,
        sdb_w,
        if light {
            "rgba(98,114,164,0.10)"
        } else {
            "rgba(68,71,90,0.55)"
        },
        "#6272a4",
        &format!("SDB  ·  {sdb_n}")
    );

    // Footer label
    ctx.set_fill_style_str(if light {
        "rgba(30,30,58,0.35)"
    } else {
        "rgba(139,147,198,0.45)"
    });
    ctx.set_font("9px monospace");
    ctx.set_text_align("center");
    ctx.fill_text("Staging Buffer", sdb_x + sdb_w / 2.0, zy + zh - 5.0)
        .ok();
    ctx.set_text_align("left");

    // Staged particle chips (18×18, grid of 4 cols)
    let chip = 18.0_f64;
    let cgap = 4.0_f64;
    let ccols = 4_usize;
    for (i, rgb) in sdb_cols.iter().enumerate().take(20) {
        let col = i % ccols;
        let row = i / ccols;
        let cx = sdb_x + 6.0 + col as f64 * (chip + cgap);
        let cy = zy + 28.0 + row as f64 * (chip + cgap);
        let [r, g, b] = *rgb;
        ctx.set_fill_style_str(&format!("rgba({r},{g},{b},0.88)"));
        ctx.fill_rect(cx, cy, chip, chip);
    }
    if sdb_n > 20 {
        ctx.set_fill_style_str(if light { "#6272a4" } else { "#bd93f9" });
        ctx.set_font("9px monospace");
        ctx.set_text_align("center");
        ctx.fill_text(
            &format!("+{}", sdb_n - 20),
            sdb_x + sdb_w / 2.0,
            zy + zh - 18.0,
        )
        .ok();
        ctx.set_text_align("left");
    }
    if sdb_n == 0 && odb_n == 0 {
        ctx.set_fill_style_str(if light { "#9090aa" } else { "#44475a" });
        ctx.set_font("9px monospace");
        ctx.set_text_align("center");
        ctx.fill_text("empty", sdb_x + sdb_w / 2.0, zy + zh / 2.0 + 5.0)
            .ok();
        ctx.set_text_align("left");
    }

    // ══ ValidationSweep Zone ═════════════════════════════════════════════════
    let swp_cx = swp_x + swp_w / 2.0;
    let swp_cy = zy + zh / 2.0;

    // Label above — color fades with sweep age
    let sweep_fill = format!(
        "rgba({},{},{},{sweep_glow:.2})",
        if sweep_fresh { 80 } else { 98 },
        if sweep_fresh { 250 } else { 114 },
        if sweep_fresh { 123 } else { 164 },
    );
    ctx.set_fill_style_str(&sweep_fill);
    ctx.set_font("bold 9px monospace");
    ctx.set_text_align("center");
    ctx.fill_text("Validation", swp_cx, swp_cy - 36.0).ok();
    ctx.fill_text("Sweep", swp_cx, swp_cy - 25.0).ok();

    // Three flowing arrows
    ctx.set_stroke_style_str(&sweep_fill);
    ctx.set_line_width(1.8);
    for row in 0i32..3 {
        let ay = swp_cy + (row as f64 - 1.0) * 16.0;
        let ax1 = swp_x + 10.0;
        let ax2 = swp_x + swp_w - 12.0;
        ctx.begin_path();
        ctx.move_to(ax1, ay);
        ctx.line_to(ax2, ay);
        ctx.stroke();
        // arrowhead
        ctx.begin_path();
        ctx.move_to(ax2 - 8.0, ay - 5.0);
        ctx.line_to(ax2, ay);
        ctx.line_to(ax2 - 8.0, ay + 5.0);
        ctx.stroke();
    }

    // Status indicator
    if sweep_fresh {
        ctx.set_fill_style_str("rgba(80,250,123,0.9)");
        ctx.set_font("bold 9px monospace");
        ctx.set_text_align("center");
        ctx.fill_text("● ACTIVE", swp_cx, swp_cy + 30.0).ok();
    } else if odb_n == 0 && qdb_n == 0 {
        ctx.set_fill_style_str("rgba(108,112,134,0.6)");
        ctx.set_font("9px monospace");
        ctx.set_text_align("center");
        ctx.fill_text("idle", swp_cx, swp_cy + 30.0).ok();
    } else {
        ctx.set_fill_style_str("rgba(80,250,123,0.4)");
        ctx.set_font("9px monospace");
        ctx.set_text_align("center");
        ctx.fill_text(&format!("tick {tick_now}"), swp_cx, swp_cy + 30.0)
            .ok();
    }
    ctx.set_text_align("left");

    // ══ ODB Zone ═════════════════════════════════════════════════════════════
    zone_header!(
        odb_x,
        odb_w,
        if light {
            "rgba(80,250,123,0.07)"
        } else {
            "rgba(30,55,35,0.55)"
        },
        "#50fa7b",
        &format!("ODB  ·  {odb_n}")
    );

    ctx.set_fill_style_str(if light {
        "rgba(30,58,30,0.3)"
    } else {
        "rgba(80,250,123,0.28)"
    });
    ctx.set_font("9px monospace");
    ctx.set_text_align("center");
    ctx.fill_text("Operational Database", odb_x + odb_w / 2.0, zy + zh - 5.0)
        .ok();
    ctx.set_text_align("left");

    // Orbital rings
    let odb_cx = odb_x + odb_w / 2.0;
    let odb_cy = zy + zh / 2.0 + 4.0;
    let max_r = (zh / 2.0 - 28.0).min(odb_w / 2.0 - 18.0);
    let orbit_rs = [max_r * 0.30, max_r * 0.60, max_r * 0.92];

    ctx.set_stroke_style_str(if light {
        "rgba(80,200,100,0.22)"
    } else {
        "rgba(80,250,123,0.14)"
    });
    ctx.set_line_width(1.0);
    for &or in &orbit_rs {
        ctx.begin_path();
        ctx.arc(odb_cx, odb_cy, or, 0.0, TAU).ok();
        ctx.stroke();
    }

    // Nucleus
    ctx.set_fill_style_str("#50fa7b");
    ctx.begin_path();
    ctx.arc(odb_cx, odb_cy, 5.0, 0.0, TAU).ok();
    ctx.fill();

    // ODB particles — golden-angle spiral distribution across orbits
    let golden = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    for (i, rgb) in odb_cols.iter().enumerate().take(36) {
        let or = orbit_rs[i % 3];
        let angle = i as f64 * golden;
        let px = odb_cx + or * angle.cos();
        let py = odb_cy + or * angle.sin();
        let [r, g, b] = *rgb;
        // Glow
        ctx.set_fill_style_str(&format!("rgba({r},{g},{b},0.20)"));
        ctx.begin_path();
        ctx.arc(px, py, 7.0, 0.0, TAU).ok();
        ctx.fill();
        // Dot
        ctx.set_fill_style_str(&format!("rgb({r},{g},{b})"));
        ctx.begin_path();
        ctx.arc(px, py, 3.5, 0.0, TAU).ok();
        ctx.fill();
    }
    if odb_n > 36 {
        ctx.set_fill_style_str("rgba(80,250,123,0.55)");
        ctx.set_font("9px monospace");
        ctx.set_text_align("center");
        ctx.fill_text(&format!("+{}", odb_n - 36), odb_cx, odb_cy + max_r + 12.0)
            .ok();
        ctx.set_text_align("left");
    }

    // ══ QDB Zone ═════════════════════════════════════════════════════════════
    zone_header!(
        qdb_x,
        qdb_w,
        if light {
            "rgba(255,85,85,0.08)"
        } else {
            "rgba(80,15,15,0.62)"
        },
        "#ff5555",
        &format!("QDB  ·  {qdb_n}")
    );

    ctx.set_fill_style_str(if light {
        "rgba(180,40,40,0.4)"
    } else {
        "rgba(255,85,85,0.28)"
    });
    ctx.set_font("9px monospace");
    ctx.set_text_align("center");
    ctx.fill_text("Quarantine Buffer", qdb_x + qdb_w / 2.0, zy + zh - 5.0)
        .ok();
    ctx.set_text_align("left");

    // Quarantine chips (14×14, tinted toward red)
    let qchip = 14.0_f64;
    let qgap = 3.0_f64;
    let qcols = ((qdb_w - 10.0) / (qchip + qgap)) as usize;
    for (i, rgb) in qdb_cols.iter().enumerate().take(30) {
        let col = i % qcols;
        let row = i / qcols;
        let cx = qdb_x + 5.0 + col as f64 * (qchip + qgap);
        let cy = zy + 28.0 + row as f64 * (qchip + qgap);
        let (tr, tg, tb) = lerp_rgb_f(*rgb, [220, 50, 50], 0.45);
        ctx.set_fill_style_str(&format!("rgba({tr},{tg},{tb},0.92)"));
        ctx.fill_rect(cx, cy, qchip, qchip);
        // ⚠ mark for malware (very dark red = malware staged as [220,40,40])
        if rgb[0] > 180 && rgb[1] < 60 && rgb[2] < 60 {
            ctx.set_fill_style_str("rgba(255,200,200,0.7)");
            ctx.set_font("bold 8px monospace");
            ctx.fill_text("!", cx + 4.0, cy + 11.0).ok();
        }
    }
    if qdb_n > 30 {
        ctx.set_fill_style_str("rgba(255,85,85,0.55)");
        ctx.set_font("9px monospace");
        ctx.set_text_align("center");
        ctx.fill_text(
            &format!("+{}", qdb_n - 30),
            qdb_x + qdb_w / 2.0,
            zy + zh - 18.0,
        )
        .ok();
        ctx.set_text_align("left");
    }
    if qdb_n == 0 {
        ctx.set_fill_style_str(if light { "#9090aa" } else { "#44475a" });
        ctx.set_font("9px monospace");
        ctx.set_text_align("center");
        ctx.fill_text("clean", qdb_x + qdb_w / 2.0, zy + zh / 2.0 + 5.0)
            .ok();
        ctx.set_text_align("left");
    }

    // ══ Journal Zone ══════════════════════════════════════════════════════════
    zone_header!(
        jnl_x,
        jnl_w,
        if light {
            "rgba(241,250,140,0.08)"
        } else {
            "rgba(45,40,10,0.62)"
        },
        "#f1fa8c",
        &format!("Journal  ·  {jnl_n}")
    );

    ctx.set_fill_style_str(if light {
        "rgba(100,80,0,0.35)"
    } else {
        "rgba(241,250,140,0.25)"
    });
    ctx.set_font("9px monospace");
    ctx.set_text_align("center");
    ctx.fill_text("Event Log", jnl_x + jnl_w / 2.0, zy + zh - 5.0)
        .ok();
    ctx.set_text_align("left");

    // Log entries (truncated to fit zone width)
    let max_chars = ((jnl_w - 12.0) / 5.6) as usize;
    for (i, (col, text)) in log_entries.iter().enumerate() {
        let ly = zy + 32.0 + i as f64 * 44.0;
        ctx.set_fill_style_str(col);
        ctx.set_font("9px monospace");
        let show = if text.len() > max_chars {
            &text[..max_chars]
        } else {
            text.as_str()
        };
        ctx.fill_text(show, jnl_x + 6.0, ly).ok();
        // second line if text is longer
        if text.len() > max_chars && text.len() > max_chars * 2 {
            let end2 = (max_chars * 2).min(text.len());
            ctx.set_fill_style_str(&format!("{}88", col));
            ctx.fill_text(&text[max_chars..end2], jnl_x + 6.0, ly + 11.0)
                .ok();
        } else if text.len() > max_chars {
            ctx.set_fill_style_str(&format!("{}88", col));
            ctx.fill_text(&text[max_chars..], jnl_x + 6.0, ly + 11.0)
                .ok();
        }
    }

    // ── Zone connector lines ──────────────────────────────────────────────────
    ctx.set_stroke_style_str(if light {
        "rgba(30,30,58,0.08)"
    } else {
        "rgba(205,214,244,0.06)"
    });
    ctx.set_line_width(1.0);
    for &vx in &[sdb_x + sdb_w, swp_x + swp_w, odb_x + odb_w, qdb_x + qdb_w] {
        let gap_cx = vx + 4.0;
        ctx.begin_path();
        ctx.move_to(gap_cx, zy);
        ctx.line_to(gap_cx, zy + zh);
        ctx.stroke();
    }

    Some(())
}

// ─── StoryEngine ─────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn setup_story_engine_events() {
    // Close button
    attach_click("story-close", || {
        STORY_STATE.with(|ss| ss.borrow_mut().close());
        if let Some(el) = doc().get_element_by_id("story-engine-panel") {
            el.set_attribute("style", "display:none").ok();
        }
    });

    // Pan buttons
    attach_click("story-pan-left", || {
        let n = STORY_STATE.with(|ss| {
            ss.borrow()
                .particle_idx
                .map(|pi| STORIES[pi].len())
                .unwrap_or(0)
        });
        STORY_STATE.with(|ss| ss.borrow_mut().pan(158.0, n));
        let _ = try_draw_story_canvas();
    });
    attach_click("story-pan-right", || {
        let n = STORY_STATE.with(|ss| {
            ss.borrow()
                .particle_idx
                .map(|pi| STORIES[pi].len())
                .unwrap_or(0)
        });
        STORY_STATE.with(|ss| ss.borrow_mut().pan(-158.0, n));
        let _ = try_draw_story_canvas();
    });

    // Story canvas: drag to pan + click to select node
    let story_md = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        STORY_DRAG.with(|d| d.set(true));
        STORY_MOVED.with(|m| m.set(false));
        STORY_DRAG_X.with(|x| x.set(e.client_x() as f64));
        STORY_PAN_0.with(|p| {
            let pan = STORY_STATE.with(|ss| ss.borrow().pan_x);
            p.set(pan);
        });
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);

    let story_mm = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        if !STORY_DRAG.with(|d| d.get()) {
            return;
        }
        let start_x = STORY_DRAG_X.with(|x| x.get());
        let delta = e.client_x() as f64 - start_x;
        if delta.abs() > 4.0 {
            STORY_MOVED.with(|m| m.set(true));
        }
        let pan0 = STORY_PAN_0.with(|p| p.get());
        let n = STORY_STATE.with(|ss| {
            ss.borrow()
                .particle_idx
                .map(|pi| STORIES[pi].len())
                .unwrap_or(0)
        });
        let max_pan = 0.0_f64;
        let min_pan = -(((n as f64 - 1.0) * 158.0).max(0.0));
        let new_pan = (pan0 + delta).clamp(min_pan, max_pan);
        STORY_STATE.with(|ss| ss.borrow_mut().pan_x = new_pan);
        let _ = try_draw_story_canvas();
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);

    let story_mu = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let was_dragging = STORY_DRAG.with(|d| d.get());
        STORY_DRAG.with(|d| d.set(false));
        if !was_dragging {
            return;
        }
        if STORY_MOVED.with(|m| m.get()) {
            return;
        }
        // Short click → hit-test nodes
        if let Some(canvas) = doc()
            .get_element_by_id("story-canvas")
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        {
            let rect = canvas.get_bounding_client_rect();
            let scale_x = canvas.width() as f64 / rect.width();
            let scale_y = canvas.height() as f64 / rect.height();
            let cx = (e.client_x() as f64 - rect.left()) * scale_x;
            let cy = (e.client_y() as f64 - rect.top()) * scale_y;
            let pan_x = STORY_STATE.with(|ss| ss.borrow().pan_x);
            let hit = STORY_STATE.with(|ss| {
                let ss = ss.borrow();
                let pi = ss.particle_idx?;
                story_engine::hit_node(STORIES[pi].len(), cx, cy, pan_x)
            });
            STORY_STATE.with(|ss| ss.borrow_mut().selected_node = hit);
            render_story_detail();
            let _ = try_draw_story_canvas();
        }
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);

    if let Some(sc) = doc().get_element_by_id("story-canvas") {
        sc.add_event_listener_with_callback("mousedown", story_md.as_ref().unchecked_ref())
            .ok();
        sc.add_event_listener_with_callback("mousemove", story_mm.as_ref().unchecked_ref())
            .ok();
        sc.add_event_listener_with_callback("mouseup", story_mu.as_ref().unchecked_ref())
            .ok();
    }
    story_md.forget();
    story_mm.forget();
    story_mu.forget();

    // Tribes star-map canvas: click to find nearest particle → open StoryEngine
    let tribes_click = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let canvas = match doc()
            .get_element_by_id("tribes-canvas")
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        {
            Some(c) => c,
            None => return,
        };
        let rect = canvas.get_bounding_client_rect();
        let scale_x = canvas.width() as f64 / rect.width();
        let scale_y = canvas.height() as f64 / rect.height();
        let cx = (e.client_x() as f64 - rect.left()) * scale_x;
        let cy = (e.client_y() as f64 - rect.top()) * scale_y;
        let w = canvas.width() as f64;
        let h = canvas.height() as f64;

        let angles: Vec<f32> = TRIBES_STATE.with(|ts| ts.borrow().angles.clone());

        for (pi, p) in PARTICLES.iter().enumerate() {
            let tribe = &TRIBES[p.tribe_idx];
            let tcx = tribe.map_x as f64 * w;
            let tcy = tribe.map_y as f64 * h;
            let (px, py) = particle_pos(p, angles[pi], tcx, tcy);
            let hit_r = p.ptype.dot_radius() * 3.5;
            if (cx - px) * (cx - px) + (cy - py) * (cy - py) <= hit_r * hit_r {
                // Highlight the particle's tribe on the star map
                TRIBES_STATE.with(|ts| ts.borrow_mut().selected = Some(p.tribe_idx));
                STORY_STATE.with(|ss| ss.borrow_mut().open(pi));
                render_story_header();
                render_story_chips();
                render_story_detail();
                if let Some(el) = doc().get_element_by_id("story-engine-panel") {
                    el.set_attribute("style", "display:block").ok();
                    el.scroll_into_view();
                }
                let _ = try_draw_story_canvas();
                return;
            }
        }
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    if let Some(tc) = doc().get_element_by_id("tribes-canvas") {
        tc.add_event_listener_with_callback("click", tribes_click.as_ref().unchecked_ref())
            .ok();
    }
    tribes_click.forget();
}

#[cfg(target_arch = "wasm32")]
fn render_story_header() {
    let pi = match STORY_STATE.with(|ss| ss.borrow().particle_idx) {
        Some(p) => p,
        None => return,
    };
    let p = &PARTICLES[pi];
    let tribe = &TRIBES[p.tribe_idx];
    let [tr, tg, tb] = tribe.rgb;
    let n_events = STORIES[pi].len();
    set_html(
        "story-title",
        &format!(
            "<span style='color:rgb({tr},{tg},{tb})'>{}</span>\
         <span class='story-sep'>·</span>\
         <span style='color:rgb({tr},{tg},{tb});opacity:0.7'>{}</span>\
         <span class='story-sep'>·</span><span style='color:{}'>{}</span>\
         <span class='story-sep'>·</span><span class='story-event-count'>{n_events} events</span>",
            p.name,
            tribe.name,
            p.ptype.css_color(),
            p.ptype.label()
        ),
    );
    render_story_context_bar(pi);
}

#[cfg(target_arch = "wasm32")]
fn render_story_context_bar(pi: usize) {
    let p = &PARTICLES[pi];
    let tribe = &TRIBES[p.tribe_idx];
    let orbit = &tribe.orbits[p.orbit_idx];
    let [tr, tg, tb] = tribe.rgb;

    let eff_status =
        STEWARD_STATE.with(|ss| ss.borrow().effective_status(pi, p.status).to_string());
    let status_hex = tribes::status_hex_color(&eff_status);
    let drift_pct = (p.drift * 100.0) as u32;
    let drift_hex = if p.drift > 0.60 {
        "#ff5555"
    } else if p.drift > 0.30 {
        "#ffb86c"
    } else if p.drift > 0.15 {
        "#f1fa8c"
    } else {
        "#50fa7b"
    };

    set_html(
        "story-context-bar",
        &format!(
            "<div class='scb-tribe'>\
           <div class='scb-dot' style='background:rgb({tr},{tg},{tb})'></div>\
           <span style='color:rgb({tr},{tg},{tb})'>{}</span>\
         </div>\
         <span class='scb-sep'>›</span>\
         <span class='scb-orbit'>Orbit: <strong>{}</strong></span>\
         <span class='scb-sep'>·</span>\
         <span style='color:{};font-weight:600'>{}</span>\
         <span class='scb-sep'>·</span>\
         <span style='color:{status_hex};font-weight:700'>{eff_status}</span>\
         <span class='scb-sep'>·</span>\
         <div class='scb-drift-bar'>\
           <span style='color:var(--text-muted)'>Drift</span>\
           <div class='scb-drift-track'>\
             <div class='scb-drift-fill' style='width:{drift_pct}%;background:{drift_hex}'></div>\
           </div>\
           <span style='color:{drift_hex};font-weight:600'>{drift_pct}%</span>\
         </div>",
            tribe.name,
            orbit.name,
            p.ptype.css_color(),
            p.ptype.label()
        ),
    );
    if let Some(el) = doc().get_element_by_id("story-context-bar") {
        el.set_attribute("style", "display:flex").ok();
    }
}

#[cfg(target_arch = "wasm32")]
fn render_story_chips() {
    let (pi, selected) = STORY_STATE.with(|ss| {
        let ss = ss.borrow();
        (ss.particle_idx, ss.selected_node)
    });
    let pi = match pi {
        Some(p) => p,
        None => return,
    };
    let events = STORIES[pi];

    let chips: String = events
        .iter()
        .enumerate()
        .map(|(i, ev)| {
            let sel = if selected == Some(i) {
                " se-chip-sel"
            } else {
                ""
            };
            let col = ev.kind.hex_color();
            format!(
                "<div class='se-chip{sel}' id='se-chip-{i}' style='border-top-color:{col}'>\
             <span style='color:{col}'>{} {}</span>\
             <span class='se-chip-tick'>T{}</span>\
             <span class='se-chip-state'>→ {}</span>\
             </div>",
                ev.kind.icon(),
                ev.kind.label(),
                ev.tick,
                ev.state
            )
        })
        .collect();
    set_html("story-chips", &chips);

    // Attach click handlers on chips
    for i in 0..events.len() {
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            STORY_STATE.with(|ss| ss.borrow_mut().selected_node = Some(i));
            render_story_chips();
            render_story_detail();
            let _ = try_draw_story_canvas();
        }) as Box<dyn FnMut(web_sys::Event)>);
        if let Some(el) = doc().get_element_by_id(&format!("se-chip-{i}")) {
            el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
        }
        cb.forget();
    }
}

#[cfg(target_arch = "wasm32")]
fn render_story_detail() {
    let (pi, sel) = STORY_STATE.with(|ss| {
        let ss = ss.borrow();
        (ss.particle_idx, ss.selected_node)
    });
    let pi = match pi {
        Some(p) => p,
        None => return,
    };
    if let Some(idx) = sel {
        let ev = &STORIES[pi][idx];
        let col = ev.kind.hex_color();
        set_html(
            "story-detail",
            &format!(
                "<span class='sd-icon' style='color:{col}'>{}</span>\
             <span class='sd-kind' style='color:{col}'>{}</span>\
             <span class='sd-tick'>Tick {}</span>\
             <span class='sd-arrow'>→</span>\
             <span class='sd-state'>{}</span>\
             <p class='sd-desc'>{}</p>",
                ev.kind.icon(),
                ev.kind.label(),
                ev.tick,
                ev.state,
                html_escape(ev.detail)
            ),
        );
    } else {
        set_html(
            "story-detail",
            "<span style='color:var(--text-muted);font-size:13px'>\
             Click a node on the canvas or an event chip below to inspect.</span>",
        );
    }
}

#[cfg(target_arch = "wasm32")]
fn try_draw_story_canvas() -> Option<()> {
    use std::f64::consts::TAU;

    let (pi, selected, pan_x) = STORY_STATE.with(|ss| {
        let ss = ss.borrow();
        Some((ss.particle_idx?, ss.selected_node, ss.pan_x))
    })?;

    let canvas = doc()
        .get_element_by_id("story-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let ctx = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;

    let cw = canvas.width() as f64;
    let ch = canvas.height() as f64;
    let light = LIGHT_MODE.with(|t| t.get());

    // ── Background ────────────────────────────────────────────────────────
    ctx.set_fill_style_str(if light { "#f0f0fb" } else { "#050509" });
    ctx.fill_rect(0.0, 0.0, cw, ch);

    // Dot grid
    ctx.set_fill_style_str(if light {
        "rgba(80,80,140,0.12)"
    } else {
        "rgba(100,100,180,0.12)"
    });
    let gs = 32.0_f64;
    let mut gx = 0.0_f64;
    while gx < cw {
        let mut gy = 0.0_f64;
        while gy < ch {
            ctx.begin_path();
            ctx.arc(gx, gy, 1.2, 0.0, TAU).ok();
            ctx.fill();
            gy += gs;
        }
        gx += gs;
    }

    let events = STORIES[pi];
    let n = events.len();

    // ── Timeline track ────────────────────────────────────────────────────
    let track_y = 117.0;
    ctx.set_stroke_style_str(if light {
        "rgba(80,80,160,0.18)"
    } else {
        "rgba(100,100,200,0.18)"
    });
    ctx.set_line_width(2.0);
    ctx.begin_path();
    let x0 = story_engine::node_bounds(0, pan_x).0 + 66.0; // center of first node
    let xn = story_engine::node_bounds(n - 1, pan_x).0 + 66.0;
    ctx.move_to(x0, track_y);
    ctx.line_to(xn, track_y);
    ctx.stroke();

    // ── Edges (bezier) ────────────────────────────────────────────────────
    for i in 0..n.saturating_sub(1) {
        let (x0, y0, w0, h0) = story_engine::node_bounds(i, pan_x);
        let (x1, y1, _, h1) = story_engine::node_bounds(i + 1, pan_x);
        let fx = x0 + w0;
        let fy = y0 + h0 / 2.0; // output port
        let tx = x1;
        let ty = y1 + h1 / 2.0; // input port
        let cp1x = fx + 28.0;
        let cp1y = fy;
        let cp2x = tx - 28.0;
        let cp2y = ty;

        let col = events[i].kind.hex_color();
        ctx.set_stroke_style_str(&format!("{}88", col)); // semi-transparent
        ctx.set_line_width(1.8);
        ctx.begin_path();
        ctx.move_to(fx, fy);
        ctx.bezier_curve_to(cp1x, cp1y, cp2x, cp2y, tx, ty);
        ctx.stroke();

        // Arrowhead at target
        ctx.set_fill_style_str(&format!("{}bb", col));
        ctx.begin_path();
        ctx.move_to(tx, ty);
        ctx.line_to(tx - 8.0, ty - 4.0);
        ctx.line_to(tx - 8.0, ty + 4.0);
        ctx.close_path();
        ctx.fill();
    }

    // ── Nodes ─────────────────────────────────────────────────────────────
    for (i, ev) in events.iter().enumerate() {
        let (nx, ny, nw, nh) = story_engine::node_bounds(i, pan_x);

        // Skip nodes entirely outside the viewport
        if nx + nw < 0.0 || nx > cw {
            continue;
        }

        let col = ev.kind.hex_color();
        let is_sel = selected == Some(i);

        // Node shadow / glow if selected
        if is_sel {
            ctx.set_shadow_color(col);
            ctx.set_shadow_blur(14.0);
        }

        // Node body background
        ctx.set_fill_style_str(if light { "#e8e8f8" } else { "#12121e" });
        draw_rrect(&ctx, nx, ny, nw, nh, 8.0);
        ctx.fill();

        ctx.set_shadow_blur(0.0);

        // Colored header strip (top ~22px)
        ctx.set_fill_style_str(&format!("{}33", col)); // 20% opacity
        draw_rrect(&ctx, nx, ny, nw, 22.0, 8.0);
        ctx.fill();
        // Bottom corners of header must be square
        ctx.set_fill_style_str(&format!("{}33", col));
        ctx.fill_rect(nx, ny + 14.0, nw, 8.0);

        // Node border
        let border_col = if is_sel {
            format!("{}", col)
        } else if light {
            "rgba(80,80,160,0.25)".to_string()
        } else {
            "rgba(100,100,200,0.20)".to_string()
        };
        ctx.set_stroke_style_str(&border_col);
        ctx.set_line_width(if is_sel { 2.0 } else { 1.0 });
        draw_rrect(&ctx, nx, ny, nw, nh, 8.0);
        ctx.stroke();

        // Input port (left-center)
        let py = ny + nh / 2.0;
        ctx.set_fill_style_str(if light { "#e8e8f8" } else { "#12121e" });
        ctx.set_stroke_style_str(col);
        ctx.set_line_width(1.5);
        ctx.begin_path();
        ctx.arc(nx, py, 4.5, 0.0, TAU).ok();
        ctx.fill();
        ctx.stroke();

        // Output port (right-center) — filled with kind color
        ctx.set_fill_style_str(col);
        ctx.begin_path();
        ctx.arc(nx + nw, py, 4.5, 0.0, TAU).ok();
        ctx.fill();

        // Icon + kind label in header
        ctx.set_fill_style_str(col);
        ctx.set_font("bold 11px monospace");
        ctx.set_text_align("left");
        ctx.fill_text(
            &format!("{} {}", ev.kind.icon(), ev.kind.label()),
            nx + 8.0,
            ny + 15.0,
        )
        .ok();

        // Tick in body
        ctx.set_fill_style_str(if light { "#6b6b88" } else { "#9090aa" });
        ctx.set_font("10px monospace");
        ctx.fill_text(&format!("T{}", ev.tick), nx + 8.0, ny + 34.0)
            .ok();

        // State in body (right-aligned)
        ctx.set_fill_style_str(if light { "#1e1e3a" } else { "#cdd6f4" });
        ctx.set_text_align("right");
        ctx.fill_text(ev.state, nx + nw - 8.0, ny + 34.0).ok();

        // "→ state" line
        ctx.set_fill_style_str(if light { "#6b6b88" } else { "#6c7086" });
        ctx.set_text_align("left");
        ctx.fill_text("→", nx + 8.0, ny + 46.0).ok();
    }

    // ── Scroll fade indicators ────────────────────────────────────────────
    // Scroll-shadow indicators when content extends beyond the viewport
    let max_x = 24.0 + (n as f64 - 1.0) * 158.0 + 132.0 + pan_x;
    if pan_x < -4.0 {
        ctx.set_fill_style_str(if light {
            "rgba(240,240,251,0.82)"
        } else {
            "rgba(5,5,9,0.82)"
        });
        ctx.fill_rect(0.0, 0.0, 20.0, ch);
    }
    if max_x > cw + 4.0 {
        ctx.set_fill_style_str(if light {
            "rgba(240,240,251,0.82)"
        } else {
            "rgba(5,5,9,0.82)"
        });
        ctx.fill_rect(cw - 20.0, 0.0, 20.0, ch);
    }

    // Hint text if no events visible in viewport
    ctx.set_text_align("left");

    Some(())
}

#[cfg(target_arch = "wasm32")]
fn draw_rrect(ctx: &CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::PI;
    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.line_to(x + w - r, y);
    ctx.arc(x + w - r, y + r, r, -PI / 2.0, 0.0).ok();
    ctx.line_to(x + w, y + h - r);
    ctx.arc(x + w - r, y + h - r, r, 0.0, PI / 2.0).ok();
    ctx.line_to(x + r, y + h);
    ctx.arc(x + r, y + h - r, r, PI / 2.0, PI).ok();
    ctx.line_to(x, y + r);
    ctx.arc(x + r, y + r, r, PI, PI * 1.5).ok();
    ctx.close_path();
}

// ─── Data Steward ─────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn setup_steward_events() {
    // ── Context menu on tribes star-map canvas ────────────────────────────
    let ctx_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        e.prevent_default();
        let canvas = match doc()
            .get_element_by_id("tribes-canvas")
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        {
            Some(c) => c,
            None => return,
        };
        let rect = canvas.get_bounding_client_rect();
        let scale_x = canvas.width() as f64 / rect.width();
        let scale_y = canvas.height() as f64 / rect.height();
        let cx = (e.client_x() as f64 - rect.left()) * scale_x;
        let cy = (e.client_y() as f64 - rect.top()) * scale_y;
        let w = canvas.width() as f64;
        let h = canvas.height() as f64;

        let angles: Vec<f32> = TRIBES_STATE.with(|ts| ts.borrow().angles.clone());
        for (pi, p) in PARTICLES.iter().enumerate() {
            let tribe = &TRIBES[p.tribe_idx];
            let tcx = tribe.map_x as f64 * w;
            let tcy = tribe.map_y as f64 * h;
            let (px, py) = particle_pos(p, angles[pi], tcx, tcy);
            let hr = p.ptype.dot_radius() * 3.5;
            if (cx - px) * (cx - px) + (cy - py) * (cy - py) <= hr * hr {
                CTX_PARTICLE.with(|c| c.set(pi));
                show_ctxmenu(pi, e.client_x(), e.client_y());
                return;
            }
        }
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    if let Some(tc) = doc().get_element_by_id("tribes-canvas") {
        tc.add_event_listener_with_callback("contextmenu", ctx_cb.as_ref().unchecked_ref())
            .ok();
    }
    ctx_cb.forget();

    // Close context menu on any document click
    let hide_cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
        hide_ctxmenu();
    }) as Box<dyn FnMut(web_sys::Event)>);
    if let Some(d) = web_sys::window().and_then(|w| w.document()) {
        d.add_event_listener_with_callback("click", hide_cb.as_ref().unchecked_ref())
            .ok();
    }
    hide_cb.forget();

    // Stop propagation inside context menu (so doc click doesn't close it immediately)
    let stop_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        e.stop_propagation();
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    if let Some(m) = doc().get_element_by_id("ctxmenu") {
        m.add_event_listener_with_callback("click", stop_cb.as_ref().unchecked_ref())
            .ok();
    }
    stop_cb.forget();

    // Context menu: Open StoryEngine
    // Use mousedown (fires on press, before any click-swallow by the fixed menu layer)
    let open_story_cb = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
        let pi = CTX_PARTICLE.with(|c| c.get());
        if pi == usize::MAX {
            return;
        }
        hide_ctxmenu();
        let tribe_idx = PARTICLES[pi].tribe_idx;
        TRIBES_STATE.with(|ts| ts.borrow_mut().selected = Some(tribe_idx));
        STORY_STATE.with(|ss| ss.borrow_mut().open(pi));
        render_story_header();
        render_story_chips();
        render_story_detail();
        if let Some(el) = doc().get_element_by_id("story-engine-panel") {
            el.set_attribute("style", "display:block").ok();
            el.scroll_into_view();
        }
        let _ = try_draw_story_canvas();
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    if let Some(el) = doc().get_element_by_id("ctx-open-story") {
        el.add_event_listener_with_callback("mousedown", open_story_cb.as_ref().unchecked_ref())
            .ok();
    }
    open_story_cb.forget();

    // Context menu: Diagnose + Steward Edit
    attach_click("ctx-steward", || {
        let pi = CTX_PARTICLE.with(|c| c.get());
        if pi == usize::MAX {
            return;
        }
        hide_ctxmenu();
        show_steward_sidebar(pi);
    });

    // Steward sidebar: close
    attach_click("steward-close", || hide_steward_sidebar());
    let bd_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        if let Some(t) = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        {
            if t.id() == "steward-backdrop" {
                hide_steward_sidebar();
            }
        }
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    if let Some(bd) = doc().get_element_by_id("steward-backdrop") {
        bd.add_event_listener_with_callback("click", bd_cb.as_ref().unchecked_ref())
            .ok();
    }
    bd_cb.forget();

    // Steward: Apply override button
    attach_click("steward-apply", || {
        let pi = match STEWARD_STATE.with(|ss| ss.borrow().target) {
            Some(p) => p,
            None => return,
        };
        // Read note from input
        let note = doc()
            .get_element_by_id("steward-note")
            .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|inp| inp.value())
            .unwrap_or_default();

        STEWARD_STATE.with(|ss| {
            let mut ss = ss.borrow_mut();
            let status = ss.edit_status.clone();
            let ptype = ss.edit_ptype.clone();
            ss.overrides[pi] = Some(story_engine::ParticleOverride {
                status,
                ptype,
                note,
            });
        });
        set_html("steward-apply-msg",
            "<span style='color:var(--s-active)'>✓ Override applied. Steward correction registered.</span>"
        );
        render_steward_diagnosis_section(pi);
        render_steward_journal_section(pi);
    });

    // Steward: Reset override button
    attach_click("steward-reset", || {
        let pi = match STEWARD_STATE.with(|ss| ss.borrow().target) {
            Some(p) => p,
            None => return,
        };
        STEWARD_STATE.with(|ss| {
            let mut ss = ss.borrow_mut();
            ss.overrides[pi] = None;
            let original_status = PARTICLES[pi].status;
            let original_ptype = PARTICLES[pi].ptype.label();
            ss.edit_status = original_status.to_string();
            ss.edit_ptype = original_ptype.to_string();
        });
        set_html("steward-apply-msg", "");
        render_steward_diagnosis_section(pi);
        render_steward_journal_section(pi);
        render_steward_status_pills(pi);
        render_steward_ptype_pills(pi);
    });
}

#[cfg(target_arch = "wasm32")]
fn show_ctxmenu(pi: usize, cx: i32, cy: i32) {
    let p = &PARTICLES[pi];
    let eff_status =
        STEWARD_STATE.with(|ss| ss.borrow().effective_status(pi, p.status).to_string());
    let sc = tribes::status_css_color(&eff_status);
    set_html(
        "ctx-particle-name",
        &format!(
            "{} <span style='color:{sc};font-size:11px'>{}</span>",
            p.name, eff_status
        ),
    );
    // Show/hide Diagnose option only for Dead/Fuzzy
    let diag_display = if eff_status == "Dead" || eff_status == "Fuzzy" {
        ""
    } else {
        ""
    };
    if let Some(el) = doc().get_element_by_id("ctx-steward") {
        el.set_attribute("style", &format!("display:{diag_display}"))
            .ok();
    }
    if let Some(el) = doc().get_element_by_id("ctxmenu") {
        el.set_attribute("style", &format!("left:{cx}px;top:{cy}px"))
            .ok();
        el.set_attribute("class", "open").ok();
    }
}

#[cfg(target_arch = "wasm32")]
fn hide_ctxmenu() {
    if let Some(el) = doc().get_element_by_id("ctxmenu") {
        el.remove_attribute("class").ok();
    }
}

#[cfg(target_arch = "wasm32")]
fn show_steward_sidebar(pi: usize) {
    let p = &PARTICLES[pi];
    STEWARD_STATE.with(|ss| {
        let mut ss = ss.borrow_mut();
        ss.target = Some(pi);
        // Seed edit state from override or original
        let eff_status = ss.effective_status(pi, p.status).to_string();
        let eff_ptype = ss.effective_ptype(pi, p.ptype.label()).to_string();
        ss.edit_status = eff_status;
        ss.edit_ptype = eff_ptype;
        ss.edit_note = String::new();
    });
    set_html("steward-apply-msg", "");

    render_steward_header_section(pi);
    let _ = try_draw_steward_mini_canvas(pi); // canvas now in DOM after header render
    render_steward_diagnosis_section(pi);
    render_steward_journal_section(pi);
    render_steward_status_pills(pi);
    render_steward_ptype_pills(pi);

    if let Some(el) = doc().get_element_by_id("steward-backdrop") {
        el.set_attribute("class", "open").ok();
    }
}

#[cfg(target_arch = "wasm32")]
fn hide_steward_sidebar() {
    if let Some(el) = doc().get_element_by_id("steward-backdrop") {
        el.remove_attribute("class").ok();
    }
    STEWARD_STATE.with(|ss| ss.borrow_mut().target = None);
}

#[cfg(target_arch = "wasm32")]
fn render_steward_header_section(pi: usize) {
    let p = &PARTICLES[pi];
    let tribe = &TRIBES[p.tribe_idx];
    let orbit = &tribe.orbits[p.orbit_idx];
    let [tr, tg, tb] = tribe.rgb;
    let eff_status =
        STEWARD_STATE.with(|ss| ss.borrow().effective_status(pi, p.status).to_string());
    let sc = tribes::status_css_color(&eff_status);
    let ovr = STEWARD_STATE.with(|ss| ss.borrow().overrides[pi].is_some());
    let ovr_badge = if ovr {
        "<span class='sd-ovr-badge'>✎ Override active</span>"
    } else {
        ""
    };

    // Tribe color bar at top of sidebar
    if let Some(el) = doc().get_element_by_id("steward-tribe-bar") {
        el.set_attribute(
            "style",
            &format!("height:4px;background:rgb({tr},{tg},{tb})"),
        )
        .ok();
    }

    // Mini orbital canvas + text side-by-side
    set_html(
        "steward-header-info",
        &format!(
            "<div style='display:flex;align-items:center;gap:14px'>\
         <canvas id='steward-mini-canvas' width='120' height='120' class='sd-mini-cv'></canvas>\
         <div style='flex:1;min-width:0'>\
           <div class='sd-particle-name'>{}</div>\
           <div class='sd-meta'>\
             <span style='color:rgb({tr},{tg},{tb})'>{}</span>\
             &middot;\
             <span style='color:var(--text-muted)'>Orbit:&nbsp;<strong>{}</strong></span>\
             &middot;\
             <span style='color:{}'>{}</span>\
             &middot;\
             <span style='color:{sc}'>{eff_status}</span>\
             {ovr_badge}\
           </div>\
         </div>\
         </div>",
            p.name,
            tribe.name,
            orbit.name,
            p.ptype.css_color(),
            p.ptype.label()
        ),
    );
}

#[cfg(target_arch = "wasm32")]
fn try_draw_steward_mini_canvas(pi: usize) -> Option<()> {
    use std::f64::consts::TAU;

    let canvas = doc()
        .get_element_by_id("steward-mini-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let ctx = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;

    let cw = canvas.width() as f64;
    let ch = canvas.height() as f64;
    let cx = cw / 2.0;
    let cy = ch / 2.0;
    let light = LIGHT_MODE.with(|t| t.get());

    // Background
    ctx.set_fill_style_str(if light { "#eeeef8" } else { "#07070f" });
    ctx.fill_rect(0.0, 0.0, cw, ch);

    let p = &PARTICLES[pi];
    let tribe = &TRIBES[p.tribe_idx];
    let [tr, tg, tb] = tribe.rgb;
    let max_r = 50.0_f64;

    // Orbit rings — active ring is brighter and thicker
    for (i, orbit) in tribe.orbits.iter().enumerate() {
        let r = orbit.radius as f64 * max_r;
        let is_active = i == p.orbit_idx;
        let (alpha, lw) = if is_active {
            (0.65_f64, 2.0_f64)
        } else {
            (0.18_f64, 1.0_f64)
        };
        ctx.set_stroke_style_str(&format!("rgba({tr},{tg},{tb},{alpha:.2})"));
        ctx.set_line_width(lw);
        ctx.begin_path();
        ctx.arc(cx, cy, r, 0.0, TAU).ok();
        ctx.stroke();
    }

    // Nucleus dot
    ctx.set_fill_style_str(&format!("rgb({tr},{tg},{tb})"));
    ctx.begin_path();
    ctx.arc(cx, cy, 5.0, 0.0, TAU).ok();
    ctx.fill();

    // Particle dot on active orbit at 45°
    let orbit_r = tribe.orbits[p.orbit_idx].radius as f64 * max_r;
    let angle = std::f64::consts::PI / 4.0;
    let dx = cx + orbit_r * angle.cos();
    let dy = cy + orbit_r * angle.sin();
    let [pr, pg, pb] = p.rgb;
    let dot_r = p.ptype.dot_radius();

    // Glow halo
    ctx.set_fill_style_str(&format!("rgba({pr},{pg},{pb},0.28)"));
    ctx.begin_path();
    ctx.arc(dx, dy, dot_r * 2.8, 0.0, TAU).ok();
    ctx.fill();

    // Solid dot
    ctx.set_fill_style_str(&format!("rgb({pr},{pg},{pb})"));
    ctx.begin_path();
    ctx.arc(dx, dy, dot_r, 0.0, TAU).ok();
    ctx.fill();

    Some(())
}

#[cfg(target_arch = "wasm32")]
fn render_steward_diagnosis_section(pi: usize) {
    let p = &PARTICLES[pi];
    let eff_status =
        STEWARD_STATE.with(|ss| ss.borrow().effective_status(pi, p.status).to_string());
    let html = match eff_status.as_str() {
        "Dead" => format!(
            "<div class='sd-diag-banner sd-diag-dead'>\
             <span class='sd-diag-icon'>💀</span>\
             <div>\
               <strong>Dead Particle</strong> — identity anchor unresponsive or orbit signal lost.<br>\
               Events highlighted in red below are the registered root causes.\
               A Data Steward can restore the particle by correcting its Status and Type.\
             </div>\
             </div>"
        ),
        "Fuzzy" => format!(
            "<div class='sd-diag-banner sd-diag-fuzzy'>\
             <span class='sd-diag-icon'>🌫</span>\
             <div>\
               <strong>Fuzzy Particle</strong> — contradictory drift readings or inconclusive validation.<br>\
               Events highlighted in amber below contributed to the ambiguous state.\
               Re-validate measurements or correct Status to resolve.\
             </div>\
             </div>"
        ),
        other => format!(
            "<div class='sd-diag-banner sd-diag-ok'>\
             <span class='sd-diag-icon'>✓</span>\
             <div>Particle state is <strong>{other}</strong> — no steward action required.</div>\
             </div>"
        ),
    };
    set_html("steward-diagnosis", &html);
}

#[cfg(target_arch = "wasm32")]
fn render_steward_journal_section(pi: usize) {
    let p = &PARTICLES[pi];
    let eff_status =
        STEWARD_STATE.with(|ss| ss.borrow().effective_status(pi, p.status).to_string());
    let events = STORIES[pi];
    let causal = story_engine::causal_events(&eff_status, events);

    let rows: String = events
        .iter()
        .enumerate()
        .map(|(i, ev)| {
            let is_causal = causal.contains(&i);
            let col = ev.kind.hex_color();
            let (row_cls, badge) = if is_causal {
                let badge_cls = if eff_status == "Dead" {
                    "sd-ev-badge-dead"
                } else {
                    "sd-ev-badge-fuzzy"
                };
                let label = if eff_status == "Dead" {
                    "⚠ Root Cause"
                } else {
                    "~ Contributor"
                };
                (
                    "sd-ev-row sd-ev-causal",
                    format!("<span class='{badge_cls}'>{label}</span>"),
                )
            } else {
                ("sd-ev-row", String::new())
            };
            format!(
                "<div class='{row_cls}'>\
             <span class='sd-ev-icon' style='color:{col}'>{} {}</span>\
             <span class='sd-ev-tick'>T{}</span>\
             <span class='sd-ev-arrow'>→</span>\
             <span class='sd-ev-state'>{}</span>\
             {badge}\
             <div class='sd-ev-detail'>{}</div>\
             </div>",
                ev.kind.icon(),
                ev.kind.label(),
                ev.tick,
                ev.state,
                html_escape(ev.detail)
            )
        })
        .collect();
    set_html("steward-journal", &rows);
}

#[cfg(target_arch = "wasm32")]
fn render_steward_status_pills(_pi: usize) {
    let edit_status = STEWARD_STATE.with(|ss| ss.borrow().edit_status.clone());
    const STATUSES: &[&str] = &[
        "Active", "Watch", "Warning", "Critical", "Pending", "Archive", "Fuzzy", "Dead",
    ];
    let html: String = STATUSES
        .iter()
        .map(|s| {
            let sel = if edit_status == *s {
                " se-pill-sel"
            } else {
                ""
            };
            let sc = tribes::status_css_color(s);
            format!(
                "<button class='se-pill{sel}' id='se-st-{s}' style='--pill-c:{sc}'>{s}</button>"
            )
        })
        .collect();
    set_html("steward-status-pills", &html);

    for &s in STATUSES {
        let s_str = s.to_string();
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let pi = match STEWARD_STATE.with(|ss| ss.borrow().target) {
                Some(p) => p,
                None => return,
            };
            STEWARD_STATE.with(|ss| ss.borrow_mut().edit_status = s_str.clone());
            render_steward_status_pills(pi);
        }) as Box<dyn FnMut(web_sys::Event)>);
        if let Some(el) = doc().get_element_by_id(&format!("se-st-{s}")) {
            el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
        }
        cb.forget();
    }
}

#[cfg(target_arch = "wasm32")]
fn render_steward_ptype_pills(_pi: usize) {
    let edit_ptype = STEWARD_STATE.with(|ss| ss.borrow().edit_ptype.clone());
    const PTYPES: &[tribes::PType] = &[
        tribes::PType::Zikru,
        tribes::PType::Musaru,
        tribes::PType::Kali,
        tribes::PType::Alu,
        tribes::PType::Eresh,
    ];
    let html: String = PTYPES.iter().map(|pt| {
        let lbl = pt.label();
        let id  = lbl.replace('û', "u"); // sanitise for DOM id
        let sel = if edit_ptype == lbl { " se-pill-sel" } else { "" };
        let col = pt.css_color();
        format!("<button class='se-pill{sel}' id='se-pt-{id}' style='--pill-c:{col}'>{lbl}</button>")
    }).collect();
    set_html("steward-ptype-pills", &html);

    for &pt in PTYPES {
        let lbl = pt.label();
        let id = lbl.replace('û', "u");
        let lbl_str = lbl.to_string();
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let pi = match STEWARD_STATE.with(|ss| ss.borrow().target) {
                Some(p) => p,
                None => return,
            };
            STEWARD_STATE.with(|ss| ss.borrow_mut().edit_ptype = lbl_str.clone());
            render_steward_ptype_pills(pi);
        }) as Box<dyn FnMut(web_sys::Event)>);
        if let Some(el) = doc().get_element_by_id(&format!("se-pt-{id}")) {
            el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
        }
        cb.forget();
    }
}

// ─── Dashboard ────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn render_dashboard() {
    let total = PARTICLES.len();
    let active = PARTICLES.iter().filter(|p| p.status == "Active").count();
    let at_risk = PARTICLES
        .iter()
        .filter(|p| matches!(p.status, "Warning" | "Watch" | "Critical"))
        .count();
    let dead_fuzzy = PARTICLES
        .iter()
        .filter(|p| p.status == "Dead" || p.status == "Fuzzy")
        .count();
    let overrides =
        STEWARD_STATE.with(|ss| ss.borrow().overrides.iter().filter(|o| o.is_some()).count());

    set_html(
        "dash-metrics",
        &format!(
            "<div class='dash-metric'>\
           <div class='dm-label'>Total Particles</div>\
           <div class='dm-val'>{total}</div>\
         </div>\
         <div class='dash-metric'>\
           <div class='dm-label'>Active</div>\
           <div class='dm-val dm-green'>{active}</div>\
         </div>\
         <div class='dash-metric'>\
           <div class='dm-label'>At Risk</div>\
           <div class='dm-val dm-yellow'>{at_risk}</div>\
         </div>\
         <div class='dash-metric'>\
           <div class='dm-label'>Dead / Fuzzy</div>\
           <div class='dm-val dm-red'>{dead_fuzzy}</div>\
         </div>\
         <div class='dash-metric'>\
           <div class='dm-label'>Overrides</div>\
           <div class='dm-val dm-accent'>{overrides}</div>\
         </div>"
        ),
    );

    render_tribe_health_bars();
    render_alert_queue();
    let _ = try_draw_drift_canvas();
}

#[cfg(target_arch = "wasm32")]
fn render_tribe_health_bars() {
    use tribes::status_hex_color;

    const STATUSES: &[&str] = &[
        "Active", "Watch", "Warning", "Critical", "Pending", "Archive", "Dead", "Fuzzy",
    ];

    let html: String = TRIBES.iter().enumerate().map(|(ti, tribe)| {
        let [r, g, b] = tribe.rgb;
        let ps: Vec<&tribes::RParticle> = PARTICLES.iter()
            .filter(|p| p.tribe_idx == ti).collect();
        let total = ps.len();
        if total == 0 { return String::new(); }

        let segs: String = STATUSES.iter().filter_map(|s| {
            let count = ps.iter().filter(|p| p.status == *s).count();
            if count == 0 { return None; }
            let pct = count as f64 / total as f64 * 100.0;
            let col = status_hex_color(s);
            Some(format!(
                "<div class='thb-seg' style='width:{:.1}%;background:{col}' title='{s}: {count}'></div>",
                pct
            ))
        }).collect();

        format!(
            "<div class='thb-row'>\
             <div class='thb-name' style='color:rgb({r},{g},{b})'>{}</div>\
             <div class='thb-bar'>{segs}</div>\
             <div class='thb-count'>{total} particles</div>\
             </div>",
            tribe.name
        )
    }).collect();

    set_html("dash-tribe-health", &html);
}

#[cfg(target_arch = "wasm32")]
fn render_alert_queue() {
    let alert_particles: Vec<(usize, String)> = PARTICLES
        .iter()
        .enumerate()
        .filter_map(|(pi, p)| {
            let eff =
                STEWARD_STATE.with(|ss| ss.borrow().effective_status(pi, p.status).to_string());
            if eff == "Dead" || eff == "Fuzzy" {
                Some((pi, eff))
            } else {
                None
            }
        })
        .collect();

    if alert_particles.is_empty() {
        set_html(
            "dash-alert-queue",
            "<div style='color:var(--text-muted);font-size:13px;padding:10px 0'>\
             No particles require steward attention.</div>",
        );
        return;
    }

    let rows: String = alert_particles
        .iter()
        .map(|(pi, eff)| {
            let p = &PARTICLES[*pi];
            let tribe = &TRIBES[p.tribe_idx];
            let [tr, tg, tb] = tribe.rgb;
            let sc = tribes::status_css_color(eff);
            let drift_pct = (p.drift * 100.0) as u32;
            let has_ovr = STEWARD_STATE.with(|ss| ss.borrow().overrides[*pi].is_some());
            let ovr = if has_ovr {
                "<span style='color:var(--accent);font-size:10px'> ✎</span>"
            } else {
                ""
            };
            format!(
                "<div class='daq-row'>\
             <div class='daq-name'>{}{ovr}</div>\
             <div style='color:rgb({tr},{tg},{tb});font-size:12px'>{}</div>\
             <div style='color:{sc};font-weight:700;font-size:12px'>{eff}</div>\
             <div class='daq-drift'>Drift {drift_pct}%</div>\
             <button class='daq-btn' id='daq-{pi}'>Steward Edit</button>\
             </div>",
                html_escape(p.name),
                tribe.name
            )
        })
        .collect();

    set_html("dash-alert-queue", &rows);

    for (pi, _) in &alert_particles {
        let pi = *pi;
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            show_steward_sidebar(pi);
        }) as Box<dyn FnMut(web_sys::Event)>);
        if let Some(el) = doc().get_element_by_id(&format!("daq-{pi}")) {
            el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
        }
        cb.forget();
    }
}

#[cfg(target_arch = "wasm32")]
fn try_draw_drift_canvas() -> Option<()> {
    let canvas = doc()
        .get_element_by_id("drift-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let ctx = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;

    let cw = canvas.width() as f64;
    let ch = canvas.height() as f64;
    let light = LIGHT_MODE.with(|t| t.get());

    ctx.set_fill_style_str(if light { "#eeeef8" } else { "#07070f" });
    ctx.fill_rect(0.0, 0.0, cw, ch);

    // Sort particles by drift descending
    let mut indexed: Vec<(usize, &tribes::RParticle)> = PARTICLES.iter().enumerate().collect();
    indexed.sort_by(|a, b| {
        b.1.drift
            .partial_cmp(&a.1.drift)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let n = indexed.len();
    let bar_h = 14.0_f64;
    let gap = 4.0_f64;
    let label_w = 120.0_f64;
    let chart_x = label_w + 8.0;
    let chart_w = cw - chart_x - 24.0;
    let top_pad = 22.0_f64;

    // Threshold guide lines + labels
    for &(threshold, col) in &[
        (0.15_f64, "#50fa7b"),
        (0.30_f64, "#f1fa8c"),
        (0.60_f64, "#ffb86c"),
        (0.80_f64, "#ff5555"),
    ] {
        let x = chart_x + threshold * chart_w;
        ctx.set_stroke_style_str(&format!("{col}55"));
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.move_to(x, top_pad - 8.0);
        ctx.line_to(x, top_pad + n as f64 * (bar_h + gap));
        ctx.stroke();
        ctx.set_fill_style_str(col);
        ctx.set_font("9px monospace");
        ctx.set_text_align("center");
        ctx.fill_text(&format!("{:.0}%", threshold * 100.0), x, top_pad - 2.0)
            .ok();
    }
    ctx.set_text_align("left");

    for (row, (pi, p)) in indexed.iter().enumerate() {
        let y = top_pad + row as f64 * (bar_h + gap);
        let eff = STEWARD_STATE.with(|ss| ss.borrow().effective_status(*pi, p.status).to_string());
        let col = tribes::status_hex_color(&eff);
        let bar_len = (p.drift as f64 * chart_w).min(chart_w);

        // Name label
        ctx.set_fill_style_str(if light { "#1e1e3a" } else { "#cdd6f4" });
        ctx.set_font("10px monospace");
        ctx.fill_text(&p.name.replace('_', " "), 0.0, y + bar_h - 2.0)
            .ok();

        // Bar background track
        ctx.set_fill_style_str(if light {
            "rgba(80,80,140,0.08)"
        } else {
            "rgba(100,100,200,0.08)"
        });
        ctx.fill_rect(chart_x, y, chart_w, bar_h);

        // Colored drift bar
        ctx.set_fill_style_str(&format!("{col}cc"));
        ctx.fill_rect(chart_x, y, bar_len, bar_h);

        // Drift % label
        let pct = (p.drift * 100.0) as u32;
        ctx.set_fill_style_str(if light { "#6b6b88" } else { "#9090aa" });
        ctx.set_font("9px monospace");
        ctx.fill_text(&format!("{pct}%"), chart_x + bar_len + 4.0, y + bar_h - 2.0)
            .ok();
    }

    Some(())
}

// ─── Hubble-Zooming Service ───────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn setup_hubble_events() {
    let hub_click = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let canvas = match doc()
            .get_element_by_id("hubble-canvas")
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        {
            Some(c) => c,
            None => return,
        };
        let rect = canvas.get_bounding_client_rect();
        let sx = canvas.width() as f64 / rect.width();
        let sy = canvas.height() as f64 / rect.height();
        let cx = (e.client_x() as f64 - rect.left()) * sx;
        let cy = (e.client_y() as f64 - rect.top()) * sy;
        let w = canvas.width() as f64;
        let h = canvas.height() as f64;

        let zoom = HUBBLE_STATE.with(|hs| hs.borrow().zoom.clone());
        match zoom {
            hubble::ZoomLevel::Universe => {
                for (ti, tribe) in TRIBES.iter().enumerate() {
                    let tx = tribe.map_x as f64 * w;
                    let ty = tribe.map_y as f64 * h;
                    if (cx - tx) * (cx - tx) + (cy - ty) * (cy - ty) <= 100.0 * 100.0 {
                        HUBBLE_STATE.with(|hs| hs.borrow_mut().zoom_into_tribe(ti));
                        render_hubble();
                        let _ = try_draw_hubble_canvas();
                        return;
                    }
                }
            }
            hubble::ZoomLevel::Tribe(ti) => {
                let max_r = 190.0_f64;
                let pcx = w / 2.0;
                let pcy = h / 2.0;
                let angles = TRIBES_STATE.with(|ts| ts.borrow().angles.clone());
                for (pi, p) in PARTICLES
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.tribe_idx == ti)
                {
                    let orbit_r = TRIBES[ti].orbits[p.orbit_idx].radius as f64 * max_r;
                    let actual_r = orbit_r + p.drift as f64 * max_r * 0.28;
                    let angle = angles[pi] as f64;
                    let px = pcx + actual_r * angle.cos();
                    let py = pcy + actual_r * angle.sin();
                    let hit_r = p.ptype.dot_radius() * 4.5;
                    if (cx - px) * (cx - px) + (cy - py) * (cy - py) <= hit_r * hit_r {
                        TRIBES_STATE.with(|ts| ts.borrow_mut().selected = Some(ti));
                        STORY_STATE.with(|ss| ss.borrow_mut().open(pi));
                        render_story_header();
                        render_story_chips();
                        render_story_detail();
                        switch_tab("tribes");
                        if let Some(el) = doc().get_element_by_id("story-engine-panel") {
                            el.set_attribute("style", "display:block").ok();
                            el.scroll_into_view();
                        }
                        let _ = try_draw_story_canvas();
                        return;
                    }
                }
            }
        }
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    if let Some(hc) = doc().get_element_by_id("hubble-canvas") {
        hc.add_event_listener_with_callback("click", hub_click.as_ref().unchecked_ref())
            .ok();
    }
    hub_click.forget();
}

#[cfg(target_arch = "wasm32")]
fn render_hubble() {
    let zoom = HUBBLE_STATE.with(|hs| hs.borrow().zoom.clone());
    match &zoom {
        hubble::ZoomLevel::Universe => {
            let total = PARTICLES.len();
            let active = PARTICLES.iter().filter(|p| p.status == "Active").count();
            let at_risk = PARTICLES
                .iter()
                .filter(|p| matches!(p.status, "Warning" | "Watch" | "Critical"))
                .count();
            let dead_fuzzy = PARTICLES
                .iter()
                .filter(|p| p.status == "Dead" || p.status == "Fuzzy")
                .count();
            set_html(
                "hubble-stats",
                &format!(
                    "<div class='hub-stat' style='border-left-color:#50fa7b'>\
                   <div class='hub-stat-lbl'>Total</div>\
                   <div class='hub-stat-val'>{total}</div>\
                 </div>\
                 <div class='hub-stat' style='border-left-color:#50fa7b'>\
                   <div class='hub-stat-lbl'>Active</div>\
                   <div class='hub-stat-val' style='color:#50fa7b'>{active}</div>\
                 </div>\
                 <div class='hub-stat' style='border-left-color:#ffb86c'>\
                   <div class='hub-stat-lbl'>At Risk</div>\
                   <div class='hub-stat-val' style='color:#ffb86c'>{at_risk}</div>\
                 </div>\
                 <div class='hub-stat' style='border-left-color:#ff5555'>\
                   <div class='hub-stat-lbl'>Dead/Fuzzy</div>\
                   <div class='hub-stat-val' style='color:#ff5555'>{dead_fuzzy}</div>\
                 </div>"
                ),
            );
            set_html(
                "hubble-breadcrumb",
                "<span class='hub-bc-cur'>&#127760; Universe</span>",
            );
            set_text("hubble-hint", "Click a tribe cluster to zoom in  \u{00b7}  Ring arcs = status distribution  \u{00b7}  Outer dim dots = escaped Dead/Fuzzy particles");
        }
        hubble::ZoomLevel::Tribe(ti) => {
            let tribe = &TRIBES[*ti];
            let [tr, tg, tb] = tribe.rgb;
            let ps: Vec<&tribes::RParticle> =
                PARTICLES.iter().filter(|p| p.tribe_idx == *ti).collect();
            let total = ps.len();
            let avg_drift_pct =
                (ps.iter().map(|p| p.drift as f64).sum::<f64>() / total as f64 * 100.0) as u32;
            let max_drift_pct = (ps.iter().map(|p| p.drift).fold(0.0_f32, f32::max) * 100.0) as u32;
            set_html(
                "hubble-stats",
                &format!(
                    "<div class='hub-stat' style='border-left-color:rgb({tr},{tg},{tb})'>\
                   <div class='hub-stat-lbl'>Tribe</div>\
                   <div class='hub-stat-val' style='color:rgb({tr},{tg},{tb})'>{}</div>\
                 </div>\
                 <div class='hub-stat' style='border-left-color:rgb({tr},{tg},{tb})'>\
                   <div class='hub-stat-lbl'>Particles</div>\
                   <div class='hub-stat-val'>{total}</div>\
                   <div class='hub-stat-sub'>{} orbits</div>\
                 </div>\
                 <div class='hub-stat' style='border-left-color:#ffb86c'>\
                   <div class='hub-stat-lbl'>Avg Drift</div>\
                   <div class='hub-stat-val' style='color:#ffb86c'>{avg_drift_pct}%</div>\
                 </div>\
                 <div class='hub-stat' style='border-left-color:#ff5555'>\
                   <div class='hub-stat-lbl'>Max Drift</div>\
                   <div class='hub-stat-val' style='color:#ff5555'>{max_drift_pct}%</div>\
                 </div>",
                    tribe.name,
                    tribe.orbits.len()
                ),
            );
            let ti_val = *ti;
            set_html(
                "hubble-breadcrumb",
                &format!(
                    "<span class='hub-bc-link' id='hub-back'>&#127760; Universe</span>\
                 <span style='color:var(--border2)'>&nbsp;\u{203a}&nbsp;</span>\
                 <span class='hub-bc-cur' style='color:rgb({tr},{tg},{tb})'>{}</span>",
                    tribe.name
                ),
            );
            let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
                HUBBLE_STATE.with(|hs| hs.borrow_mut().zoom_out());
                render_hubble();
                let _ = try_draw_hubble_canvas();
            }) as Box<dyn FnMut(web_sys::Event)>);
            if let Some(el) = doc().get_element_by_id("hub-back") {
                el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                    .ok();
            }
            cb.forget();
            let _ = ti_val; // suppress unused warning
            set_text("hubble-hint", "Ghost dots = nominal orbit  \u{00b7}  Lines = drift displacement  \u{00b7}  Click particle to open StoryEngine");
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn try_draw_hubble_canvas() -> Option<()> {
    let canvas = doc()
        .get_element_by_id("hubble-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let ctx = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    let light = LIGHT_MODE.with(|t| t.get());

    ctx.set_fill_style_str(if light { "#eeeef8" } else { "#07070f" });
    ctx.fill_rect(0.0, 0.0, w, h);

    let zoom = HUBBLE_STATE.with(|hs| hs.borrow().zoom.clone());
    match zoom {
        hubble::ZoomLevel::Universe => draw_hubble_universe(&ctx, w, h, light),
        hubble::ZoomLevel::Tribe(ti) => draw_hubble_tribe_zoom(&ctx, w, h, ti, light),
    }
    Some(())
}

#[cfg(target_arch = "wasm32")]
fn draw_hubble_universe(ctx: &CanvasRenderingContext2d, w: f64, h: f64, light: bool) {
    use std::f64::consts::{PI, TAU};

    let angles = TRIBES_STATE.with(|ts| ts.borrow().angles.clone());

    // Universe title
    ctx.set_fill_style_str(if light {
        "rgba(30,30,58,0.35)"
    } else {
        "rgba(205,214,244,0.25)"
    });
    ctx.set_font("11px monospace");
    ctx.set_text_align("center");
    ctx.fill_text("BahyWay Universe", w / 2.0, 18.0).ok();
    ctx.set_text_align("left");

    for (ti, tribe) in TRIBES.iter().enumerate() {
        let cx = tribe.map_x as f64 * w;
        let cy = tribe.map_y as f64 * h;
        let [tr, tg, tb] = tribe.rgb;

        let ps: Vec<&tribes::RParticle> = PARTICLES.iter().filter(|p| p.tribe_idx == ti).collect();
        let total = ps.len() as f64;
        let active = ps.iter().filter(|p| p.status == "Active").count() as f64;
        let at_risk = ps
            .iter()
            .filter(|p| matches!(p.status, "Warning" | "Watch" | "Critical"))
            .count() as f64;
        let dead_fuzzy = ps
            .iter()
            .filter(|p| p.status == "Dead" || p.status == "Fuzzy")
            .count() as f64;
        let avg_drift = ps.iter().map(|p| p.drift as f64).sum::<f64>() / total;

        // Color degradation for glow: healthy=tribe color, degraded=dark red
        let (gr, gg, gb) = lerp_rgb_f(
            [tr, tg, tb],
            [122, 58, 58],
            (avg_drift * 1.4).min(1.0) as f32,
        );

        // Nebula glow layers
        let glow_r = 72.0 + avg_drift * 30.0;
        for i in 0..5_u32 {
            let r = glow_r * (1.0 + i as f64 * 0.38);
            let a = 0.055 - i as f64 * 0.009;
            ctx.set_fill_style_str(&format!("rgba({gr},{gg},{gb},{a:.3})"));
            ctx.begin_path();
            ctx.arc(cx, cy, r, 0.0, TAU).ok();
            ctx.fill();
        }

        // Status ring chart at radius 42
        ctx.set_line_width(5.0);
        let mut angle_start = -PI / 2.0;
        for (frac, col) in [
            (active / total, "#50fa7b"),
            (at_risk / total, "#ffb86c"),
            (dead_fuzzy / total, "#ff5555"),
        ] {
            if frac <= 0.0 {
                continue;
            }
            let end = angle_start + frac * TAU;
            ctx.set_stroke_style_str(col);
            ctx.begin_path();
            ctx.arc(cx, cy, 42.0, angle_start, end).ok();
            ctx.stroke();
            angle_start = end;
        }
        // Remaining (archive/pending) — muted
        let remain = -PI / 2.0 + TAU - angle_start;
        if remain > 0.01 {
            ctx.set_stroke_style_str(if light { "#b0b0c0" } else { "#6c7086" });
            ctx.begin_path();
            ctx.arc(cx, cy, 42.0, angle_start, -PI / 2.0 + TAU).ok();
            ctx.stroke();
        }

        // Nucleus
        ctx.set_fill_style_str(&format!("rgb({tr},{tg},{tb})"));
        ctx.begin_path();
        ctx.arc(cx, cy, 11.0, 0.0, TAU).ok();
        ctx.fill();

        // Active particles on their nominal orbits (small bright dots)
        for (pi, p) in PARTICLES
            .iter()
            .enumerate()
            .filter(|(_, p)| p.tribe_idx == ti && p.status == "Active")
        {
            let orbit_r = tribe.orbits[p.orbit_idx].radius as f64 * 88.0;
            let a = angles[pi] as f64;
            let px = cx + orbit_r * a.cos();
            let py = cy + orbit_r * a.sin();
            let [pr, pg, pb] = p.rgb;
            ctx.set_fill_style_str(&format!("rgba({pr},{pg},{pb},0.75)"));
            ctx.begin_path();
            ctx.arc(px, py, 2.5, 0.0, TAU).ok();
            ctx.fill();
        }

        // Dead/Fuzzy "escaped" particles — dim, outside max orbit
        for (i, p) in ps
            .iter()
            .filter(|p| p.status == "Dead" || p.status == "Fuzzy")
            .enumerate()
        {
            let orbit_r = tribe.orbits[p.orbit_idx].radius as f64 * 88.0;
            let escape_r = orbit_r + p.drift as f64 * 28.0 + 12.0;
            let a = PI * 0.9 + i as f64 * 2.1;
            let px = cx + escape_r * a.cos();
            let py = cy + escape_r * a.sin();
            let [pr, pg, pb] = p.rgb;
            ctx.set_fill_style_str(&format!("rgba({pr},{pg},{pb},0.28)"));
            ctx.begin_path();
            ctx.arc(px, py, 3.5, 0.0, TAU).ok();
            ctx.fill();
        }

        // Tribe label
        ctx.set_fill_style_str(&format!("rgba({tr},{tg},{tb},0.85)"));
        ctx.set_font("bold 11px monospace");
        ctx.set_text_align("center");
        ctx.fill_text(tribe.name, cx, cy + 88.0 + 18.0).ok();

        let health_pct = (active / total * 100.0) as u32;
        let hcol = if active / total > 0.7 {
            "#50fa7b"
        } else if active / total > 0.4 {
            "#ffb86c"
        } else {
            "#ff5555"
        };
        ctx.set_fill_style_str(hcol);
        ctx.set_font("10px monospace");
        ctx.fill_text(&format!("{health_pct}% healthy"), cx, cy + 88.0 + 31.0)
            .ok();
        ctx.set_text_align("left");
    }
}

#[cfg(target_arch = "wasm32")]
fn draw_hubble_tribe_zoom(ctx: &CanvasRenderingContext2d, w: f64, h: f64, ti: usize, light: bool) {
    use std::f64::consts::TAU;

    let tribe = &TRIBES[ti];
    let [tr, tg, tb] = tribe.rgb;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let max_r = 190.0_f64;
    let angles = TRIBES_STATE.with(|ts| ts.borrow().angles.clone());

    // Orbit rings + labels
    for (i, orbit) in tribe.orbits.iter().enumerate() {
        let r = orbit.radius as f64 * max_r;
        let is_active_ring = PARTICLES
            .iter()
            .filter(|p| p.tribe_idx == ti && p.orbit_idx == i)
            .any(|p| p.status == "Active");
        let alpha = if is_active_ring { 0.28_f64 } else { 0.12_f64 };
        ctx.set_stroke_style_str(&format!("rgba({tr},{tg},{tb},{alpha:.2})"));
        ctx.set_line_width(1.2);
        ctx.begin_path();
        ctx.arc(cx, cy, r, 0.0, TAU).ok();
        ctx.stroke();
        ctx.set_fill_style_str(&format!("rgba({tr},{tg},{tb},0.45)"));
        ctx.set_font("10px monospace");
        ctx.set_text_align("left");
        ctx.fill_text(orbit.name, cx + r + 5.0, cy + 4.0).ok();
    }

    // Nucleus
    ctx.set_fill_style_str(&format!("rgb({tr},{tg},{tb})"));
    ctx.begin_path();
    ctx.arc(cx, cy, 10.0, 0.0, TAU).ok();
    ctx.fill();

    // Particles — orbit decay + color degradation + ghost + trail
    for (pi, p) in PARTICLES
        .iter()
        .enumerate()
        .filter(|(_, p)| p.tribe_idx == ti)
    {
        let orbit_r = tribe.orbits[p.orbit_idx].radius as f64 * max_r;
        let drift_off = p.drift as f64 * max_r * 0.28;
        let actual_r = orbit_r + drift_off;
        let angle = angles[pi] as f64;
        let px = cx + actual_r * angle.cos();
        let py = cy + actual_r * angle.sin();

        // Ghost at nominal position + drift trail
        if p.drift > 0.08 {
            let gx = cx + orbit_r * angle.cos();
            let gy = cy + orbit_r * angle.sin();
            ctx.set_stroke_style_str(&format!("rgba({tr},{tg},{tb},0.18)"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(gx, gy);
            ctx.line_to(px, py);
            ctx.stroke();
            ctx.set_fill_style_str(&format!("rgba({tr},{tg},{tb},0.14)"));
            ctx.begin_path();
            ctx.arc(gx, gy, p.ptype.dot_radius() * 0.75, 0.0, TAU).ok();
            ctx.fill();
        }

        // Color degradation
        let [pr_b, pg_b, pb_b] = p.rgb;
        let (dr, dg, db) = if p.drift > 0.60 {
            lerp_rgb_f(
                [pr_b, pg_b, pb_b],
                [122, 58, 58],
                ((p.drift - 0.60) / 0.40).min(1.0),
            )
        } else if p.drift > 0.30 {
            lerp_rgb_f(
                [pr_b, pg_b, pb_b],
                [255, 184, 108],
                ((p.drift - 0.30) / 0.30).min(1.0),
            )
        } else {
            (pr_b, pg_b, pb_b)
        };

        let dot_r = p.ptype.dot_radius() * (1.0 + p.drift as f64 * 1.6);

        // Glow
        ctx.set_fill_style_str(&format!("rgba({dr},{dg},{db},0.22)"));
        ctx.begin_path();
        ctx.arc(px, py, dot_r * 2.6, 0.0, TAU).ok();
        ctx.fill();

        // Dot
        ctx.set_fill_style_str(&format!("rgb({dr},{dg},{db})"));
        ctx.begin_path();
        ctx.arc(px, py, dot_r, 0.0, TAU).ok();
        ctx.fill();

        // Name label
        ctx.set_fill_style_str(if light {
            "rgba(30,30,58,0.75)"
        } else {
            "rgba(205,214,244,0.75)"
        });
        ctx.set_font("9px monospace");
        ctx.set_text_align("left");
        ctx.fill_text(p.name, px + dot_r + 3.0, py + 3.0).ok();

        // Status warning for Dead/Fuzzy
        let eff = STEWARD_STATE.with(|ss| ss.borrow().effective_status(pi, p.status).to_string());
        if eff == "Dead" || eff == "Fuzzy" {
            let sc = tribes::status_hex_color(&eff);
            ctx.set_fill_style_str(sc);
            ctx.set_font("bold 9px monospace");
            ctx.fill_text(&format!("\u{26a0} {eff}"), px + dot_r + 3.0, py + 14.0)
                .ok();
        }
    }

    // Tribe title + tag
    ctx.set_fill_style_str(&format!("rgb({tr},{tg},{tb})"));
    ctx.set_font("bold 13px monospace");
    ctx.set_text_align("center");
    ctx.fill_text(tribe.name, cx, 20.0).ok();
    ctx.set_fill_style_str(if light { "#6b6b88" } else { "#9090aa" });
    ctx.set_font("10px monospace");
    ctx.fill_text(tribe.tag, cx, 33.0).ok();
    ctx.set_text_align("left");
}

// ─── SC+GA PROBE ─────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn setup_probe_events() {
    // epsilon + button
    attach_click("btn-probe-plus", || {
        SC_GA.with(|s| s.borrow_mut().adjust_epsilon(0.05));
        render_probe();
        let _ = try_draw_probe_canvas();
    });
    // epsilon - button
    attach_click("btn-probe-minus", || {
        SC_GA.with(|s| s.borrow_mut().adjust_epsilon(-0.05));
        render_probe();
        let _ = try_draw_probe_canvas();
    });
    // Run PROBE
    attach_click("btn-run-probe", || {
        SC_GA.with(|s| s.borrow_mut().run());
        render_probe();
        let _ = try_draw_probe_canvas();
    });
}

#[cfg(target_arch = "wasm32")]
fn render_probe() {
    let (eps, ran, beta0, beta1, edges, triangles) = SC_GA.with(|s| {
        let s = s.borrow();
        let (b0, b1, e, t) =
            s.sc.as_ref()
                .map(|sc| (sc.beta0, sc.beta1, sc.edges.len(), sc.triangles.len()))
                .unwrap_or((0, 0, 0, 0));
        (s.epsilon, s.ran, b0, b1, e, t)
    });

    // Epsilon display
    set_text("probe-epsilon", &format!("epsilon = {eps:.2}"));

    if !ran {
        set_html("probe-metrics", "<span style='color:var(--text-muted);font-size:12px'>Click <strong>Run PROBE</strong> to compute the Simplicial Complex and GA path.</span>");
        set_html("probe-ga-path", "");
        set_html("probe-verdict", "");
        return;
    }

    // ── Metrics row ──────────────────────────────────────────────────────────
    let n = sc_ga::RATS.len();
    let metrics = format!(
        "<div class='prb-metric'><div class='prb-lbl'>Vertices V</div>\
         <div class='prb-val'>{n}</div><div class='prb-sub'>rat particles</div></div>\
         <div class='prb-metric'><div class='prb-lbl'>Edges E</div>\
         <div class='prb-val' style='color:#f1fa8c'>{edges}</div><div class='prb-sub'>1-simplices</div></div>\
         <div class='prb-metric'><div class='prb-lbl'>Triangles T</div>\
         <div class='prb-val' style='color:#ffb86c'>{triangles}</div><div class='prb-sub'>2-simplices</div></div>\
         <div class='prb-metric'><div class='prb-lbl'>&beta;&#8320; Components</div>\
         <div class='prb-val' style='color:#50fa7b'>{beta0}</div><div class='prb-sub'>colonies</div></div>\
         <div class='prb-metric'><div class='prb-lbl'>&beta;&#8321; Loops</div>\
         <div class='prb-val' style='color:#ff79c6'>{beta1}</div><div class='prb-sub'>rat circuits</div></div>"
    );
    set_html("probe-metrics", &metrics);

    // ── GA path steps ────────────────────────────────────────────────────────
    let steps_html = SC_GA.with(|s| {
        s.borrow()
            .ga
            .as_ref()
            .map(|ga| {
                ga.steps
                    .iter()
                    .enumerate()
                    .map(|(i, step)| {
                        let col = if i == 0 {
                            "#8be9fd"
                        } else if step.rotor_deg.abs() > 0.1 {
                            "#ff79c6"
                        } else {
                            "#6c7086"
                        };
                        let rotor_str = if step.rotor_deg.abs() < 0.1 {
                            String::from("translate")
                        } else {
                            format!(
                                "R&#8336;: e^({:+.0}&deg;&middot;e&#x2081;&#x2082;)",
                                step.rotor_deg
                            )
                        };
                        format!(
                            "<div class='prb-step'>\
                     <span class='prb-step-i' style='color:{col}'>{i}</span>\
                     <span class='prb-step-op' style='color:{col}'>{rotor_str}</span>\
                     <span class='prb-step-r'>r={:.2}</span>\
                     <span class='prb-step-note'>{}</span></div>",
                            step.radius, step.note
                        )
                    })
                    .collect::<String>()
            })
            .unwrap_or_default()
    });
    set_html(
        "probe-ga-path",
        &format!("<div class='prb-section-title'>Cl(2,0) Rotor Navigation Path</div>{steps_html}"),
    );

    // ── Verdict ──────────────────────────────────────────────────────────────
    let colony_word = if beta0 == 1 {
        "1 unified colony".to_string()
    } else {
        format!("{beta0} isolated colonies")
    };
    let circuit_word = if beta1 == 0 {
        "no circuit loops".to_string()
    } else if beta1 == 1 {
        "1 rat circuit loop".to_string()
    } else {
        format!("{beta1} independent rat circuit loops")
    };
    let verdict = format!(
        "<span style='color:#ff5555;font-weight:bold'>PROBE VERDICT:</span>\
         &nbsp; Graveyard Necropolis topology confirmed &mdash; \
         {colony_word} detected, {circuit_word} operational \
         (beta&#8321; = {beta1} independent cycles around the perimeter).\
         &nbsp; GA optimal path threads {n_rotors} Cl(2,0) rotors &mdash; \
         slip through {n_rotors} patrol gaps &mdash; \
         RatLord [Dead] located at Sewer Nucleus (sector 3, &ang;90&deg;, r=0.08).\
         &nbsp; Underbelly [Fuzzy] status ambiguous: schedule follow-up PROBE.\
         &mdash;&mdash;\
         &nbsp;<em>In WPDEngine this is your pipeline loop-detection;\
         in NajafEngine this is your crowd-circuit analysis.\
         Same math &mdash; different rats.</em>",
        n_rotors = 4,
    );
    set_html("probe-verdict", &verdict);
}

#[cfg(target_arch = "wasm32")]
fn try_draw_probe_canvas() -> Option<()> {
    use std::f64::consts::TAU;

    let canvas = doc()
        .get_element_by_id("probe-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let ctx = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;

    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    let light = LIGHT_MODE.with(|t| t.get());

    // ── Background ────────────────────────────────────────────────────────────
    ctx.set_fill_style_str(if light { "#0d140d" } else { "#050a05" });
    ctx.fill_rect(0.0, 0.0, w, h);

    // Subtle grid
    ctx.set_stroke_style_str(if light {
        "rgba(80,200,80,0.06)"
    } else {
        "rgba(80,200,80,0.04)"
    });
    ctx.set_line_width(0.5);
    let step = 60.0_f64;
    let mut gx = 0.0_f64;
    while gx < w {
        ctx.begin_path();
        ctx.move_to(gx, 0.0);
        ctx.line_to(gx, h);
        ctx.stroke();
        gx += step;
    }
    let mut gy = 0.0_f64;
    while gy < h {
        ctx.begin_path();
        ctx.move_to(0.0, gy);
        ctx.line_to(w, gy);
        ctx.stroke();
        gy += step;
    }

    // Title
    ctx.set_fill_style_str("rgba(100,220,100,0.25)");
    ctx.set_font("11px monospace");
    ctx.set_text_align("center");
    ctx.fill_text(
        "Graveyard Necropolis  \u{00b7}  SC+GA PROBE (Demonstration)",
        w / 2.0,
        16.0,
    )
    .ok();
    ctx.set_text_align("left");

    let cx = w / 2.0;
    let cy = h / 2.0;
    let max_r = (h / 2.0 - 30.0).min(w / 2.0 - 80.0);

    // ── Sector orbit rings ────────────────────────────────────────────────────
    let sector_colors: &[&str] = &[
        "rgba(140,140,150,0.18)",
        "rgba(210,162,70,0.20)",
        "rgba(220,70,70,0.22)",
        "rgba(90,25,25,0.30)",
    ];
    let sector_labels: &[&str] = &["Outer Perimeter", "Mid Yard", "Inner Crypt", "Sewer"];
    ctx.set_line_width(1.0);
    for (i, &r) in sc_ga::SECTOR_RADII.iter().enumerate() {
        let rpx = r as f64 * max_r;
        ctx.set_stroke_style_str(sector_colors[i]);
        ctx.begin_path();
        ctx.arc(cx, cy, rpx, 0.0, TAU).ok();
        ctx.stroke();
        // sector label at rightmost point
        ctx.set_fill_style_str(sector_colors[i]);
        ctx.set_font("9px monospace");
        ctx.set_text_align("left");
        ctx.fill_text(sector_labels[i], cx + rpx + 4.0, cy + 3.0)
            .ok();
    }

    // ── Collect SC state ──────────────────────────────────────────────────────
    let (ran, edges, triangles, ga_pts) = SC_GA.with(|s| {
        let s = s.borrow();
        let edges = s.sc.as_ref().map(|sc| sc.edges.clone()).unwrap_or_default();
        let triangles =
            s.sc.as_ref()
                .map(|sc| sc.triangles.clone())
                .unwrap_or_default();
        let ga_pts =
            s.ga.as_ref()
                .map(|ga| ga.path_points.clone())
                .unwrap_or_default();
        (s.ran, edges, triangles, ga_pts)
    });

    // Rat positions in pixel space
    let rat_px: Vec<(f64, f64)> = sc_ga::RATS
        .iter()
        .map(|r| {
            let (nx, ny) = sc_ga::rat_pos(r);
            (cx + nx as f64 * max_r, cy - ny as f64 * max_r) // flip y for canvas coords
        })
        .collect();

    // ── 2-simplices (triangles, filled) ──────────────────────────────────────
    if ran {
        ctx.set_fill_style_str("rgba(241,250,140,0.07)");
        for &(i, j, k) in &triangles {
            let (ax, ay) = rat_px[i];
            let (bx, by) = rat_px[j];
            let (ccx, ccy) = rat_px[k];
            ctx.begin_path();
            ctx.move_to(ax, ay);
            ctx.line_to(bx, by);
            ctx.line_to(ccx, ccy);
            ctx.close_path();
            ctx.fill();
        }
    }

    // ── 1-simplices (edges) ───────────────────────────────────────────────────
    if ran {
        ctx.set_stroke_style_str("rgba(241,250,140,0.30)");
        ctx.set_line_width(0.8);
        for &(i, j) in &edges {
            let (ax, ay) = rat_px[i];
            let (bx, by) = rat_px[j];
            ctx.begin_path();
            ctx.move_to(ax, ay);
            ctx.line_to(bx, by);
            ctx.stroke();
        }
    }

    // ── GA path ───────────────────────────────────────────────────────────────
    if ran && ga_pts.len() > 1 {
        // Glow layer
        ctx.set_stroke_style_str("rgba(139,233,253,0.25)");
        ctx.set_line_width(5.0);
        ctx.begin_path();
        for (i, &(nx, ny)) in ga_pts.iter().enumerate() {
            let px = cx + nx as f64 * max_r;
            let py = cy - ny as f64 * max_r;
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.stroke();
        // Sharp path
        ctx.set_stroke_style_str("rgba(139,233,253,0.80)");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        for (i, &(nx, ny)) in ga_pts.iter().enumerate() {
            let px = cx + nx as f64 * max_r;
            let py = cy - ny as f64 * max_r;
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.stroke();
        // Waypoint dots
        for (i, &(nx, ny)) in ga_pts.iter().enumerate() {
            let px = cx + nx as f64 * max_r;
            let py = cy - ny as f64 * max_r;
            let is_rotor = SC_GA.with(|s| {
                s.borrow()
                    .ga
                    .as_ref()
                    .and_then(|ga| ga.steps.get(i))
                    .map(|st| st.rotor_deg.abs() > 0.1)
                    .unwrap_or(false)
            });
            ctx.set_fill_style_str(if is_rotor {
                "rgba(255,121,198,0.9)"
            } else {
                "rgba(139,233,253,0.7)"
            });
            ctx.begin_path();
            ctx.arc(px, py, if is_rotor { 4.5 } else { 3.0 }, 0.0, TAU)
                .ok();
            ctx.fill();
        }
    }

    // ── Rat particles ─────────────────────────────────────────────────────────
    for (i, rat) in sc_ga::RATS.iter().enumerate() {
        let (px, py) = rat_px[i];
        let [r, g, b] = rat.4;

        // Glow
        ctx.set_fill_style_str(&format!("rgba({r},{g},{b},0.18)"));
        ctx.begin_path();
        ctx.arc(px, py, 11.0, 0.0, TAU).ok();
        ctx.fill();
        // Dot
        ctx.set_fill_style_str(&format!("rgb({r},{g},{b})"));
        ctx.begin_path();
        ctx.arc(px, py, 4.5, 0.0, TAU).ok();
        ctx.fill();
        // Name label (short, on right side to avoid clutter)
        ctx.set_fill_style_str(&format!("rgba({r},{g},{b},0.70)"));
        ctx.set_font("8px monospace");
        ctx.set_text_align("left");
        ctx.fill_text(rat.0, px + 6.5, py + 3.0).ok();
    }

    // ── Graveyard nucleus ─────────────────────────────────────────────────────
    ctx.set_fill_style_str("rgba(90,25,25,0.6)");
    ctx.begin_path();
    ctx.arc(cx, cy, 8.0, 0.0, TAU).ok();
    ctx.fill();
    ctx.set_fill_style_str("rgba(90,25,25,0.9)");
    ctx.begin_path();
    ctx.arc(cx, cy, 3.5, 0.0, TAU).ok();
    ctx.fill();
    ctx.set_fill_style_str("rgba(180,60,60,0.5)");
    ctx.set_font("bold 9px monospace");
    ctx.set_text_align("center");
    ctx.fill_text("Tomb", cx, cy + 19.0).ok();
    ctx.set_text_align("left");

    // ── Legend ────────────────────────────────────────────────────────────────
    let lx = 12.0_f64;
    let mut ly = h - 70.0;
    let items: &[(&str, &str)] = &[
        ("rgba(241,250,140,0.30)", "1-simplex edge (proximity)"),
        ("rgba(241,250,140,0.07)", "2-simplex triangle (clique)"),
        ("rgba(139,233,253,0.80)", "GA rotor path Cl(2,0)"),
        ("rgba(255,121,198,0.90)", "rotor waypoint R\u{2099}"),
    ];
    ctx.set_font("9px monospace");
    for &(col, label) in items {
        ctx.set_fill_style_str(col);
        ctx.fill_rect(lx, ly - 6.0, 14.0, 8.0);
        ctx.set_fill_style_str("rgba(180,220,180,0.55)");
        ctx.fill_text(label, lx + 18.0, ly + 2.0).ok();
        ly += 16.0;
    }

    // Stats overlay (top-right)
    if ran {
        let (e_n, t_n, b0, b1) = SC_GA.with(|s| {
            let s = s.borrow();
            s.sc.as_ref()
                .map(|sc| (sc.edges.len(), sc.triangles.len(), sc.beta0, sc.beta1))
                .unwrap_or_default()
        });
        let eps = SC_GA.with(|s| s.borrow().epsilon);
        ctx.set_fill_style_str("rgba(20,35,20,0.72)");
        ctx.fill_rect(w - 155.0, 22.0, 143.0, 72.0);
        ctx.set_font("9px monospace");
        ctx.set_text_align("left");
        for (i, line) in [
            format!("epsilon  = {eps:.2}"),
            format!("E (edges) = {e_n}"),
            format!("T (tri)   = {t_n}"),
            format!("beta0    = {b0}  (colonies)"),
            format!("beta1    = {b1}  (circuits)"),
        ]
        .iter()
        .enumerate()
        {
            let col = match i {
                2 => "#ffb86c",
                3 => "#50fa7b",
                4 => "#ff79c6",
                _ => "#8be9fd",
            };
            ctx.set_fill_style_str(col);
            ctx.fill_text(line, w - 150.0, 35.0 + i as f64 * 12.0).ok();
        }
        ctx.set_text_align("left");
    }

    Some(())
}

// ─── Schema Config + Particles Data Model ────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn setup_schema_events() {
    // Load Schema button
    attach_click("schema-load-btn", || {
        let text = doc()
            .get_element_by_id("schema-input-ta")
            .and_then(|el| el.dyn_into::<HtmlTextAreaElement>().ok())
            .map(|ta| ta.value())
            .unwrap_or_default();
        let n = SCHEMA_STATE.with(|s| {
            s.borrow_mut().load_schema(&text);
            s.borrow().attrs.len()
        });
        render_schema_view();
        attach_schema_table_events(n);
        let _ = try_draw_particles_model_canvas();
    });

    // Export .dqprofile button
    attach_click("schema-export-btn", || {
        let profile = SCHEMA_STATE.with(|s| s.borrow_mut().export().to_string());
        if let Some(ta) = doc()
            .get_element_by_id("schema-output-ta")
            .and_then(|el| el.dyn_into::<HtmlTextAreaElement>().ok())
        {
            ta.set_value(&profile);
        }
    });

    // Format layer select
    let fmt_cb = Closure::wrap(Box::new(move |e: Event| {
        if let Some(sel) = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
        {
            let idx = sel.selected_index() as usize;
            SCHEMA_STATE.with(|s| s.borrow_mut().format_layer = idx.min(FORMAT_LAYERS.len() - 1));
        }
    }) as Box<dyn FnMut(Event)>);
    if let Some(el) = doc().get_element_by_id("schema-format") {
        el.add_event_listener_with_callback("change", fmt_cb.as_ref().unchecked_ref())
            .ok();
    }
    fmt_cb.forget();

    // Source trust select
    let trust_cb = Closure::wrap(Box::new(move |e: Event| {
        if let Some(sel) = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
        {
            let idx = sel.selected_index() as usize;
            SCHEMA_STATE
                .with(|s| s.borrow_mut().source_trust = idx.min(SOURCE_TRUST_LEVELS.len() - 1));
        }
    }) as Box<dyn FnMut(Event)>);
    if let Some(el) = doc().get_element_by_id("schema-trust") {
        el.add_event_listener_with_callback("change", trust_cb.as_ref().unchecked_ref())
            .ok();
    }
    trust_cb.forget();

    // Gem threshold input
    let gem_cb = Closure::wrap(Box::new(move |e: InputEvent| {
        if let Some(inp) = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        {
            let val = inp.value().parse::<u8>().unwrap_or(200);
            SCHEMA_STATE.with(|s| s.borrow_mut().gem_threshold = val);
        }
    }) as Box<dyn FnMut(InputEvent)>);
    if let Some(el) = doc().get_element_by_id("schema-gem") {
        el.add_event_listener_with_callback("input", gem_cb.as_ref().unchecked_ref())
            .ok();
    }
    gem_cb.forget();

    // Tribe threshold input
    let tribe_cb = Closure::wrap(Box::new(move |e: InputEvent| {
        if let Some(inp) = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        {
            let val = inp.value().parse::<u8>().unwrap_or(140);
            SCHEMA_STATE.with(|s| s.borrow_mut().tribe_threshold = val);
        }
    }) as Box<dyn FnMut(InputEvent)>);
    if let Some(el) = doc().get_element_by_id("schema-tribe") {
        el.add_event_listener_with_callback("input", tribe_cb.as_ref().unchecked_ref())
            .ok();
    }
    tribe_cb.forget();
}

#[cfg(target_arch = "wasm32")]
fn render_schema_view() {
    let html = SCHEMA_STATE.with(|s| schema_config::render_schema_table_html(&s.borrow()));
    set_html("schema-attr-table", &html);
}

#[cfg(target_arch = "wasm32")]
fn attach_schema_table_events(n: usize) {
    for i in 0..n {
        // Nullable toggle
        let cb = Closure::wrap(Box::new(move |_: Event| {
            SCHEMA_STATE.with(|s| s.borrow_mut().toggle_nullable(i));
            render_schema_view();
            attach_schema_table_events(SCHEMA_STATE.with(|s| s.borrow().attrs.len()));
        }) as Box<dyn FnMut(Event)>);
        if let Some(el) = doc().get_element_by_id(&format!("attr-null-{i}")) {
            el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
        }
        cb.forget();

        // Threshold slider
        let cb = Closure::wrap(Box::new(move |e: InputEvent| {
            if let Some(inp) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                let val = inp.value().parse::<u8>().unwrap_or(0);
                SCHEMA_STATE.with(|s| s.borrow_mut().set_threshold(i, val));
                if let Some(span) = doc().get_element_by_id(&format!("thresh-{i}")) {
                    span.set_text_content(Some(&format!("{val}%")));
                }
            }
        }) as Box<dyn FnMut(InputEvent)>);
        if let Some(el) = doc().get_element_by_id(&format!("attr-thresh-{i}")) {
            el.add_event_listener_with_callback("input", cb.as_ref().unchecked_ref())
                .ok();
        }
        cb.forget();

        // Dim cycle button
        let cb = Closure::wrap(Box::new(move |_: Event| {
            SCHEMA_STATE.with(|s| s.borrow_mut().cycle_dim(i));
            render_schema_view();
            attach_schema_table_events(SCHEMA_STATE.with(|s| s.borrow().attrs.len()));
        }) as Box<dyn FnMut(Event)>);
        if let Some(el) = doc().get_element_by_id(&format!("attr-dim-{i}")) {
            el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
        }
        cb.forget();

        // Weight slider
        let cb = Closure::wrap(Box::new(move |e: InputEvent| {
            if let Some(inp) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                let val = inp.value().parse::<u8>().unwrap_or(0);
                SCHEMA_STATE.with(|s| s.borrow_mut().set_weight(i, val));
                if let Some(span) = doc().get_element_by_id(&format!("weight-{i}")) {
                    span.set_text_content(Some(&format!("{val}%")));
                }
            }
        }) as Box<dyn FnMut(InputEvent)>);
        if let Some(el) = doc().get_element_by_id(&format!("attr-weight-{i}")) {
            el.add_event_listener_with_callback("input", cb.as_ref().unchecked_ref())
                .ok();
        }
        cb.forget();
    }
}

#[cfg(target_arch = "wasm32")]
fn try_draw_particles_model_canvas() -> Option<()> {
    let canvas = doc()
        .get_element_by_id("particles-model-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let ctx = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    schema_config::draw_particles_model(&ctx, w, h);
    Some(())
}

// ─── Live MDM Data — bahyway-api bridge ──────────────────────────────────────

#[cfg(target_arch = "wasm32")]
const LIVE_API_URL: &str = "http://localhost:8082/api/v1/live";

#[cfg(target_arch = "wasm32")]
const BATCHES_API_URL: &str = "http://localhost:8082/api/v1/batches";

#[cfg(target_arch = "wasm32")]
const TRIBES_API_URL: &str = "http://localhost:8082/api/v1/tribes";

#[cfg(target_arch = "wasm32")]
fn setup_live_data_events() {
    attach_click("btn-refresh-live", || fetch_live_stats());
    attach_click("btn-refresh-batches", || {
        fetch_batch_history();
        fetch_tribe_stats();
    });
}

#[cfg(target_arch = "wasm32")]
fn fetch_live_stats() {
    wasm_bindgen_futures::spawn_local(async {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        set_html(
            "dash-live-status",
            "<span style='color:var(--text-muted)'>&#8635; fetching…</span>",
        );

        let fetch_promise = match window.fetch_with_str(LIVE_API_URL) {
            Ok(p) => p,
            Err(_) => {
                set_html("dash-live-status",
                    "<span style='color:var(--red)'>&#9888; API unreachable — run bahyway-api</span>");
                return;
            }
        };

        let resp_val = match JsFuture::from(fetch_promise).await {
            Ok(v) => v,
            Err(_) => {
                set_html(
                    "dash-live-status",
                    "<span style='color:var(--red)'>&#9888; network error</span>",
                );
                return;
            }
        };

        let resp: web_sys::Response = match resp_val.dyn_into() {
            Ok(r) => r,
            Err(_) => return,
        };

        if !resp.ok() {
            set_html(
                "dash-live-status",
                &format!(
                    "<span style='color:var(--red)'>HTTP {}</span>",
                    resp.status()
                ),
            );
            return;
        }

        let text_promise = match resp.text() {
            Ok(p) => p,
            Err(_) => return,
        };

        let text_val = match JsFuture::from(text_promise).await {
            Ok(v) => v,
            Err(_) => return,
        };

        let text = text_val.as_string().unwrap_or_default();
        parse_and_render_live_stats(&text);
    });
}

#[cfg(target_arch = "wasm32")]
fn parse_and_render_live_stats(json: &str) {
    fn num(json: &str, key: &str) -> usize {
        let needle = format!("\"{}\":", key);
        json.find(&needle)
            .and_then(|pos| {
                let rest = &json[pos + needle.len()..];
                let end = rest
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(rest.len());
                rest[..end].trim().parse().ok()
            })
            .unwrap_or(0)
    }
    fn str_val(json: &str, key: &str) -> String {
        let needle = format!("\"{}\":\"", key);
        json.find(&needle)
            .map(|pos| {
                let rest = &json[pos + needle.len()..];
                let end = rest.find('"').unwrap_or(rest.len());
                rest[..end].to_string()
            })
            .unwrap_or_else(|| "—".to_string())
    }

    let status = str_val(json, "status");
    if status == "no_data" {
        let msg = str_val(json, "msg");
        set_html(
            "dash-live-stats",
            &format!("<p style='color:var(--text-muted);font-size:13px'>{msg}</p>"),
        );
        set_html(
            "dash-live-status",
            "<span style='color:var(--yellow)'>&#9888; no data yet</span>",
        );
        return;
    }

    let golden = num(json, "golden");
    let fuzzy = num(json, "fuzzy");
    let dead = num(json, "dead");
    let total = num(json, "total");
    let batches = num(json, "batches");
    let ts = num(json, "timestamp");
    let batch = str_val(json, "last_batch");

    let pct_gold = if total > 0 { golden * 100 / total } else { 0 };
    let pct_fuzzy = if total > 0 { fuzzy * 100 / total } else { 0 };

    set_html(
        "dash-live-stats",
        &format!(
            "<div class='live-stats-grid'>\
           <div class='live-stat-box' style='border-left-color:var(--accent)'>\
             <div class='live-stat-label'>Last Batch</div>\
             <div class='live-stat-val' style='color:var(--accent);font-size:15px'>{batch}</div>\
           </div>\
           <div class='live-stat-box' style='border-left-color:var(--text-muted)'>\
             <div class='live-stat-label'>Batches Processed</div>\
             <div class='live-stat-val'>{batches}</div>\
           </div>\
           <div class='live-stat-box' style='border-left-color:var(--text)'>\
             <div class='live-stat-label'>Total Records</div>\
             <div class='live-stat-val'>{total}</div>\
           </div>\
           <div class='live-stat-box' style='border-left-color:var(--green)'>\
             <div class='live-stat-label'>Golden</div>\
             <div class='live-stat-val' style='color:var(--green)'>{golden}</div>\
             <div class='live-stat-sub'>{pct_gold}% of total</div>\
           </div>\
           <div class='live-stat-box' style='border-left-color:var(--yellow)'>\
             <div class='live-stat-label'>Fuzzy</div>\
             <div class='live-stat-val' style='color:var(--yellow)'>{fuzzy}</div>\
             <div class='live-stat-sub'>{pct_fuzzy}% of total</div>\
           </div>\
           <div class='live-stat-box' style='border-left-color:var(--red)'>\
             <div class='live-stat-label'>Dead</div>\
             <div class='live-stat-val' style='color:var(--red)'>{dead}</div>\
           </div>\
         </div>",
        ),
    );
    set_html(
        "dash-live-status",
        &format!("<span style='color:var(--green)'>&#10003; live  ·  ts {ts}</span>"),
    );

    // Update the Notebook live banner with same data
    let summary = format!(
        "{total} records &mdash; \
         <span style='color:var(--green)'>{golden} Golden</span> &nbsp;\
         <span style='color:var(--yellow)'>{fuzzy} Fuzzy</span> &nbsp;\
         <span style='color:var(--red)'>{dead} Dead</span> &nbsp;\
         &middot; {batches} batches &middot; last: <strong>{batch}</strong>"
    );
    set_html("nb-live-summary", &summary);
    if let Some(el) = doc().get_element_by_id("nb-live-banner") {
        el.set_attribute(
            "style",
            "display:block;margin-bottom:12px;padding:8px 14px;\
            background:var(--surface);border:1px solid var(--border);\
            border-left:3px solid var(--green);border-radius:6px;\
            font-size:12px;color:var(--text-muted)",
        )
        .ok();
    }
}

// ─── Batch history fetch + render ─────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn fetch_batch_history() {
    wasm_bindgen_futures::spawn_local(async {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        set_html(
            "tribes-live-status",
            "<span style='color:var(--text-muted)'>&#8635; fetching…</span>",
        );

        let fetch_promise = match window.fetch_with_str(BATCHES_API_URL) {
            Ok(p) => p,
            Err(_) => {
                set_html(
                    "tribes-live-status",
                    "<span style='color:var(--red)'>&#9888; API unreachable</span>",
                );
                return;
            }
        };
        let resp_val = match JsFuture::from(fetch_promise).await {
            Ok(v) => v,
            Err(_) => {
                set_html(
                    "tribes-live-status",
                    "<span style='color:var(--red)'>&#9888; network error</span>",
                );
                return;
            }
        };
        let resp: web_sys::Response = match resp_val.dyn_into() {
            Ok(r) => r,
            Err(_) => return,
        };
        if !resp.ok() {
            set_html(
                "tribes-live-status",
                &format!(
                    "<span style='color:var(--red)'>HTTP {}</span>",
                    resp.status()
                ),
            );
            return;
        }
        let text_promise = match resp.text() {
            Ok(p) => p,
            Err(_) => return,
        };
        let text_val = match JsFuture::from(text_promise).await {
            Ok(v) => v,
            Err(_) => return,
        };
        let text = text_val.as_string().unwrap_or_default();
        parse_and_render_batch_history(&text);
    });
}

#[cfg(target_arch = "wasm32")]
fn parse_and_render_batch_history(json: &str) {
    // Minimal JSON helpers (no serde)
    fn num(json: &str, key: &str) -> usize {
        let needle = format!("\"{}\":", key);
        json.find(&needle)
            .and_then(|pos| {
                let rest = &json[pos + needle.len()..];
                let end = rest
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(rest.len());
                rest[..end].trim().parse().ok()
            })
            .unwrap_or(0)
    }
    fn str_val(json: &str, key: &str) -> String {
        let needle = format!("\"{}\":\"", key);
        json.find(&needle)
            .map(|pos| {
                let rest = &json[pos + needle.len()..];
                let end = rest.find('"').unwrap_or(rest.len());
                rest[..end].to_string()
            })
            .unwrap_or_else(|| "—".to_string())
    }

    let status = str_val(json, "status");
    if status == "no_data" {
        set_html(
            "tribes-batch-history",
            "<span style='color:var(--text-muted);font-size:13px'>\
             No batches processed yet — run bee-watchdog to process a ZIP batch.</span>",
        );
        set_html(
            "tribes-live-status",
            "<span style='color:var(--yellow)'>&#9888; no batches</span>",
        );
        return;
    }

    let tribe_id = str_val(json, "tribe_id");
    let count = num(json, "count");

    // Parse batches array: each element is a {...} object
    // Simple approach: split on "},{"
    let batches_start = match json.find("[{") {
        Some(p) => p + 1,
        None => {
            set_html(
                "tribes-batch-history",
                "<span style='color:var(--text-muted)'>Empty batch history.</span>",
            );
            set_html(
                "tribes-live-status",
                "<span style='color:var(--yellow)'>&#9888; 0 batches</span>",
            );
            return;
        }
    };
    let batches_end = json[batches_start..]
        .rfind('}')
        .map(|p| batches_start + p + 1)
        .unwrap_or(json.len());
    let batches_json = &json[batches_start..batches_end];

    // Split individual batch objects
    let mut batch_objs: Vec<&str> = Vec::new();
    let mut depth = 0i32;
    let mut obj_start = 0usize;
    for (i, ch) in batches_json.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    obj_start = i;
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    batch_objs.push(&batches_json[obj_start..=i]);
                }
            }
            _ => {}
        }
    }

    if batch_objs.is_empty() {
        set_html(
            "tribes-batch-history",
            "<span style='color:var(--text-muted)'>No batch records found.</span>",
        );
        set_html(
            "tribes-live-status",
            "<span style='color:var(--yellow)'>&#9888; 0 batches</span>",
        );
        return;
    }

    // Build table rows (newest first)
    let mut rows = String::from(
        "<table class='batch-hist-table'>\
         <thead><tr>\
           <th>#</th><th>Batch Name</th><th>Total</th>\
           <th>Golden</th><th>Fuzzy</th><th>Quality Bar</th>\
         </tr></thead><tbody>",
    );

    let n = batch_objs.len();
    for (idx, obj) in batch_objs.iter().enumerate().rev() {
        let name = str_val(obj, "name");
        let golden = num(obj, "golden");
        let fuzzy = num(obj, "fuzzy");
        let total = num(obj, "total");
        let batch_num = n - idx;
        let pct_g = if total > 0 { golden * 100 / total } else { 0 };
        let pct_f = if total > 0 { fuzzy * 100 / total } else { 0 };
        rows.push_str(&format!(
            "<tr>\
             <td style='color:var(--text-muted)'>{batch_num}</td>\
             <td style='color:var(--accent);font-weight:600'>{name}</td>\
             <td>{total}</td>\
             <td style='color:var(--green);font-weight:600'>{golden}</td>\
             <td style='color:var(--yellow)'>{fuzzy}</td>\
             <td><div class='bh-bar-wrap'>\
               <div class='bh-bar'>\
                 <div class='bh-gold-seg' style='width:{pct_g}%'></div>\
                 <div class='bh-fuzz-seg' style='width:{pct_f}%'></div>\
               </div>\
               <span style='font-size:10px;color:var(--text-muted)'>{pct_g}%</span>\
             </div></td>\
             </tr>"
        ));
    }
    rows.push_str("</tbody></table>");

    set_html("tribes-batch-history", &rows);
    set_html(
        "tribes-live-status",
        &format!(
        "<span style='color:var(--green)'>&#10003; {count} batches &middot; tribe {tribe_id}</span>"
    ),
    );
}

// ─── Tribe stats fetch + render ───────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn fetch_tribe_stats() {
    wasm_bindgen_futures::spawn_local(async {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        let fetch_promise = match window.fetch_with_str(TRIBES_API_URL) {
            Ok(p) => p,
            Err(_) => return,
        };
        let resp_val = match JsFuture::from(fetch_promise).await {
            Ok(v) => v,
            Err(_) => return,
        };
        let resp: web_sys::Response = match resp_val.dyn_into() {
            Ok(r) => r,
            Err(_) => return,
        };
        if !resp.ok() {
            return;
        }
        let text_promise = match resp.text() {
            Ok(p) => p,
            Err(_) => return,
        };
        let text_val = match JsFuture::from(text_promise).await {
            Ok(v) => v,
            Err(_) => return,
        };
        let text = text_val.as_string().unwrap_or_default();
        parse_and_render_tribe_stats(&text);
    });
}

#[cfg(target_arch = "wasm32")]
fn parse_and_render_tribe_stats(json: &str) {
    fn num(json: &str, key: &str) -> usize {
        let needle = format!("\"{}\":", key);
        json.find(&needle)
            .and_then(|pos| {
                let rest = &json[pos + needle.len()..];
                let end = rest
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(rest.len());
                rest[..end].trim().parse().ok()
            })
            .unwrap_or(0)
    }
    fn str_val(json: &str, key: &str) -> String {
        let needle = format!("\"{}\":\"", key);
        json.find(&needle)
            .map(|pos| {
                let rest = &json[pos + needle.len()..];
                let end = rest.find('"').unwrap_or(rest.len());
                rest[..end].to_string()
            })
            .unwrap_or_default()
    }

    let status = str_val(json, "status");
    if status == "no_data" || status.is_empty() {
        return;
    }

    // Find the first tribe object in the tribes array
    let tribe_obj_start = match json.find("[{") {
        Some(p) => p + 1,
        None => return,
    };
    let tribe_obj_end = json[tribe_obj_start..]
        .find('}')
        .map(|p| tribe_obj_start + p + 1)
        .unwrap_or(json.len());
    let tribe_json = &json[tribe_obj_start..tribe_obj_end];

    let tribe_id = str_val(tribe_json, "id");
    let last_batch = str_val(tribe_json, "last_batch");
    let golden = num(tribe_json, "golden");
    let fuzzy = num(tribe_json, "fuzzy");
    let dead = num(tribe_json, "dead");
    let total = num(tribe_json, "total");
    let batches = num(tribe_json, "batches");

    if tribe_id.is_empty() {
        return;
    }

    LIVE_TRIBE.with(|lt| {
        *lt.borrow_mut() = Some(LiveTribeStat {
            tribe_id,
            last_batch,
            golden,
            fuzzy,
            dead,
            total,
            batches,
        });
    });

    // Re-render tribes overview to apply live badge on matching card
    render_tribes_overview();
}
