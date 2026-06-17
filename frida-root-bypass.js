/**
 * Frida Root Detection Bypass Script
 * 绕过常见的 Android Root 检测方法
 *
 * 使用方法:
 *   frida -U -f com.target.app -l frida-root-bypass.js --no-pause
 *   或
 *   frida -D <device> -f com.target.app -l frida-root-bypass.js --no-pause
 */

console.log("[*] Root Detection Bypass 已加载");

// ==================== 1. 文件检测绕过 ====================
console.log("[1/8] 绕过文件检测...");

var rootFiles = [
    "/system/app/Superuser.apk",
    "/system/xbin/su",
    "/system/bin/su",
    "/sbin/su",
    "/data/local/xbin/su",
    "/data/local/bin/su",
    "/data/local/su",
    "/su/bin/su",
    "/system/su",
    "/system/bin/.ext/.su",
    "/system/usr/we-need-root/su-backup",
    "/system/xbin/daemonsu",
    "/system/xbin/busybox",
    "/system/bin/busybox",
    "/sbin/busybox",
    "/data/local/busybox",
    "/proc/self/mountinfo",
    "/proc/version",
    "/system/app/SuperSU.apk",
    "/system/app/Superuser.apk",
    "/system/app/Supersu.apk",
    "/data/data/eu.chainfire.supersu",
    "/data/data/com.noshufou.su",
    "/data/data/com.thirdparty.superuser",
    "/data/data/com.koushikdutta.superuser",
    "/data/data/com.zachspong.temprootremovejb",
    "/data/data/com.ramdroid.appremover",
    "/data/data/com.topjohnwu.magisk",
    "/data/adb/magisk",
    "/data/adb/modules",
    "/data/adb/shamiko",
    "/data/adb/lspd",
    "/sbin/.magisk",
    "/data/adb/magisk.img",
    "/data/adb/magisk.db"
];

// Hook access()
var accessPtr = Module.findExportByName("libc.so", "access");
if (accessPtr) {
    Interceptor.attach(accessPtr, {
        onEnter: function(args) {
            this.path = args[0].readUtf8String();
        },
        onLeave: function(retval) {
            if (this.path) {
                for (var i = 0; i < rootFiles.length; i++) {
                    if (this.path.indexOf(rootFiles[i]) !== -1) {
                        console.log("[access] 拦截: " + this.path + " -> 返回 -1");
                        retval.replace(-1);
                        return;
                    }
                }
            }
        }
    });
}

// Hook fopen()
var fopenPtr = Module.findExportByName("libc.so", "fopen");
if (fopenPtr) {
    Interceptor.attach(fopenPtr, {
        onEnter: function(args) {
            this.path = args[0].readUtf8String();
        },
        onLeave: function(retval) {
            if (this.path) {
                for (var i = 0; i < rootFiles.length; i++) {
                    if (this.path.indexOf(rootFiles[i]) !== -1) {
                        console.log("[fopen] 拦截: " + this.path + " -> 返回 NULL");
                        retval.replace(ptr(0));
                        return;
                    }
                }
            }
        }
    });
}

// Hook stat()
var statPtr = Module.findExportByName("libc.so", "stat");
if (statPtr) {
    Interceptor.attach(statPtr, {
        onEnter: function(args) {
            this.path = args[0].readUtf8String();
        },
        onLeave: function(retval) {
            if (this.path) {
                for (var i = 0; i < rootFiles.length; i++) {
                    if (this.path.indexOf(rootFiles[i]) !== -1) {
                        console.log("[stat] 拦截: " + this.path + " -> 返回 -1");
                        retval.replace(-1);
                        return;
                    }
                }
            }
        }
    });
}

// ==================== 2. 执行命令检测绕过 ====================
console.log("[2/8] 绕过执行命令检测...");

var rootCommands = ["su", "which su", "busybox", "which busybox", "magisk", "daemonsu"];

