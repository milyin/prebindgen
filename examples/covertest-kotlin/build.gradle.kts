plugins {
    kotlin("jvm") version "1.9.25"
    application
}

repositories {
    mavenCentral()
}

kotlin {
    jvmToolchain(21)
}

// Hand-written loader + assertion test live in `kotlin/`; the prebindgen-generated
// classes are written by build.rs into `kotlin/generated/`.
sourceSets {
    main {
        kotlin.srcDirs("kotlin", "kotlin/generated")
    }
}

// Workspace root is two levels up (examples/covertest-kotlin/ -> repo root).
val workspaceRoot = projectDir.parentFile.parentFile
val releaseDir = workspaceRoot.resolve("target/release")

// Build the Rust cdylib (and regenerate src/generated_bindings.rs + kotlin/generated/**).
val buildRustJni by tasks.registering(Exec::class) {
    description = "cargo build --release -p covertest-kotlin (cdylib + generated sources)"
    workingDir = workspaceRoot
    commandLine("cargo", "build", "--release", "-p", "covertest-kotlin")
}

tasks.named("compileKotlin") {
    dependsOn(buildRustJni)
}

application {
    mainClass.set("io.prebindgen.covertest.TestKt")
    // Find libcovertest_kotlin.{dylib,so} produced by cargo without installing it.
    applicationDefaultJvmArgs = listOf("-Djava.library.path=${releaseDir.absolutePath}")
}

tasks.named<JavaExec>("run") {
    dependsOn(buildRustJni)
    // Surface a non-zero exit when the coverage asserts fail.
    setIgnoreExitValue(false)
}
