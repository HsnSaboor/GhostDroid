package com.devicespooflab.hooks.utils;

import java.util.Arrays;
import java.util.Collections;
import java.util.HashSet;
import java.util.Locale;
import java.util.Set;

// Single source of truth for root/cloak tooling identity tokens.
// Shared by ApplistHooks (package lists) and ProcessHooks (running
// process/service lists) so the deny set cannot drift between them.
public final class DenyTokens {

    private static final Set<String> PACKAGES = Collections.unmodifiableSet(
            new HashSet<>(Arrays.asList(
                    "com.topjohnwu.magisk",
                    "io.github.huskydg.magisk",
                    "com.hv.magisk",
                    "org.lsposed.manager",
                    "org.lsposed.lspatch",
                    "org.frknkrc44.hma_oss",
                    "com.tsng.hidemyapplist",
                    "de.robv.android.xposed.installer",
                    "re.frida.server",
                    "com.frida.server",
                    "org.maus.lspd",
                    "moe.shizuku.privileged.api")));

    private static final Set<String> SUBSTRINGS = Collections.unmodifiableSet(
            new HashSet<>(Arrays.asList(
                    "magisk", "lsposed", "lspd", "shamiko", "frida",
                    "xposed", "shizuku", "edxposed", "riru")));

    private DenyTokens() {
    }

    public static boolean isDeniedPackage(String pkg) {
        if (pkg == null) {
            return false;
        }
        String lower = pkg.toLowerCase(Locale.US);
        for (String d : PACKAGES) {
            if (lower.equals(d) || lower.startsWith(d + ".")) {
                return true;
            }
        }
        for (String token : SUBSTRINGS) {
            if (lower.contains(token)) {
                return true;
            }
        }
        return false;
    }

    public static boolean isDeniedProcess(String processName) {
        if (processName == null) {
            return false;
        }
        String lower = processName.toLowerCase(Locale.US);
        if (lower.contains(":")) {
            lower = lower.substring(0, lower.indexOf(':'));
        }
        if (isDeniedPackage(lower)) {
            return true;
        }
        return lower.contains("frida")
                || lower.contains("magisk")
                || lower.contains("xposed")
                || lower.contains("lsposed")
                || lower.contains("shamiko")
                || lower.contains("shizuku")
                || lower.contains("riru")
                || lower.contains("edxposed");
    }
}
