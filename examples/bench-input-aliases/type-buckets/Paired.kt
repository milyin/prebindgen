import bench.types.*
import com.sun.management.ThreadMXBean
import java.lang.management.ManagementFactory
import kotlin.random.Random

@Volatile private var pairedSink=0L
private data class PairedCase(val name:String,val size:Int,val expected:Long,val run:List<()->Long>)
fun main(args:Array<String>) {
 val sizes=args[0].split(',').map{it.toInt()};val shuffle=args[1]=="shuffled";val fork=args[2].toInt()
 val batches=args[3].toInt();val target=args[4].toLong()*1000000;val aa=args.getOrNull(5)=="aa"
 val close=mutableListOf<AutoCloseable>();val cases=mutableListOf<PairedCase>()
 val smallA=aNew(1);val smallB=bNew(2);val smallC=cNew(3);val smallD=dNew(4);val smallToken=tokenNew(7)
 try {
  repeat(20000){
   check(mixedMut(listOf(smallA),listOf(smallB),listOf(smallC),listOf(smallD))==10L)
   check(mixedMutFlat(listOf(smallA),listOf(smallB),listOf(smallC),listOf(smallD))==10L)
   check(mixedShared(listOf(smallA),listOf(smallB),listOf(smallC),listOf(smallD),smallToken)==17L)
   check(mixedSharedFlat(listOf(smallA),listOf(smallB),listOf(smallC),listOf(smallD),smallToken)==17L)
  }
  for(call in listOf<()->Long>(
    {mixedMut(listOf(smallA,smallA),listOf(smallB),listOf(smallC),listOf(smallD))},
    {mixedMutFlat(listOf(smallA,smallA),listOf(smallB),listOf(smallC),listOf(smallD))}
  )) {var failed=false;try{call()}catch(e:IllegalArgumentException){failed=true};check(failed)}
  check(mixedSharedFlat(listOf(smallA,smallA),listOf(smallB),listOf(smallC),listOf(smallD),smallToken)==18L)
 } finally {smallToken.close();smallD.close();smallC.close();smallB.close();smallA.close()}
 cases.add(PairedCase("control",1,43,listOf({control(42)},{control(42)})))
 for(n in sizes) {
  val lengths=IntArray(4){n/4+if(it<n%4)1 else 0}
  val a=ArrayList<A>();val b=ArrayList<B>();val c=ArrayList<C>();val d=ArrayList<D>()
  for(i in 0 until lengths.max()){
   if(i<lengths[0])a.add(aNew(i+1L));if(i<lengths[1])b.add(bNew(i+1L))
   if(i<lengths[2])c.add(cNew(i+1L));if(i<lengths[3])d.add(dNew(i+1L))
  }
  if(shuffle){a.shuffle(Random(n+1));b.shuffle(Random(n+2));c.shuffle(Random(n+3));d.shuffle(Random(n+4))}
  close.addAll(a);close.addAll(b);close.addAll(c);close.addAll(d)
  val token=tokenNew(7);close.add(token);val sum=lengths.sumOf{it.toLong()*(it+1)/2}
  val mut:()->Long={mixedMut(a,b,c,d)};val shared:()->Long={mixedShared(a,b,c,d,token)}
  cases.add(PairedCase("mixed_mut",n,sum,listOf(mut,if(aa)mut else { {mixedMutFlat(a,b,c,d)} })))
  cases.add(PairedCase("mixed_shared",n,sum+7,listOf(shared,if(aa)shared else { {mixedSharedFlat(a,b,c,d,token)} })))
  val one:()->Long={mutOne(a)}
  cases.add(PairedCase("single_type_control",lengths[0],lengths[0].toLong()*(lengths[0]+1)/2,listOf(one,one)))
 }
 val bean=ManagementFactory.getThreadMXBean() as ThreadMXBean;bean.isThreadAllocatedMemoryEnabled=true;val thread=Thread.currentThread().id
 fun sample(c:PairedCase,v:Int,it:Int):Pair<Long,Long>{val bytes=bean.getThreadAllocatedBytes(thread);val start=System.nanoTime();var sum=0L
  repeat(it){sum+=c.run[v]()};val ns=System.nanoTime()-start;val allocated=bean.getThreadAllocatedBytes(thread)-bytes
  check(sum==c.expected*it);pairedSink=pairedSink xor sum;return ns to allocated
 }
 try {
  for(c in cases.shuffled(Random(fork+42))) {
   var iterations=(10000/(c.size+1)).coerceAtLeast(1)
   repeat(4){val(ns,_)=sample(c,0,iterations);val scale=(target.toDouble()/ns).coerceIn(.25,8.0);iterations=(iterations*scale).toInt().coerceIn(1,3000000)}
   repeat(4){for(v in 0..1)sample(c,v,iterations)}
   repeat(batches){batch->for(step in 0..1){val v=(step+batch+fork)%2;val(ns,bytes)=sample(c,v,iterations);println("PAIR,${c.name},${c.size},$v,$batch,$iterations,$ns,$bytes")}}
  }
 }finally {close.asReversed().forEach{it.close()}}
 check(live()==0L);println("SINK,$pairedSink")
}
