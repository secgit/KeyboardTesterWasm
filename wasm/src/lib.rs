use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    Document, DocumentFragment, HtmlButtonElement, HtmlElement, HtmlInputElement,
    HtmlTableSectionElement, HtmlUListElement, KeyboardEvent, MouseEvent, Node, Performance,
    Window,
};

const MAX_LOG_ROWS: usize = 300;
const MAX_PATTERN_LENGTH: usize = 80;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let app = App::init()?;
    APP_INSTANCE.with(|slot| {
        *slot.borrow_mut() = Some(app);
    });
    Ok(())
}

thread_local! {
    static APP_INSTANCE: RefCell<Option<App>> = RefCell::new(None);
}

struct App {
    #[allow(dead_code)]
    state: Rc<RefCell<AppState>>,
    _keydown: Closure<dyn FnMut(KeyboardEvent)>,
    _keyup: Closure<dyn FnMut(KeyboardEvent)>,
    _clear: Closure<dyn FnMut(MouseEvent)>,
    _body_click: Closure<dyn FnMut(MouseEvent)>,
}

struct AppState {
    document: Document,
    performance: Performance,
    body: HtmlElement,
    held_keys_list: HtmlUListElement,
    no_held_keys_label: HtmlElement,
    event_log_body: HtmlTableSectionElement,
    pattern_sequence_el: HtmlElement,
    pattern_counts_el: HtmlUListElement,
    pause_toggle: HtmlInputElement,
    clear_button: HtmlButtonElement,
    active_keys: HashMap<String, KeyInfo>,
    repeat_counts: HashMap<String, RepeatData>,
    pattern_buffer: Vec<String>,
    last_event_time: Option<f64>,
    origin_time: f64,
}

#[derive(Clone)]
struct KeyInfo {
    key: String,
    code: String,
    pressed_at: f64,
}

struct RepeatData {
    key: String,
    count: u32,
}

impl App {
    fn init() -> Result<Self, JsValue> {
        let window: Window =
            web_sys::window().ok_or_else(|| JsValue::from_str("missing window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("missing document"))?;
        let performance = window
            .performance()
            .ok_or_else(|| JsValue::from_str("missing performance"))?;
        let body = document
            .body()
            .ok_or_else(|| JsValue::from_str("missing body"))?;

        let held_keys_list = document
            .get_element_by_id("held-keys")
            .ok_or_else(|| JsValue::from_str("missing held-keys element"))?
            .dyn_into::<HtmlUListElement>()?;
        let no_held_keys_label = document
            .get_element_by_id("no-held-keys")
            .ok_or_else(|| JsValue::from_str("missing no-held-keys element"))?
            .dyn_into::<HtmlElement>()?;
        let event_log_body = document
            .get_element_by_id("event-log")
            .ok_or_else(|| JsValue::from_str("missing event-log element"))?
            .dyn_into::<HtmlTableSectionElement>()?;
        let pattern_sequence_el = document
            .get_element_by_id("pattern-sequence")
            .ok_or_else(|| JsValue::from_str("missing pattern-sequence element"))?
            .dyn_into::<HtmlElement>()?;
        let pattern_counts_el = document
            .get_element_by_id("pattern-counts")
            .ok_or_else(|| JsValue::from_str("missing pattern-counts element"))?
            .dyn_into::<HtmlUListElement>()?;
        let pause_toggle = document
            .get_element_by_id("toggle-pause")
            .ok_or_else(|| JsValue::from_str("missing toggle-pause element"))?
            .dyn_into::<HtmlInputElement>()?;
        let clear_button = document
            .get_element_by_id("clear-log")
            .ok_or_else(|| JsValue::from_str("missing clear-log element"))?
            .dyn_into::<HtmlButtonElement>()?;

        let origin_time = performance.now();

        let state = Rc::new(RefCell::new(AppState {
            document: document.clone(),
            performance,
            body: body.clone(),
            held_keys_list,
            no_held_keys_label,
            event_log_body,
            pattern_sequence_el,
            pattern_counts_el,
            pause_toggle,
            clear_button,
            active_keys: HashMap::new(),
            repeat_counts: HashMap::new(),
            pattern_buffer: Vec::new(),
            last_event_time: None,
            origin_time,
        }));

        {
            let state_ref = state.borrow();
            state_ref.body.set_attribute("tabindex", "0")?;
            state_ref.body.focus()?;
            render_held_keys(&state_ref);
            render_pattern_sequence(&state_ref);
            render_pattern_counts(&state_ref);
        }

        let keydown_state = state.clone();
        let keydown = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            handle_keydown(&keydown_state, event);
        }) as Box<dyn FnMut(_)>);

