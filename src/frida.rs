use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::core;

pub fn generate_hook(target: &str, hook_type: &str, output: Option<&Path>) -> Result<()> {
    let script = match hook_type {
        "ssl" => ssl_pinning_bypass(),
        "crypto" => crypto_hook(target),
        "http" => http_hook(),
        "websocket" => websocket_hook(),
        "storage" => storage_hook(),
        "native" => native_hook(target),
        "class" => class_hook(target),
        _ => general_hook(target),
    };

    match output {
        Some(o) => {
            fs::write(o, &script)?;
            core::success(&format!("Frida 脚本已保存到 {}", o.display()));
        }
        None => print!("{}", script),
    }
    Ok(())
}

pub fn attach(process: &str, script_path: &str) -> Result<()> {
    core::info(&format!("准备附加到进程: {}", process));

    let script = if std::path::Path::new(script_path).exists() {
        fs::read_to_string(script_path)?
    } else {
        return Err(anyhow!("脚本文件不存在: {}", script_path));
    };

    core::info(&format!("脚本大小: {} 字节", script.len()));
    core::info("请使用以下命令启动 Frida:");
    println!();
    println!("  frida -U -n \"{}\" -l {}", process, script_path);
    println!();
    println!("  或者:");
    println!("  frida -U -p <PID> -l {}", script_path);
    println!();

    Ok(())
}

pub fn list_processes() -> Result<()> {
    core::info("正在列出设备进程...");
    println!();
    println!("请使用以下命令查看进程:");
    println!("  frida-ps -U          # USB 设备");
    println!("  frida-ps -R          # 远程设备");
    println!("  frida-ps -D <device> # 指定设备");
    Ok(())
}

// ========== Frida 脚本模板 ==========

fn ssl_pinning_bypass() -> String {
    r#"// Reptool Frida - SSL Pinning Bypass
// 用法: frida -U -n "TargetApp" -l ssl_bypass.js

Java.perform(function() {
    console.log("[Reptool] SSL Pinning Bypass 已加载");

    // Android SSLTrustManager bypass
    var TrustManagerImpl = Java.use("com.android.org.conscrypt.TrustManagerImpl");
    try {
        TrustManagerImpl.verifyChain.implementation = function(untrustedChain, trustAnchorChain, host, clientAuth, ocspData, tlsSctData) {
            console.log("[Reptool] Bypassing TrustManagerImpl.verifyChain for: " + host);
            return untrustedChain;
        };
    } catch(e) {
        console.log("[Reptool] TrustManagerImpl not found, trying other methods...");
    }

    // OkHttp3 CertificatePinner bypass
    try {
        var CertificatePinner = Java.use("okhttp3.CertificatePinner");
        CertificatePinner.check.overload('java.lang.String', 'java.util.List').implementation = function(hostname, peerCertificates) {
            console.log("[Reptool] Bypassing OkHttp3 CertificatePinner for: " + hostname);
        };
    } catch(e) {}

    // WebViewClient SSL error bypass
    try {
        var WebViewClient = Java.use("android.webkit.WebViewClient");
        WebViewClient.onReceivedSslError.implementation = function(view, handler, error) {
            console.log("[Reptool] Bypassing WebView SSL error");
            handler.proceed();
        };
    } catch(e) {}

    console.log("[Reptool] SSL Bypass 完成");
});
"#.to_string()
}

