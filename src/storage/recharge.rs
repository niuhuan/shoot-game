//! 充值系统
//! 处理充值相关的 HTTP 请求 + Web 端 HTML 覆盖层输入

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::GameState;

#[cfg(target_arch = "wasm32")]
use std::collections::VecDeque;
#[cfg(target_arch = "wasm32")]
use std::sync::{Mutex, OnceLock};

/// 充值插件
pub struct RechargePlugin;

impl Plugin for RechargePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RechargeState::default())
            .add_message::<RechargeEvent>()
            .add_message::<RechargeResultEvent>()
            .add_systems(OnEnter(GameState::Recharge), on_enter_recharge)
            .add_systems(OnExit(GameState::Recharge), on_exit_recharge)
            .add_systems(
                Update,
                (
                    process_recharge_events,
                    #[cfg(target_arch = "wasm32")]
                    drain_js_bridge_commands,
                    #[cfg(target_arch = "wasm32")]
                    drain_recharge_results,
                    handle_recharge_result,
                )
                    .run_if(in_state(GameState::Recharge)),
            );
    }
}

/// 充值状态
#[derive(Resource)]
pub struct RechargeState {
    pub username: String,
    pub order_id: String,
    pub is_processing: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub active_field: RechargeField,
}

impl Default for RechargeState {
    fn default() -> Self {
        Self {
            username: String::new(),
            order_id: String::new(),
            is_processing: false,
            error_message: None,
            success_message: None,
            active_field: RechargeField::Username,
        }
    }
}

/// 输入字段（原生 UI 使用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RechargeField {
    #[default]
    Username,
    OrderId,
}

/// 充值请求事件
#[derive(Message)]
pub struct RechargeEvent {
    pub username: String,
    pub order_id: String,
}

/// 充值结果事件
#[derive(Message, Clone)]
pub struct RechargeResultEvent {
    pub success: bool,
    pub message: String,
    pub coins_added: Option<u32>,
}

/// 充值请求数据
#[derive(Serialize)]
struct RechargeRequest {
    username: String,
    order_id: String,
    timestamp: u64,
}

/// 充值响应数据
#[derive(Deserialize)]
struct RechargeResponse {
    success: bool,
    message: String,
    coins: Option<u32>,
}

fn on_enter_recharge(mut state: ResMut<RechargeState>) {
    state.username.clear();
    state.order_id.clear();
    state.is_processing = false;
    state.error_message = None;
    state.success_message = None;
    state.active_field = RechargeField::Username;

    #[cfg(target_arch = "wasm32")]
    {
        show_input_overlay();
        set_recharge_message("", true);
    }
}

fn on_exit_recharge() {
    #[cfg(target_arch = "wasm32")]
    hide_input_overlay();
}

fn process_recharge_events(
    mut events: MessageReader<RechargeEvent>,
    mut state: ResMut<RechargeState>,
    mut results: MessageWriter<RechargeResultEvent>,
) {
    #[cfg(target_arch = "wasm32")]
    let _ = &mut results;

    for event in events.read() {
        if state.is_processing {
            continue;
        }

        if let Err(message) = validate_username(&event.username) {
            state.error_message = Some(message.clone());
            state.success_message = None;
            #[cfg(target_arch = "wasm32")]
            set_recharge_message(&message, true);
            continue;
        }
        if let Err(message) = validate_order_id(&event.order_id) {
            state.error_message = Some(message.clone());
            state.success_message = None;
            #[cfg(target_arch = "wasm32")]
            set_recharge_message(&message, true);
            continue;
        }

        state.username = event.username.clone();
        state.order_id = event.order_id.clone();
        state.is_processing = true;
        state.error_message = None;
        state.success_message = None;

        #[cfg(target_arch = "wasm32")]
        {
            set_recharge_message("正在提交…", false);
            send_recharge_request(
                event.username.clone(),
                event.order_id.clone(),
                "https://api.example.com/recharge",
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // 原生环境下模拟成功
            results.write(RechargeResultEvent {
                success: true,
                message: "充值成功！".to_string(),
                coins_added: Some(100),
            });
        }
    }
}

fn handle_recharge_result(
    mut result_events: MessageReader<RechargeResultEvent>,
    mut state: ResMut<RechargeState>,
    mut save_data: ResMut<super::SaveData>,
) {
    for event in result_events.read() {
        state.is_processing = false;

        if event.success {
            state.success_message = Some(event.message.clone());
            state.error_message = None;

            if let Some(coins) = event.coins_added {
                save_data.total_coins += coins;
                save_data.has_purchased = true;
            }
        } else {
            state.error_message = Some(event.message.clone());
            state.success_message = None;
        }

        #[cfg(target_arch = "wasm32")]
        set_recharge_message(&event.message, !event.success);
    }
}

fn validate_username(username: &str) -> Result<(), String> {
    if username.is_empty() {
        return Err("请输入用户名".to_string());
    }
    if username != username.trim() {
        return Err("用户名不能包含首尾空格".to_string());
    }
    let len = username.chars().count();
    if !(3..=20).contains(&len) {
        return Err("用户名长度需为 3-20 个字符".to_string());
    }
    let mut chars = username.chars();
    let Some(first) = chars.next() else {
        return Err("请输入用户名".to_string());
    };
    if !first.is_ascii_alphabetic() {
        return Err("用户名必须以字母开头".to_string());
    }
    for ch in chars {
        if !(ch.is_ascii_alphanumeric() || ch == '_') {
            return Err("用户名只能包含字母、数字和下划线".to_string());
        }
    }
    Ok(())
}

fn validate_order_id(order_id: &str) -> Result<(), String> {
    if order_id.is_empty() {
        return Err("请输入订单号".to_string());
    }
    if order_id != order_id.trim() {
        return Err("订单号不能包含首尾空格".to_string());
    }
    let len = order_id.chars().count();
    if len > 64 {
        return Err("订单号过长（最多 64 字符）".to_string());
    }
    for ch in order_id.chars() {
        if !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') {
            return Err("订单号只能包含字母、数字、- 和 _".to_string());
        }
    }
    Ok(())
}

// ---- WASM：异步 HTTP ----

/// 发送充值请求 (WASM)
#[cfg(target_arch = "wasm32")]
pub fn send_recharge_request(username: String, order_id: String, api_url: &str) {
    use wasm_bindgen_futures::spawn_local;

    let url = api_url.to_string();
    spawn_local(async move {
        match perform_recharge_request(&username, &order_id, &url).await {
            Ok(response) => enqueue_recharge_result(RechargeResultEvent {
                success: response.success,
                message: response.message,
                coins_added: response.coins,
            }),
            Err(e) => enqueue_recharge_result(RechargeResultEvent {
                success: false,
                message: e,
                coins_added: None,
            }),
        }
    });
}

#[cfg(target_arch = "wasm32")]
async fn perform_recharge_request(
    username: &str,
    order_id: &str,
    url: &str,
) -> Result<RechargeResponse, String> {
    use wasm_bindgen::JsCast;
    use web_sys::{Request, RequestInit, RequestMode, Response};

    let request_data = RechargeRequest {
        username: username.to_string(),
        order_id: order_id.to_string(),
        timestamp: js_sys::Date::now() as u64,
    };

    let body =
        serde_json::to_string(&request_data).map_err(|e| format!("Serialize error: {e}"))?;

    let mut opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body));

    let request =
        Request::new_with_str_and_init(url, &opts).map_err(|_| "Failed to create request")?;

    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|_| "Failed to set header")?;

    let window = web_sys::window().ok_or("No window")?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: Response = resp_value.dyn_into().map_err(|_| "Response cast failed")?;
    let text_value = wasm_bindgen_futures::JsFuture::from(resp.text().map_err(|_| "Text failed")?)
        .await
        .map_err(|_| "Text await failed")?;

    let text = text_value.as_string().ok_or("Response is not a string")?;
    serde_json::from_str(&text).map_err(|e| format!("Deserialize error: {e}"))
}

/// 发送充值请求 (Native - 模拟)
#[cfg(not(target_arch = "wasm32"))]
pub fn send_recharge_request(username: String, order_id: String, _api_url: &str) {
    log::info!(
        "Native recharge request (simulated): username={username}, order_id={order_id}"
    );
}

// ---- WASM：JS 桥接：submit/cancel & async result 回传 ----

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub enum JsRechargeCommand {
    Submit {
        username: String,
        order_id: String,
    },
    Cancel,
}

#[cfg(target_arch = "wasm32")]
static JS_RECHARGE_COMMANDS: OnceLock<Mutex<VecDeque<JsRechargeCommand>>> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
static JS_RECHARGE_RESULTS: OnceLock<Mutex<VecDeque<RechargeResultEvent>>> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
fn commands_queue() -> &'static Mutex<VecDeque<JsRechargeCommand>> {
    JS_RECHARGE_COMMANDS.get_or_init(|| Mutex::new(VecDeque::new()))
}

#[cfg(target_arch = "wasm32")]
fn results_queue() -> &'static Mutex<VecDeque<RechargeResultEvent>> {
    JS_RECHARGE_RESULTS.get_or_init(|| Mutex::new(VecDeque::new()))
}

#[cfg(target_arch = "wasm32")]
pub fn enqueue_recharge_submit(username: String, order_id: String) {
    let mut q = commands_queue().lock().expect("js recharge queue lock poisoned");
    q.push_back(JsRechargeCommand::Submit {
        username,
        order_id,
    });
}

#[cfg(target_arch = "wasm32")]
pub fn enqueue_recharge_cancel() {
    let mut q = commands_queue().lock().expect("js recharge queue lock poisoned");
    q.push_back(JsRechargeCommand::Cancel);
}

#[cfg(target_arch = "wasm32")]
fn enqueue_recharge_result(result: RechargeResultEvent) {
    let mut q = results_queue().lock().expect("js recharge result queue lock poisoned");
    q.push_back(result);
}

#[cfg(target_arch = "wasm32")]
fn drain_js_bridge_commands(
    mut next_state: ResMut<NextState<GameState>>,
    mut state: ResMut<RechargeState>,
    mut events: MessageWriter<RechargeEvent>,
) {
    let mut q = commands_queue().lock().expect("js recharge queue lock poisoned");
    while let Some(cmd) = q.pop_front() {
        match cmd {
            JsRechargeCommand::Cancel => {
                // 返回菜单：Menu UI 会在 OnEnter(Menu) 重建；并由 OnExit(Recharge) 关闭覆盖层
                next_state.set(GameState::Menu);
            }
            JsRechargeCommand::Submit {
                username,
                order_id,
            } => {
                state.username = username.clone();
                state.order_id = order_id.clone();
                events.write(RechargeEvent {
                    username,
                    order_id,
                });
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn drain_recharge_results(mut writer: MessageWriter<RechargeResultEvent>) {
    let mut q = results_queue().lock().expect("js recharge result queue lock poisoned");
    while let Some(result) = q.pop_front() {
        writer.write(result);
    }
}

// ---- WASM：HTML 覆盖层 ----

/// 显示 HTML 输入覆盖层 (WASM)
#[cfg(target_arch = "wasm32")]
fn show_input_overlay() {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };

    if document.get_element_by_id("recharge-overlay").is_some() {
        if let Some(overlay) = document.get_element_by_id("recharge-overlay") {
            let _ = overlay.set_attribute("style", "display: flex;");
        }
        return;
    }

    let overlay_html = r#"
        <div id="recharge-overlay" style="
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.8);
            display: flex;
            justify-content: center;
            align-items: center;
            z-index: 1000;
        ">
            <div style="
                background: #1a1a2e;
                padding: 30px;
                border-radius: 10px;
                border: 2px solid #00d4ff;
                text-align: center;
                max-width: 420px;
                width: 90%;
            ">
                <h2 style="color: #00d4ff; margin-bottom: 20px;">充值中心</h2>
                <p style="color: #fff; margin-bottom: 10px;">请填写用户名与订单号</p>
                <p style="color: #aaa; margin-bottom: 15px; font-size: 12px; line-height: 1.5;">
                    提示：用户名 3-20 位、字母开头、仅允许字母/数字/下划线；订单号仅允许字母/数字/-/_，最多 64 位。
                </p>
                <input type="text" id="recharge-username" maxlength="20" style="
                    width: 100%;
                    padding: 10px;
                    font-size: 16px;
                    border: 1px solid #00d4ff;
                    border-radius: 5px;
                    background: #0a0a1a;
                    color: #fff;
                    margin-bottom: 10px;
                    box-sizing: border-box;
                " placeholder="用户名（3-20，字母开头，仅字母/数字/_）">
                <input type="text" id="recharge-order" maxlength="64" style="
                    width: 100%;
                    padding: 10px;
                    font-size: 16px;
                    border: 1px solid #00d4ff;
                    border-radius: 5px;
                    background: #0a0a1a;
                    color: #fff;
                    margin-bottom: 10px;
                    box-sizing: border-box;
                " placeholder="订单号（最多64，字母/数字/-/_）">
                <div style="display: flex; gap: 10px; justify-content: center;">
                    <button id="recharge-submit" style="
                        padding: 10px 30px;
                        font-size: 16px;
                        background: #00d4ff;
                        border: none;
                        border-radius: 5px;
                        cursor: pointer;
                        color: #000;
                    ">确认</button>
                    <button id="recharge-cancel" style="
                        padding: 10px 30px;
                        font-size: 16px;
                        background: #333;
                        border: 1px solid #666;
                        border-radius: 5px;
                        cursor: pointer;
                        color: #fff;
                    ">取消</button>
                </div>
                <p id="recharge-message" style="color: #ff6b6b; margin-top: 15px; min-height: 20px;"></p>
            </div>
        </div>
    "#;

    if let Ok(div) = document.create_element("div") {
        div.set_inner_html(overlay_html);
        if let Some(body) = document.body() {
            let _ = body.append_child(&div);
        }
    }

    // 让 web/index.html 里注册的监听器生效
    let _ = js_sys::eval("if(window.setupRechargeListeners){window.setupRechargeListeners();}");
}

/// 隐藏 HTML 输入覆盖层 (WASM)
#[cfg(target_arch = "wasm32")]
fn hide_input_overlay() {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    if let Some(overlay) = document.get_element_by_id("recharge-overlay") {
        let _ = overlay.set_attribute("style", "display: none;");
    }
}

/// 设置消息 (WASM)
#[cfg(target_arch = "wasm32")]
pub fn set_recharge_message(message: &str, is_error: bool) {
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    let Some(elem) = document.get_element_by_id("recharge-message") else { return };
    let Some(html) = elem.dyn_ref::<web_sys::HtmlElement>() else { return };

    html.set_inner_text(message);
    let color = if is_error { "#ff6b6b" } else { "#4ade80" };
    let _ = html.style().set_property("color", color);
}