        document.add_event_listener_with_callback_and_bool(
            "keydown",
            keydown.as_ref().unchecked_ref(),
            true,
        )?;

        let keyup_state = state.clone();
        let keyup = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            handle_keyup(&keyup_state, event);
        }) as Box<dyn FnMut(_)>);

        document.add_event_listener_with_callback_and_bool(
            "keyup",
            keyup.as_ref().unchecked_ref(),
            true,
        )?;

        let clear_state = state.clone();
        let clear = Closure::wrap(Box::new(move |_event: MouseEvent| {
            if let Err(err) = reset_all(&clear_state, true) {
                web_sys::console::error_1(&err);
            }
        }) as Box<dyn FnMut(_)>);

        {
            let clear_button = state.borrow().clear_button.clone();
            clear_button
                .add_event_listener_with_callback("click", clear.as_ref().unchecked_ref())?;
        }

        let body_for_click = state.borrow().body.clone();
        let body_click = Closure::wrap(Box::new(move |_event: MouseEvent| {
            let _ = body_for_click.focus();
        }) as Box<dyn FnMut(_)>);

        body.add_event_listener_with_callback("click", body_click.as_ref().unchecked_ref())?;

        Ok(App {
            state,
            _keydown: keydown,
            _keyup: keyup,
            _clear: clear,
            _body_click: body_click,
        })
    }
}

fn handle_keydown(state: &Rc<RefCell<AppState>>, event: KeyboardEvent) {
    if state.borrow().pause_toggle.checked() {
        return;
    }

    let key = event.key();
    let code = event.code();
    let repeat = event.repeat();
    let time_stamp = event.time_stamp();

    {
        let mut state_mut = state.borrow_mut();
        if !state_mut.active_keys.contains_key(&code) {
            state_mut.active_keys.insert(
                code.clone(),
                KeyInfo {
                    key: key.clone(),
                    code: code.clone(),
                    pressed_at: time_stamp,
                },
            );
            render_held_keys(&state_mut);
        }

        append_log_row(&state_mut, "keydown", &key, &code, repeat, time_stamp);
        state_mut.last_event_time = Some(time_stamp);

        if repeat {
            let label = if key.chars().count() == 1 {
                key.clone()
            } else {
                code.clone()
            };
            state_mut.pattern_buffer.push(label);
            if state_mut.pattern_buffer.len() > MAX_PATTERN_LENGTH {
                state_mut.pattern_buffer.remove(0);
            }

            state_mut
                .repeat_counts
                .entry(code.clone())
                .and_modify(|data| data.count += 1)
                .or_insert(RepeatData {
                    key: key.clone(),
                    count: 1,
                });

            render_pattern_sequence(&state_mut);
            render_pattern_counts(&state_mut);
        }
    }
}

fn handle_keyup(state: &Rc<RefCell<AppState>>, event: KeyboardEvent) {
    if state.borrow().pause_toggle.checked() {
        return;
    }

    let key = event.key();
    let code = event.code();
    let time_stamp = event.time_stamp();

    {
        let mut state_mut = state.borrow_mut();
        append_log_row(&state_mut, "keyup", &key, &code, false, time_stamp);
        state_mut.last_event_time = Some(time_stamp);

        if state_mut.active_keys.remove(&code).is_some() {
            render_held_keys(&state_mut);
        }
    }
}

fn reset_all(state: &Rc<RefCell<AppState>>, clear_pause: bool) -> Result<(), JsValue> {
    let performance_now = {
        let state_ref = state.borrow();
        state_ref.performance.now()
    };

    let mut state_mut = state.borrow_mut();
    state_mut.active_keys.clear();
    state_mut.repeat_counts.clear();
    state_mut.pattern_buffer.clear();
    state_mut.event_log_body.set_inner_html("");
    state_mut.last_event_time = None;
    state_mut.origin_time = performance_now;

    if clear_pause {
        state_mut.pause_toggle.set_checked(false);
    }

    render_held_keys(&state_mut);
    render_pattern_sequence(&state_mut);
    render_pattern_counts(&state_mut);

    Ok(())
}

