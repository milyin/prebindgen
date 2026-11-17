import bench.*
private var sink = 0L
fun main(args: Array<String>) {
    val count=args[0].toLong()
    val scalar=scalarHandlerNew({x -> sink += x}, {m -> error(m ?: "native error")})
    val record=recordHandlerNew({r -> sink += r.x + r.y}, {m -> error(m ?: "native error")})
    val owned=ownedHandlerNew({_ -> sink++}, {m -> error(m ?: "native error")})
    val cases=listOf<Pair<String,()->Long>>("scalar" to {scalarFire(scalar,count) {m -> error(m ?: "native error")}}, "record" to {recordFire(record,count) {m -> error(m ?: "native error")}}, "owned" to {ownedFire(owned,count) {m -> error(m ?: "native error")}})
    repeat(args[1].toInt()) { for ((_,run) in cases) run() }
    repeat(args[2].toInt()) { pass ->
        for ((name,run) in if(pass%2==0) cases else cases.reversed()) println("BENCH,$name,$count,${run()}")
    }
    scalar.close();record.close();owned.close()
    println("SINK,$sink")
}
