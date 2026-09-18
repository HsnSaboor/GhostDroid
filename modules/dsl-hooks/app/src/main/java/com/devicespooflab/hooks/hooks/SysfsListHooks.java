// DeviceInfoHW USB/DRIVERS tabs enumerate sysfs via java.io.File
// (File.listFiles -> opendir/getdents64), which bypasses libc open/openat
// export hooks. Hook at the Java level instead:
//
// - File.listFiles(File.list / listFiles, 4 overloads): when the target dir
//   is a masked sysfs tree, return a synthetic listing containing only a
//   single Qualcomm node.
// - File.exists / isDirectory: keep masked dirs "present" so the app takes
//   the sysfs path (and shows our fakes) instead of falling back to a
//   different probe (e.g. /proc or dumpsys) that would leak host strings.
//
// Masked trees (contents fully synthetic):
//   /sys/bus/usb/drivers/usb          -> ["usb1"]
//   /sys/bus/usb/devices              -> ["usb1"]
//   /sys/bus/pci/drivers              -> ["pcieport"]
//   /sys/bus/pci/devices              -> ["0000:00:00.0"]
//   /sys/bus/platform/drivers         -> ["phone"]
//   /sys/module                       -> virtio_* only
//
// NOTE: /sys/devices/pci0000:00 is intentionally NOT masked — minigbm/libdrm
// need the real Intel GPU node for GBM BO allocation (2026-09-19 root
// cause). PCI Wi-Fi strings are covered by the native uevent redirect
// (file_hooks.cpp -> gs_fake_pci_uevent) for readers that open uevent
// files; directory listings of pci/devices stay real.
package com.devicespooflab.hooks.hooks;

import java.io.File;
import java.io.FileFilter;
import java.io.FilenameFilter;

import de.robv.android.xposed.XC_MethodHook;
import de.robv.android.xposed.XposedBridge;
import de.robv.android.xposed.XposedHelpers;
import de.robv.android.xposed.callbacks.XC_LoadPackage;

public class SysfsListHooks {

    private static final String TAG = "DeviceSpoofLab-SysfsList";

    private static final String USB_DRIVERS = "/sys/bus/usb/drivers/usb";
    private static final String USB_DEVICES = "/sys/bus/usb/devices";
    private static final String PCI_DRIVERS = "/sys/bus/pci/drivers";
    private static final String PCI_DEVICES = "/sys/bus/pci/devices";
    private static final String PLATFORM_DRIVERS = "/sys/bus/platform/drivers";
    private static final String SYS_MODULE = "/sys/module";

    public static void hook(XC_LoadPackage.LoadPackageParam lpparam) {
        hookListFiles();
        hookList();
    }

    private static String[] fakeListing(String path) {
        if (USB_DRIVERS.equals(path) || USB_DEVICES.equals(path)) {
            return new String[]{"usb1"};
        }
        if (PCI_DRIVERS.equals(path)) {
            return new String[]{"pcieport"};
        }
        if (PCI_DEVICES.equals(path)) {
            return new String[]{"0000:00:00.0"};
        }
        if (PLATFORM_DRIVERS.equals(path)) {
            return new String[]{"phone"};
        }
        if (SYS_MODULE.equals(path)) {
            return new String[]{
                    "virtio_gpu", "virtio_input", "virtio_snd",
                    "virtio_blk", "virtio_net"};
        }
        return null;
    }

    private static boolean isMasked(String path) {
        return fakeListing(path) != null;
    }

    private static void hookListFiles() {
        try {
            XposedHelpers.findAndHookMethod(File.class, "listFiles",
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            File self = (File) param.thisObject;
                            if (self == null) return;
                            String[] fake = fakeListing(self.getPath());
                            if (fake == null) return;
                            File[] out = new File[fake.length];
                            for (int i = 0; i < fake.length; i++) {
                                out[i] = new File(self, fake[i]);
                            }
                            param.setResult(out);
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook File.listFiles: " + t);
        }
        try {
            XposedHelpers.findAndHookMethod(File.class, "listFiles",
                    FileFilter.class,
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            File self = (File) param.thisObject;
                            if (self == null) return;
                            String[] fake = fakeListing(self.getPath());
                            if (fake == null) return;
                            FileFilter filter = (FileFilter) param.args[0];
                            java.util.ArrayList<File> out = new java.util.ArrayList<>();
                            for (String name : fake) {
                                File f = new File(self, name);
                                if (filter == null || filter.accept(f)) out.add(f);
                            }
                            param.setResult(out.toArray(new File[0]));
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook File.listFiles(FileFilter): " + t);
        }
        try {
            XposedHelpers.findAndHookMethod(File.class, "listFiles",
                    FilenameFilter.class,
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            File self = (File) param.thisObject;
                            if (self == null) return;
                            String[] fake = fakeListing(self.getPath());
                            if (fake == null) return;
                            FilenameFilter filter = (FilenameFilter) param.args[0];
                            java.util.ArrayList<File> out = new java.util.ArrayList<>();
                            for (String name : fake) {
                                if (filter == null
                                        || filter.accept(self, name)) {
                                    out.add(new File(self, name));
                                }
                            }
                            param.setResult(out.toArray(new File[0]));
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook File.listFiles(FilenameFilter): " + t);
        }
    }

    private static void hookList() {
        try {
            XposedHelpers.findAndHookMethod(File.class, "list",
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            File self = (File) param.thisObject;
                            if (self == null) return;
                            String[] fake = fakeListing(self.getPath());
                            if (fake == null) return;
                            param.setResult(fake.clone());
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook File.list: " + t);
        }
        try {
            XposedHelpers.findAndHookMethod(File.class, "list",
                    FilenameFilter.class,
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            File self = (File) param.thisObject;
                            if (self == null) return;
                            if (!isMasked(self.getPath())) return;
                            String[] fake = fakeListing(self.getPath());
                            FilenameFilter filter = (FilenameFilter) param.args[0];
                            if (filter == null) {
                                param.setResult(fake.clone());
                                return;
                            }
                            java.util.ArrayList<String> out = new java.util.ArrayList<>();
                            for (String name : fake) {
                                if (filter.accept(self, name)) out.add(name);
                            }
                            param.setResult(out.toArray(new String[0]));
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook File.list(FilenameFilter): " + t);
        }
    }
}