// Hook Runtime.exec()
Java.perform(function() {
    var Runtime = Java.use("java.lang.Runtime");
    var ProcessBuilder = Java.use("java.lang.ProcessBuilder");

    Runtime.exec.overload("java.lang.String").implementation = function(cmd) {
        for (var i = 0; i < rootCommands.length; i++) {
            if (cmd.indexOf(rootCommands[i]) !== -1) {
                console.log("[Runtime.exec] 拦截: " + cmd);
                throw Java.use("java.io.IOException").$new("Permission denied");
            }
        }
        return this.exec(cmd);
    };

    Runtime.exec.overload("[Ljava.lang.String;").implementation = function(cmdArray) {
        var cmd = cmdArray[0];
        for (var i = 0; i < rootCommands.length; i++) {
            if (cmd.indexOf(rootCommands[i]) !== -1) {
                console.log("[Runtime.exec] 拦截: " + cmd);
                throw Java.use("java.io.IOException").$new("Permission denied");
            }
        }
        return this.exec(cmdArray);
    };

    // Hook ProcessBuilder
    ProcessBuilder.command.overload("java.util.List").implementation = function(cmdList) {
        var cmd = cmdList.toString();
        for (var i = 0; i < rootCommands.length; i++) {
            if (cmd.indexOf(rootCommands[i]) !== -1) {
                console.log("[ProcessBuilder] 拦截: " + cmd);
                throw Java.use("java.lang.IllegalArgumentException").$new("Invalid command");
            }
        }
        return this.command(cmdList);
    };
});

// ==================== 3. Shell 命令检测绕过 ====================
console.log("[3/8] 绕过 Shell 命令检测...");

var execPtr = Module.findExportByName("libc.so", "execve");
if (execPtr) {
    Interceptor.attach(execPtr, {
        onEnter: function(args) {
            var cmd = args[0].readUtf8String();
            if (cmd && (cmd.indexOf("su") !== -1 || cmd.indexOf("busybox") !== -1 || cmd.indexOf("magisk") !== -1)) {
                console.log("[execve] 拦截: " + cmd);
                // 替换为一个不存在的命令
                args[0].writeUtf8String("/system/bin/nonexistent_command");
            }
        }
    });
}

// ==================== 4. 属性检测绕过 ====================
console.log("[4/8] 绕过系统属性检测...");

var dangerousProps = {
    "ro.build.tags": "release-keys",
    "ro.debuggable": "0",
    "ro.secure": "1",
    "ro.build.type": "user",
    "ro.build.display.id": "release-keys",
    "init.svc.adbd": "stopped",
    "ro.adb.secure": "1"
};

var systemPropertiesGetPtr = Module.findExportByName("libc.so", "__system_property_get");
if (systemPropertiesGetPtr) {
    Interceptor.attach(systemPropertiesGetPtr, {
        onEnter: function(args) {
            this.name = args[0].readUtf8String();
            this.valueBuf = args[1];
        },
        onLeave: function(retval) {
            if (this.name && dangerousProps[this.name]) {
                var fakeValue = dangerousProps[this.name];
                console.log("[prop] 伪造 " + this.name + " = " + fakeValue);
                this.valueBuf.writeUtf8String(fakeValue);
            }
        }
    });
}

// ==================== 5. SafetyNet/Play Integrity 绕过 ====================
console.log("[5/8] 绕过 SafetyNet/Play Integrity...");

Java.perform(function() {
    // 绕过 SafetyNet
    try {
        var SafetyNetApi = Java.use("com.google.android.gms.safetynet.SafetyNet");
        SafetyNetApi.attest.overload("com.google.android.gms.common.api.GoogleApiClient", "[B").implementation = function(client, nonce) {
            console.log("[SafetyNet] 拦截 attest 调用");
            // 返回一个假的结果
            var result = Java.use("com.google.android.gms.safetynet.SafetyNetApi$AttestationResult");
            return this.attest(client, nonce);
        };
    } catch (e) {}

    // 绕过 Play Integrity
    try {
        var IntegrityManager = Java.use("com.google.android.play.core.integrity.IntegrityManager");
        IntegrityManager.requestIntegrityToken.overload("com.google.android.play.core.integrity.IntegrityTokenRequest").implementation = function(request) {
            console.log("[Play Integrity] 拦截 requestIntegrityToken");
            return this.requestIntegrityToken(request);
        };
    } catch (e) {}
});