fn render_held_keys(state: &AppState) {
    state.held_keys_list.set_inner_html("");
    if state.active_keys.is_empty() {
        state.no_held_keys_label.set_hidden(false);
        return;
    }

    state.no_held_keys_label.set_hidden(true);

    let mut values: Vec<_> = state.active_keys.values().cloned().collect();
    values.sort_by(|a, b| match a.pressed_at.partial_cmp(&b.pressed_at) {
        Some(Ordering::Less) => Ordering::Less,
        Some(Ordering::Greater) => Ordering::Greater,
        _ => Ordering::Equal,
    });

    let fragment: DocumentFragment = state.document.create_document_fragment();

    for info in values {
        if let Ok(li) = state.document.create_element("li") {
            if let Ok(key_span) = state.document.create_element("span") {
                key_span.set_class_name("pill-key");
                key_span.set_text_content(Some(&info.key));
                let _ = li.append_child(&key_span);
            }
            if let Ok(meta_span) = state.document.create_element("span") {
                meta_span.set_class_name("pill-meta");
                meta_span.set_text_content(Some(&info.code));
                let _ = li.append_child(&meta_span);
            }
            let _ = fragment.append_child(&li);
        }
    }

    let _ = state.held_keys_list.append_child(&fragment);
}

fn render_pattern_sequence(state: &AppState) {
    if state.pattern_buffer.is_empty() {
        state
            .pattern_sequence_el
            .set_text_content(Some("Waiting for repeated events..."));
        return;
    }

    let sequence = state.pattern_buffer.join(" → ");
    state.pattern_sequence_el.set_text_content(Some(&sequence));
}

fn render_pattern_counts(state: &AppState) {
    state.pattern_counts_el.set_inner_html("");
    if state.repeat_counts.is_empty() {
        return;
    }

    let mut entries: Vec<_> = state
        .repeat_counts
        .iter()
        .map(|(code, data)| (code.clone(), data.key.clone(), data.count))
        .collect();

    entries.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

    let fragment = state.document.create_document_fragment();

    for (code, key, count) in entries {
        if let Ok(li) = state.document.create_element("li") {
            if let Ok(key_span) = state.document.create_element("span") {
                key_span.set_class_name("pill-key");
                key_span.set_text_content(Some(&key));
                let _ = li.append_child(&key_span);
            }
            if let Ok(meta_span) = state.document.create_element("span") {
                meta_span.set_class_name("pill-meta");
                meta_span.set_text_content(Some(&format!("{} ×{}", code, count)));
                let _ = li.append_child(&meta_span);
            }
            let _ = fragment.append_child(&li);
        }
    }

    let _ = state.pattern_counts_el.append_child(&fragment);
}

fn append_log_row(
    state: &AppState,
    event_type: &str,
    key: &str,
    code: &str,
    repeat: bool,
    time_stamp: f64,
) {
    if let Ok(row) = state.document.create_element("tr") {
        let delta_display = format_delta(state, time_stamp);
        let repeat_text = if repeat { "yes" } else { "no" };
        let html = format!(
            "<td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>",
            format_seconds(state, time_stamp),
            delta_display,
            escape_html(event_type),
            escape_html(key),
            escape_html(code),
            repeat_text,
        );
        row.set_inner_html(&html);

        let node: &Node = row.as_ref();
        let _ = state
            .event_log_body
            .insert_before(node, state.event_log_body.first_child().as_ref());

        trim_log_rows(&state.event_log_body);
    }
}

fn trim_log_rows(body: &HtmlTableSectionElement) {
    while body.child_element_count() as usize > MAX_LOG_ROWS {
        if let Some(last) = body.last_child() {
            let _ = body.remove_child(&last);
        } else {
            break;
        }
    }
}

fn format_seconds(state: &AppState, time_stamp: f64) -> String {
    format!("{:.3}", (time_stamp - state.origin_time) / 1000.0)
}

fn format_delta(state: &AppState, time_stamp: f64) -> String {
    match state.last_event_time {
        Some(last) => format!("{} ms", (time_stamp - last).round() as i64),
        None => "—".to_string(),
    }
}

fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
