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

// Hand-written loader/benchmark live in `kotlin/`; the prebindgen-generated
// classes are written by build.rs into `kotlin/generated/`.
sourceSets {
    main {
        kotlin.srcDirs("kotlin", "kotlin/generated")
    }
}

// Workspace root is two levels up (examples/perftest-kotlin/ -> repo root).
val workspaceRoot = projectDir.parentFile.parentFile
val releaseDir = workspaceRoot.resolve("target/release")

// Build the Rust cdylib (and regenerate src/generated_bindings.rs + kotlin/generated/**).
val buildRustJni by tasks.registering(Exec::class) {
    description = "cargo build --release -p perftest-kotlin (cdylib + generated sources)"
    workingDir = workspaceRoot
    commandLine("cargo", "build", "--release", "-p", "perftest-kotlin")
}

tasks.named("compileKotlin") {
    dependsOn(buildRustJni)
}

application {
    mainClass.set("io.prebindgen.perftest.BenchKt")
    // Find libperftest_kotlin.{dylib,so} produced by cargo without installing it.
    applicationDefaultJvmArgs = listOf("-Djava.library.path=${releaseDir.absolutePath}")
}

tasks.named<JavaExec>("run") {
    dependsOn(buildRustJni)
    // `./gradlew run -PperftestN=<N>` → `-Dperftest.n=<N>` for the forked app JVM, so
    // the shared `perftest-bench.sh` harness can set the iteration count (daemon-env
    // independent, unlike reading PERFTEST_N from the Gradle process environment).
    (project.findProperty("perftestN") as String?)?.let { systemProperty("perftest.n", it) }
}