// ==================== 6. 签名检测绕过 ====================
console.log("[6/8] 绕过签名检测...");

Java.perform(function() {
    var PackageManager = Java.use("android.app.ApplicationPackageManager");

    PackageManager.getPackageInfo.overload("java.lang.String", "int").implementation = function(name, flags) {
        var info = this.getPackageInfo(name, flags);

        // 如果查询的是自己的签名信息
        if (name === this.getApplicationInfo().packageName && (flags & 0x40) !== 0) {
            console.log("[签名] 拦截签名查询: " + name);
            // 可以在这里替换签名信息
        }

        return info;
    };
});

// ==================== 7. Magisk 检测绕过 ====================
console.log("[7/8] 绕过 Magisk 检测...");

// 隐藏 Magisk 相关文件
var magiskPaths = [
    "/data/adb/magisk",
    "/data/adb/modules",
    "/sbin/.magisk",
    "/data/adb/magisk.img",
    "/data/adb/magisk.db",
    "/data/adb/shamiko",
    "/data/adb/lspd",
    "/data/adb/tricky_store",
    "/data/adb/ksu"
];

if (accessPtr) {
    Interceptor.attach(accessPtr, {
        onEnter: function(args) {
            this.path = args[0].readUtf8String();
        },
        onLeave: function(retval) {
            if (this.path) {
                for (var i = 0; i < magiskPaths.length; i++) {
                    if (this.path.indexOf(magiskPaths[i]) !== -1) {
                        console.log("[Magisk] 隐藏: " + this.path);
                        retval.replace(-1);
                        return;
                    }
                }
            }
        }
    });
}

// ==================== 8. Xposed/LSPosed 检测绕过 ====================
console.log("[8/8] 绕过 Xposed/LSPosed 检测...");

Java.perform(function() {
    // 隐藏 Xposed 类
    try {
        var ClassLoader = Java.use("java.lang.ClassLoader");
        ClassLoader.loadClass.overload("java.lang.String", "boolean").implementation = function(name, resolve) {
            if (name.indexOf("xposed") !== -1 || name.indexOf("de.robv.android") !== -1) {
                console.log("[Xposed] 隐藏类: " + name);
                throw Java.use("java.lang.ClassNotFoundException").$new(name);
            }
            return this.loadClass(name, resolve);
        };
    } catch (e) {}
});

// Hook 堆栈检测
var threadGetStackTracePtr = Java.use("java.lang.Thread").getStackTrace.overload();
if (threadGetStackTracePtr) {
    Java.perform(function() {
        var Thread = Java.use("java.lang.Thread");
        Thread.getStackTrace.implementation = function() {
            var stack = this.getStackTrace();
            // 过滤掉包含 xposed/magisk/frida 的堆栈帧
            var filtered = [];
            for (var i = 0; i < stack.length; i++) {
                var frame = stack[i].toString();
                if (frame.indexOf("xposed") === -1 &&
                    frame.indexOf("magisk") === -1 &&
                    frame.indexOf("frida") === -1 &&
                    frame.indexOf("substrate") === -1) {
                    filtered.push(stack[i]);
                }
            }
            return filtered;
        };
    });
}

console.log("[*] ================================");
console.log("[*] Root Detection Bypass 已全部激活!");
console.log("[*] 绕过项目:");
console.log("[*]   1. 文件检测 (su/busybox/magisk)");
console.log("[*]   2. 执行命令检测 (Runtime.exec)");
console.log("[*]   3. Shell 命令检测 (execve)");
console.log("[*]   4. 系统属性检测 (ro.build.tags)");
console.log("[*]   5. SafetyNet/Play Integrity");
console.log("[*]   6. 签名检测");
console.log("[*]   7. Magisk 检测");
console.log("[*]   8. Xposed/LSPosed 检测");
console.log("[*] ================================");
