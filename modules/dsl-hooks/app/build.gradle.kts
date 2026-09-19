plugins {
    id("com.android.application")
}

android {
    namespace = "com.devicespooflab.hooks"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.devicespooflab.hooks"
        minSdk = 26
        targetSdk = 34
        versionCode = 5
        versionName = "1.3-ghostdroid"

        // GhostDroid Option A: Java-only. No native build — the LSPlt
        // cpp/ tree (property/uname hooks) is dead code; ghost-stealth
        // owns those symbols via Dobby. Shipping a ds_native.so would
        // reintroduce the SEGV_ACCERR collision. (trigger stealth-ndk)
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
        jniLibs {
            useLegacyPackaging = false
        }
    }
}

dependencies {
    // Pure libxposed API 102: zero de.robv references (Vector blocks
    // de.robv.* in target processes when targetApiVersion >= 102).
    // Stub is compileOnly (Vector injects the real runtime into targets).
    compileOnly(files("libs/libxposed-api-stub.jar"))
    // Service bridge MUST be packaged (implementation): it provides
    // XposedProvider + IXposedService binder used for RemotePreferences.
    // (Verified present in classes.dex; do NOT demote to compileOnly.)
    implementation("io.github.libxposed:service:102.0.0")
    // lsplt REMOVED (Java-only): native hook dep would pull the
    // LSPlt runtime into the APK for zero benefit.
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.recyclerview:recyclerview:1.3.2")
}
