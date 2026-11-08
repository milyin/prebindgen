#!/usr/bin/env python3
"""Patch only multi-type Kotlin preparation; native bytes are shared."""
import argparse,re,shutil,subprocess,json,hashlib
from pathlib import Path
HERE=Path(__file__).resolve().parent
HELPER='''package bench.types

private val flatOrder = Comparator<NativeHandle> { a,b -> compareHandleOrder(a,b) }
internal fun <R> withFlatInputLocks(values:ArrayList<NativeHandle>, exclusive:Boolean,body:()->R):R {
    // Fixture only: mixedMut is all exclusive; mixedShared has a singleton
    // Token of a different type, so none of its shared lists can conflict.
    values.sortWith(flatOrder)
    if(exclusive) for(i in 1 until values.size) {
        val a=values[i-1];val b=values[i]
        if(a.handleType==b.handleType && a.address==b.address)
            throw HandleValidationFailure("conflicting input aliases")
    }
    return withOrderedHandleLocks(values,body)
}
'''
def patch(source):
 for name,count,exclusive in [('mixedShared',5,False),('mixedMut',4,True)]:
  start=source.index('public fun '+name+'(');end=source.find('\npublic fun ',start+1)
  if end<0:end=len(source)
  f=source[start:end];a=f.index('val __pb_ordered0 =')
  pattern=r'val __pb_ordered\d+ = bench.types.prepareInputHandles\((.*?), (true|false)\); '
  decls=list(re.finditer(pattern,f[a:]));assert len(decls)==count
  expressions=[d[1] for d in decls]
  # Inspect actual generated bucket order, then prove the combined list uses it.
  expected=[4,1,2,0,3] if count==5 else [1,2,0,3]
  actual=[int(re.search(r'__pb_handle(\d+)',e)[1]) for e in expressions]
  assert actual==expected,(name,actual)
  prefix=''.join(d[0] for d in decls)
  assert f[a:a+len(prefix)]==prefix
  # Parenthesize nullable-count expression; otherwise following sums bind to else.
  size=' + '.join(('(if (__pb_handle4 == null) 0 else 1)' if 'listOfNotNull' in e else e+'.size') for e in expressions)
  replacement=f'val __pb_all = ArrayList<bench.types.NativeHandle>({size}); '
  for e in expressions:
   replacement+=('__pb_handle4?.let { __pb_all.add(it) }; ' if 'listOfNotNull' in e else f'for (__pb_index in {e}.indices) {{ __pb_all.add({e}[__pb_index]) }}; ')
  locks=''.join(f'bench.types.withOrderedHandleLocks(__pb_ordered{i}) {{ ' for i in range(count))
  assert f[a+len(prefix):].startswith(locks)
  body_start=a+len(prefix)+len(locks);depth=1;body_end=None
  # Locate matching end of innermost lock; braces in fixture strings do not occur here.
  for i in range(body_start,len(f)):
   if f[i]=='{':depth+=1
   if f[i]=='}':
    depth-=1
    if depth==0:body_end=i;break
  assert body_end is not None
  tail=body_end+1
  for _ in range(count-1):
   while f[tail].isspace():tail+=1
   assert f[tail]=='}';tail+=1
  replacement+=f'bench.types.withFlatInputLocks(__pb_all, {str(exclusive).lower()}) {{ '+f[body_start:body_end]+'}'
  f=f[:a]+replacement+f[tail:];source=source[:start]+f+source[end:]
 return source

def main():
 p=argparse.ArgumentParser();p.add_argument('--build',type=Path,required=True);p.add_argument('--variants',nargs='+',choices=['buckets','flat','paired'],default=['buckets','flat','paired']);p.add_argument('--kotlinc',default='/snap/kotlin/current/bin/kotlinc');a=p.parse_args();out=a.build.resolve()
 flat=out/'kotlin-flat';shutil.copytree(out/'kotlin-buckets',flat,dirs_exist_ok=True)
 f=flat/'bench/types/Functions.kt';f.write_text(patch(f.read_text()));(flat/'bench/types/FlatLocks.kt').write_text(HELPER)
 paired=out/'kotlin-paired';shutil.copytree(out/'kotlin-buckets',paired,dirs_exist_ok=True)
 patched=f.read_text();functions=[]
 for order in [['B','C','A','D'],['Token','B','C','A','D']]:
  ids=[int(re.search(r'handleType: Long get\(\) = (-?\d+)L',(flat/'bench/types'/(name+'.kt')).read_text())[1]) for name in order]
  assert ids==sorted(ids),ids
 for name in ['mixedMut','mixedShared']:
  start=patched.index('public fun '+name+'(');end=patched.find('\npublic fun ',start+1)
  if end<0:end=len(patched)
  functions.append(patched[start:end].replace('public fun '+name+'(', 'public fun '+name+'Flat(',1))
 (paired/'bench/types/FlatFunctions.kt').write_text('package bench.types\n'+HELPER.split('package bench.types\n',1)[1]+'\n'.join(functions))
 for variant in a.variants:
  files=list((out/('kotlin-'+variant)).rglob('*.kt'))+[HERE/'Checks.kt',HERE/'Paired.kt' if variant=='paired' else HERE.parents[1]/'bench-handle-inputs/Bench.kt']
  with (out/(variant+'-kotlinc.log')).open('w') as log:
   subprocess.run([a.kotlinc,'-J-Xmx768m',*map(str,files),'-include-runtime','-d',str(out/(variant+'.jar'))],stdout=log,stderr=subprocess.STDOUT,check=True)
  print('compiled',variant,flush=True)
 sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
 (out/'kotlin-build.json').write_text(json.dumps({'kotlinc':subprocess.check_output([a.kotlinc,'-version'],stderr=subprocess.STDOUT,text=True),'jars':{v:sha(out/(v+'.jar')) for v in a.variants},'generated':{str(p.relative_to(out)):sha(p) for v in a.variants for p in (out/('kotlin-'+v)).rglob('*.kt')},'sources':{str(p):sha(p) for p in [HERE/'kotlin.py',HERE/'Paired.kt',HERE/'Checks.kt',HERE.parents[1]/'bench-handle-inputs/Bench.kt']}},indent=2))
if __name__=='__main__':main()
