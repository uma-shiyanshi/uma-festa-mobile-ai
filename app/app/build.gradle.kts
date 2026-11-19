plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

// trigger: 2026-11-19 / fiveStatusLimit = 模拟器上限
android {
    namespace = "com.ranko.matsuricapture"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.ranko.matsuricapture"
        minSdk = 24
        targetSdk = 35
        versionCode = 66
        versionName = "0.9.41-dsh50"
    }

    // 18766 接收端改为纯 Kotlin 实现（NativeEngine.kt），不再需要 NDK / .so

    // [APP-SIGN-FIXED] 固定签名：用仓库内 app/ci/debug.keystore，CI 每次构建签名一致，
    // 后续覆盖安装不需要卸载、不清数据。
    signingConfigs {
        create("fixed") {
            storeFile = rootProject.file("ci/debug.keystore")
            storePassword = "android"
            keyAlias = "androiddebugkey"
            keyPassword = "android"
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.getByName("fixed")
        }
        debug {
            signingConfig = signingConfigs.getByName("fixed")
        }
    }

    // [APP-FIX-085] 内置引擎是要 exec 的二进制，必须解压到 nativeLibraryDir
    packaging {
        jniLibs {
            useLegacyPackaging = true
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("androidx.webkit:webkit:1.12.1")

    // 归一化单测：用真实 org.json 实现，避开 mockable android.jar 的 stub
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.json:json:20240303")
}