fn crypto_hook(_target: &str) -> String {
    format!(r#"// Reptool Frida - Crypto Hook
// Hook 加密函数: TARGET

Java.perform(function() {{
    console.log("[Reptool] Crypto Hook 已加载");

    // Hook javax.crypto.Cipher
    var Cipher = Java.use("javax.crypto.Cipher");
    Cipher.doFinal.overload('[B').implementation = function(input) {{
        var mode = this.getAlgorithm();
        console.log("[Reptool] Cipher.doFinal algorithm: " + mode);
        console.log("[Reptool] Input (hex): " + bytesToHex(input));
        var result = this.doFinal(input);
        console.log("[Reptool] Output (hex): " + bytesToHex(result));
        return result;
    }};

    // Hook MessageDigest
    var MessageDigest = Java.use("java.security.MessageDigest");
    MessageDigest.digest.overload('[B').implementation = function(input) {{
        var algo = this.getAlgorithm();
        console.log("[Reptool] MessageDigest algorithm: " + algo);
        console.log("[Reptool] Input: " + bytesToHex(input));
        var result = this.digest(input);
        console.log("[Reptool] Hash: " + bytesToHex(result));
        return result;
    }};

    // Hook Mac
    var Mac = Java.use("javax.crypto.Mac");
    Mac.doFinal.overload('[B').implementation = function(input) {{
        var algo = this.getAlgorithm();
        console.log("[Reptool] Mac algorithm: " + algo);
        console.log("[Reptool] Input: " + bytesToHex(input));
        var result = this.doFinal(input);
        console.log("[Reptool] HMAC: " + bytesToHex(result));
        return result;
    }};

    function bytesToHex(bytes) {{
        var hex = [];
        for (var i = 0; i < bytes.length; i++) {{
            var b = (bytes[i] & 0xFF).toString(16);
            hex.push(b.length == 1 ? '0' + b : b);
        }}
        return hex.join('');
    }}

    console.log("[Reptool] Crypto Hook 完成");
}});
"#)
}

fn http_hook() -> String {
    r#"// Reptool Frida - HTTP Hook
// Hook OkHttp3 / HttpURLConnection / Retrofit

Java.perform(function() {
    console.log("[Reptool] HTTP Hook 已加载");

    // Hook OkHttp3
    try {
        var OkHttpClient = Java.use("okhttp3.OkHttpClient");
        var RealCall = Java.use("okhttp3.RealCall");
        RealCall.execute.implementation = function() {
            var req = this.request();
            console.log("[Reptool] OkHttp3 " + req.method() + " " + req.url());
            req.headers().names().forEach(function(name) {
                console.log("  " + name + ": " + req.header(name));
            });
            if (req.body()) {
                var Buffer = Java.use("okio.Buffer");
                var buf = Buffer.$new();
                req.body().writeTo(buf);
                console.log("  Body: " + buf.readUtf8());
            }
            return this.execute();
        };
    } catch(e) {}

    // Hook HttpURLConnection
    try {
        var HttpURLConnection = Java.use("java.net.HttpURLConnection");
        HttpURLConnection.getOutputStream.implementation = function() {
            console.log("[Reptool] HttpURLConnection " + this.getRequestMethod() + " " + this.getURL());
            return this.getOutputStream();
        };
    } catch(e) {}

    console.log("[Reptool] HTTP Hook 完成");
});
"#.to_string()
}

fn websocket_hook() -> String {
    r#"// Reptool Frida - WebSocket Hook
// Hook WebSocket 消息收发

Java.perform(function() {
    console.log("[Reptool] WebSocket Hook 已加载");

    try {
        var WebSocket = Java.use("okhttp3.WebSocket$Listener");
        // Hook onMessage
        console.log("[Reptool] WebSocket hook 已注入");
    } catch(e) {
        console.log("[Reptool] WebSocket hook 失败: " + e);
    }

    // Hook JavaScriptInterface (WebView JS Bridge)
    try {
        var WebView = Java.use("android.webkit.WebView");
        WebView.loadUrl.overload('java.lang.String').implementation = function(url) {
            console.log("[Reptool] WebView.loadUrl: " + url);
            return this.loadUrl(url);
        };
    } catch(e) {}

    console.log("[Reptool] WebSocket Hook 完成");
});
"#.to_string()
}

fn storage_hook() -> String {
    r#"// Reptool Frida - Storage Hook
// Hook SharedPreferences / SQLite / 文件读写

Java.perform(function() {
    console.log("[Reptool] Storage Hook 已加载");

    // SharedPreferences
    try {
        var SharedPreferencesImpl = Java.use("android.app.SharedPreferencesImpl");
        SharedPreferencesImpl.getString.implementation = function(key, defValue) {
            var value = this.getString(key, defValue);
            console.log("[Reptool] SP.getString(\"" + key + "\") = \"" + value + "\"");
            return value;
        };
        SharedPreferencesImpl.putString.implementation = function(key, value) {
            console.log("[Reptool] SP.putString(\"" + key + "\", \"" + value + "\")");
            return this.putString(key, value);
        };
    } catch(e) {}

    // SQLite
    try {
        var SQLiteDatabase = Java.use("android.database.sqlite.SQLiteDatabase");
        SQLiteDatabase.rawQuery.implementation = function(sql, selectionArgs) {
            console.log("[Reptool] SQL: " + sql);
            return this.rawQuery(sql, selectionArgs);
        };
    } catch(e) {}

    console.log("[Reptool] Storage Hook 完成");
});
"#.to_string()
}

fn native_hook(lib: &str) -> String {
    format!(r#"// Reptool Frida - Native Function Hook
// Hook native .so 库函数: {}

Interceptor.attach(Module.findExportByName(null, "{}"), {{
    onEnter: function(args) {{
        console.log("[Reptool] {} called");
        console.log("  arg0: " + args[0]);
        console.log("  arg1: " + args[1]);
    }},
    onLeave: function(retval) {{
        console.log("[Reptool] {} returned: " + retval);
    }}
}});
"#, lib, lib, lib, lib)
}

fn class_hook(class_name: &str) -> String {
    let script = format!(
        r#"// Reptool Frida - Java Class Hook
// Hook 所有方法: {class_name}

Java.perform(function() {{
    var cls = Java.use("{class_name}");

    var methods = cls.class.getDeclaredMethods();
    methods.forEach(function(method) {{
        var methodName = method.getName();
        console.log("[Reptool] Found method: " + methodName);

        try {{
            cls[methodName].overloads.forEach(function(overload) {{
                overload.implementation = function() {{
                    console.log("[Reptool] {class_name}." + methodName + " called with " + arguments.length + " args");
                    for (var i = 0; i < arguments.length; i++) {{
                        console.log("  arg" + i + ": " + arguments[i]);
                    }}
                    var result = this[methodName].apply(this, arguments);
                    console.log("[Reptool] {class_name}." + methodName + " returned: " + result);
                    return result;
                }};
            }});
        }} catch(e) {{
            console.log("[Reptool] Failed to hook " + methodName + ": " + e);
        }}
    }});

    console.log("[Reptool] Class Hook 完成");
}});
"#,
        class_name = class_name
    );
    script
}

fn general_hook(target: &str) -> String {
    format!(r#"// Reptool Frida - General Hook
// Hook 目标: TARGET

Java.perform(function() {{
    console.log("[Reptool] General Hook 已加载, 目标: {}");

    // 枚举已加载的类
    Java.enumerateLoadedClasses({{
        onMatch: function(className) {{
            if (className.indexOf("{}") !== -1) {{
                console.log("[Reptool] Found class: " + className);
            }}
        }},
        onComplete: function() {{
            console.log("[Reptool] 类枚举完成");
        }}
    }});

    // Hook 加密相关类
    try {{
        var Cipher = Java.use("javax.crypto.Cipher");
        Cipher.getInstance.overload('java.lang.String').implementation = function(transformation) {{
            console.log("[Reptool] Cipher.getInstance: " + transformation);
            return this.getInstance(transformation);
        }};
    }} catch(e) {}

    console.log("[Reptool] General Hook 完成");
}});
"#, target, target, target)
}
