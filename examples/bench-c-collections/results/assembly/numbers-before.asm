
/tmp/c-output-before-normal-v3/target/release/libc_collection_bench.so:     file format elf64-x86-64


Disassembly of section .init:

Disassembly of section .fini:

Disassembly of section .text:

0000000000014400 <bench_make_numbers>:
   14400:	48 83 ec 68          	sub    $0x68,%rsp
   14404:	48 89 3c 24          	mov    %rdi,(%rsp)
   14408:	48 8d 44 24 30       	lea    0x30(%rsp),%rax
   1440d:	48 89 44 24 08       	mov    %rax,0x8(%rsp)
   14412:	48 8d 7c 24 50       	lea    0x50(%rsp),%rdi
   14417:	48 89 e6             	mov    %rsp,%rsi
   1441a:	48 8d 54 24 08       	lea    0x8(%rsp),%rdx
   1441f:	e8 5c 1d 00 00       	call   16180 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_>
   14424:	48 83 7c 24 50 ff    	cmpq   $0xffffffffffffffff,0x50(%rsp)
   1442a:	75 0f                	jne    1443b <bench_make_numbers+0x3b>
   1442c:	48 8b 44 24 30       	mov    0x30(%rsp),%rax
   14431:	48 8b 54 24 38       	mov    0x38(%rsp),%rdx
   14436:	48 83 c4 68          	add    $0x68,%rsp
   1443a:	c3                   	ret
   1443b:	48 8b 44 24 60       	mov    0x60(%rsp),%rax
   14440:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
   14445:	0f 10 44 24 50       	movups 0x50(%rsp),%xmm0
   1444a:	0f 29 44 24 10       	movaps %xmm0,0x10(%rsp)
   1444f:	48 8d 44 24 10       	lea    0x10(%rsp),%rax
   14454:	48 89 44 24 40       	mov    %rax,0x40(%rsp)
   14459:	48 8d 05 b0 fc ff ff 	lea    -0x350(%rip),%rax        # 14110 <_RNvXsq_NtCscHgRw1M2fX5_5alloc6stringNtB5_6StringNtNtCsc36rpYXAlPq_4core3fmt7Display3fmt>
   14460:	48 89 44 24 48       	mov    %rax,0x48(%rsp)
   14465:	48 8d 3d 97 42 ff ff 	lea    -0xbd69(%rip),%rdi        # 8703 <GCC_except_table151+0x99f>
   1446c:	48 8d 15 4d cd 03 00 	lea    0x3cd4d(%rip),%rdx        # 511c0 <__frame_dummy_init_array_entry+0x50>
   14473:	48 8d 74 24 40       	lea    0x40(%rsp),%rsi
   14478:	ff 15 5a e6 03 00    	call   *0x3e65a(%rip)        # 52ad8 <_DYNAMIC+0x270>
   1447e:	0f 0b                	ud2
   14480:	ff 15 4a e6 03 00    	call   *0x3e64a(%rip)        # 52ad0 <_DYNAMIC+0x268>
   14486:	48 8b 7c 24 10       	mov    0x10(%rsp),%rdi
   1448b:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
   14490:	e8 0b f9 ff ff       	call   13da0 <_RINvNtCsc36rpYXAlPq_4core3ptr9drop_glueNtNtCscHgRw1M2fX5_5alloc6string6StringECsbJkkCGhgRyB_18c_collection_bench>
   14495:	ff 15 35 e6 03 00    	call   *0x3e635(%rip)        # 52ad0 <_DYNAMIC+0x268>

Disassembly of section .plt:
