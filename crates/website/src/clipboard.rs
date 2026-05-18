use js_sys::Function;

pub async fn copy_to_clipboard(text: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let clipboard = window.navigator().clipboard();
    let clipboard_is_available = {
        let js_value: &wasm_bindgen::JsValue = clipboard.as_ref();
        !js_value.is_undefined()
    };
    if clipboard_is_available {
        let promise = clipboard.write_text(text);
        return wasm_bindgen_futures::JsFuture::from(promise).await.is_ok();
    }
    copy_with_exec_command(text)
}

fn copy_with_exec_command(text: &str) -> bool {
    let function = Function::new_with_args(
        "text",
        "try {
            var ta = document.createElement('textarea');
            ta.value = text;
            ta.style.position = 'fixed';
            ta.style.left = '-9999px';
            document.body.appendChild(ta);
            ta.select();
            var ok = document.execCommand('copy');
            document.body.removeChild(ta);
            return ok;
        } catch(e) { return false; }",
    );
    function
        .call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::from_str(text))
        .map(|result| result.is_truthy())
        .unwrap_or(false)
}
