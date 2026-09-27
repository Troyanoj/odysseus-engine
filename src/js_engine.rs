use deno_core::{JsRuntime, RuntimeOptions, v8};
use deno_core::anyhow;

pub async fn eval_js(code: &str) -> Result<String, anyhow::Error> {
    let mut js_runtime = JsRuntime::new(RuntimeOptions::default());
    
    let result = js_runtime.execute_script("<anon>", code.to_string())?;
    
    let scope = &mut js_runtime.handle_scope();
    let local = v8::Local::new(scope, result);
    
    Ok(local.to_rust_string_lossy(scope))
}

pub async fn render_spa(html: &str) -> Result<String, anyhow::Error> {
    let mut js_runtime = JsRuntime::new(RuntimeOptions::default());
    
    let safe_html = html
        .replace("\\", "\\\\")
        .replace("`", "\\`")
        .replace("${", "\\${");
    
    let js_code = format!(r#"
        globalThis.window = globalThis;
        globalThis.document = {{
            body: {{ innerHTML: "" }},
            createElement: () => ({{ innerHTML: "", textContent: "", style: {{}}, appendChild: () => ({{}}), setAttribute: () => ({{}}) }}),
            createTextNode: (t) => ({{ textContent: t }}),
            getElementById: () => null,
            querySelector: () => null
        }};
        document.body.innerHTML = `{}`;
        // Simulación: Una SPA inyecta contenido dinámico tras cargar
        document.body.innerHTML += "<h2>✨ Contenido renderizado dinámicamente por V8 ✨</h2>";
    "#, safe_html);
    
    js_runtime.execute_script("<anon>", js_code)?;
    
    let result = js_runtime.execute_script("<anon>", "document.body.innerHTML")?;
    
    let scope = &mut js_runtime.handle_scope();
    let local = v8::Local::new(scope, result);
    
    Ok(local.to_rust_string_lossy(scope))
}