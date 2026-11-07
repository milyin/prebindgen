import bench.types.*
import com.sun.management.ThreadMXBean
import java.lang.management.ManagementFactory
import kotlin.random.Random

@Volatile private var sink=0L
private data class Case(val name:String,val size:Int,val expected:Long,val run:()->Long)
private fun fixture(sizes:List<Int>,shuffle:Boolean):Pair<List<Case>,AutoCloseable>{
 val close=mutableListOf<AutoCloseable>();val cases=mutableListOf<Case>()
 fun <T> order(xs:List<T>,seed:Int):List<T> = if(shuffle)xs.shuffled(Random(seed)) else xs
 cases.add(Case("control",1,43){control(42)})
 for(n in sizes){
  val a=order(List(n){aNew(it+1L)},n+1);close.addAll(a)
  val expected=n.toLong()*(n+1)/2
  cases.add(Case("read_one",n,expected){readOne(a)})
  cases.add(Case("mut_one",n,expected){mutOne(a)})
  val nested=a.map {listOf(it)}
  cases.add(Case("mut_nested",n,expected){mutNested(nested)})
  val groups=(0..3).map {g ->(n/4)+if(g<n%4)1 else 0}
  // Allocate types interleaved; grouping must not benefit merely from
  // different types occupying disjoint address runs due to fixture creation.
  val makeA=ArrayList<A>();val makeB=ArrayList<B>();val makeC=ArrayList<C>();val makeD=ArrayList<D>()
  for(i in 0 until groups.max()){
   if(i<groups[0])makeA.add(aNew(i+1L))
   if(i<groups[1])makeB.add(bNew(i+1L))
   if(i<groups[2])makeC.add(cNew(i+1L))
   if(i<groups[3])makeD.add(dNew(i+1L))
  }
  val aa=order(makeA,n+2);close.addAll(aa)
  val b=order(makeB,n+3);close.addAll(b)
  val c=order(makeC,n+4);close.addAll(c)
  val d=order(makeD,n+5);close.addAll(d)
  val token=tokenNew(7);close.add(token)
  val sum=groups.sumOf {it.toLong()*(it+1)/2}
  cases.add(Case("mixed_shared",n,sum+7){mixedShared(aa,b,c,d,token)})
  cases.add(Case("mixed_mut",n,sum){mixedMut(aa,b,c,d)})
  var owners=order(List(n){aNew(it+1L)},n+6)
  close.add(AutoCloseable{owners.forEach{it.close()}})
  cases.add(Case("owned_echo",n,n.toLong()){owners=ownedEcho(owners);owners.size.toLong()})
 }
 return cases to AutoCloseable{close.asReversed().forEach{it.close()}}
}
fun main(args:Array<String>){
 val sizes=args[0].split(',').map{it.toInt()};val shuffle=args[1]=="shuffled"
 val fork=args[2].toInt();val warm=args[3].toInt();val batches=args[4].toInt();val target=args[5].toLong()*1_000_000
 val selected=args.getOrNull(6)?.split(',')?.toSet()
 val (small,smallClose)=fixture(listOf(8),shuffle)
 try{repeat(20000){for(c in small)if(selected==null||c.name in selected){val value=c.run();check(value==c.expected);sink=sink xor value}}}finally{smallClose.close()}
 val (all,close)=fixture(sizes,shuffle)
 var cases=all.filter{selected==null||it.name in selected};if(fork%2==1)cases=cases.reversed()
 val shift=(fork/2)%cases.size;cases=cases.drop(shift)+cases.take(shift)
 val bean=ManagementFactory.getThreadMXBean() as ThreadMXBean;bean.isThreadAllocatedMemoryEnabled=true
 val thread=Thread.currentThread().id
 fun sample(c:Case,n:Int):Pair<Long,Long>{
  val before=bean.getThreadAllocatedBytes(thread);val start=System.nanoTime();var sum=0L
  repeat(n){sum+=c.run()};val ns=System.nanoTime()-start;val allocated=bean.getThreadAllocatedBytes(thread)-before
  check(sum==c.expected*n){"wrong checksum ${c.name}/${c.size}: $sum"};sink=sink xor sum
  return ns to allocated
 }
 try{
  repeat(2){for(c in cases)sample(c,16)}
  for(c in cases){
   var n=(10000/(c.size+1)).coerceIn(1,10000)
   fun calibrate(ns:Long):Int=(n*(target.toDouble()/ns).coerceIn(.25,8.0)).toInt().coerceIn(1,5_000_000)
   repeat(3){n=calibrate(sample(c,n).first)};var last=1L
   repeat(warm){last=sample(c,n).first};n=calibrate(last)
   repeat(batches){i->val(ns,bytes)=sample(c,n);println("TYPED_BENCH,${c.name},${c.size},$i,$n,$ns,$bytes")}
  }
 }finally{close.close()}
 check(live()==0L){"native owners leaked after benchmark: ${live()}"};println("SINK,$sink")
}
