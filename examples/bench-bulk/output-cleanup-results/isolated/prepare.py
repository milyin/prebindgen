"""Instrument an identical benchmark driver; reuse the frozen native libraries and generated classes."""
import argparse
import difflib
import os
import hashlib
import json
import subprocess
from pathlib import Path

p = argparse.ArgumentParser(__doc__)
p.add_argument('--before', type=Path, required=True)
p.add_argument('--after', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--kotlinc', default=os.environ.get('KOTLINC', 'kotlinc'))
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
assert not list(a.output.iterdir()), 'Use an empty output directory'
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
info = {v: json.loads((d / 'build.json').read_text()) for v, d in [('before', a.before), ('after', a.after)]}
for key in ['fixture_sha256', 'rustc', 'kotlin', 'dependency_lock_sha256', 'cargo_config_sha256', 'cargo_profile', 'profile_environment']:
    assert info['before'][key] == info['after'][key], key
for d in info.values():
    assert sha(d['jar']) == d['jar_sha256'] and sha(d['library']) == d['native_sha256']
source = Path(__file__).resolve().parents[2] / 'Bench.kt'
assert sha(source) == info['before']['fixture_sha256']['Bench.kt']
s = source.read_text()
def replace(old, new):
    global s
    assert s.count(old) == 1, old
    s = s.replace(old, new)
replace('private data class Sample(val nanos: Long, val allocated: Long)',
        'private data class Sample(val nanos: Long, val allocated: Long, val cpuNanos: Long)')
replace('    val thread = Thread.currentThread().id', '''    val thread = Thread.currentThread().id
    check(bean.isCurrentThreadCpuTimeSupported)
    bean.isThreadCpuTimeEnabled = true
    val runtime = ManagementFactory.getRuntimeMXBean()
    val compiler = ManagementFactory.getCompilationMXBean()
    check(compiler.isCompilationTimeMonitoringSupported)
    val collectors = ManagementFactory.getGarbageCollectorMXBeans()
    fun gcCount() = collectors.sumOf { it.collectionCount.coerceAtLeast(0L) }
    fun gcTime() = collectors.sumOf { it.collectionTime.coerceAtLeast(0L) }
    val minWarmup = System.getenv("PB_MIN_WARMUP_MS").toLong() * 1000000L
    val quietPeriod = System.getenv("PB_COMPILE_QUIET_MS").toLong() * 1000000L
    val maxWarmup = System.getenv("PB_MAX_WARMUP_MS").toLong() * 1000000L''')
replace('    val ordered = if (trial % 2 == 0)', '''    require(selectedCases.size == 1) { "one case and size per JVM required" }
    val ordered = if (trial % 2 == 0)''')
replace('        val start = System.nanoTime()', '        val cpuStart = bean.currentThreadCpuTime\n        val start = System.nanoTime()')
replace('        val elapsed = System.nanoTime() - start', '        val elapsed = System.nanoTime() - start\n        val cpuNanos = bean.currentThreadCpuTime - cpuStart')
replace('        return Sample(elapsed, allocated)', '        return Sample(elapsed, allocated, cpuNanos)')
replace('            repeat(warmup) { last = sample(case, iterations).nanos }', '''            val warmStart = System.nanoTime()
            var compilation = compiler.totalCompilationTime
            var lastCompilation = warmStart
            var warmBatches = 0
            while (warmBatches <= warmup || System.nanoTime() - warmStart < minWarmup || System.nanoTime() - lastCompilation < quietPeriod) {
                // Many short calls settle entry compilation; finish with full-sized batches.
                val warmIterations = if (warmBatches < warmup) (iterations / 10).coerceAtLeast(1) else iterations
                last = sample(case, warmIterations).nanos
                warmBatches++
                val current = compiler.totalCompilationTime
                if (current != compilation) {
                    compilation = current
                    lastCompilation = System.nanoTime()
                }
                // Exercise the diagnostics outside the timed operation loop.
                gcCount(); gcTime(); runtime.uptime
                if (System.nanoTime() - warmStart >= maxWarmup) break
            }
            val quietMillis = (System.nanoTime() - lastCompilation) / 1000000L
            val stable = warmBatches > warmup && System.nanoTime() - warmStart >= minWarmup && System.nanoTime() - lastCompilation >= quietPeriod
            check(warmBatches > warmup) { "warmup did not reach full-sized batches" }
            println("BENCH_WARMUP,${case.name},${case.size},$warmBatches,${runtime.uptime},$quietMillis,$stable")''')
replace('            repeat(batches) { batch ->\n                val s = sample(case, iterations)', '''            println("BENCH_PHASE,${case.name},${case.size},start,${runtime.uptime},${compiler.totalCompilationTime}")
            repeat(batches) { batch ->
                val gcBefore = gcCount(); val gcMillisBefore = gcTime()
                val compilationBefore = compiler.totalCompilationTime
                val startMillis = runtime.uptime
                val s = sample(case, iterations)
                val endMillis = runtime.uptime
                val compilationAfter = compiler.totalCompilationTime
                val gcAfter = gcCount(); val gcMillisAfter = gcTime()
                println("BENCH_DIAG,${case.name},${case.size},$batch,$startMillis,$endMillis,${s.cpuNanos},$compilationBefore,$compilationAfter,$gcBefore,$gcAfter,$gcMillisBefore,$gcMillisAfter")''')
replace('        println("SINK,$observed")', '        println("BENCH_PHASE,${cases.single().name},${cases.single().size},end,${runtime.uptime},${compiler.totalCompilationTime}")\n        println("SINK,$observed")')
# The operation loop is preserved; diagnostics and warmup surround it.
assert 'while (i < iterations) { sum += case.operation.run(i); i++ }' in s
instrumented = a.output / 'Bench.kt'
instrumented.write_text(s)
(a.output / 'driver.patch').write_text(''.join(difflib.unified_diff(source.read_text().splitlines(True), s.splitlines(True), fromfile='Bench.kt', tofile='isolated/Bench.kt')))
compiler = info['before']['kotlin']
command = [a.kotlinc, '-cp', info['before']['jar'], str(instrumented), '-nowarn', '-d', str(a.output / 'driver.jar')]
version = subprocess.run([command[0], '-version'], capture_output=True, text=True, check=True)
assert (version.stdout + version.stderr).strip() == compiler
subprocess.run(command, check=True)
manifest = dict(builds=info, original_source_sha256=sha(source), instrumented_source_sha256=sha(instrumented), driver_jar=str((a.output / 'driver.jar').resolve()), driver_jar_sha256=sha(a.output / 'driver.jar'), compiler=compiler, compile_command=command, prepare_sha256=sha(__file__))
(a.output / 'driver.json').write_text(json.dumps(manifest, indent=2) + '\n')
print('Prepared identical instrumented driver:', a.output / 'driver.jar', flush=True)
