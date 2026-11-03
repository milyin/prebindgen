
/tmp/c-output-after-normal/target/release/libc_collection_bench.so:     file format elf64-x86-64


Disassembly of section .init:

Disassembly of section .fini:

Disassembly of section .text:

0000000000014f10 <bench_make_numbers>:
   14f10:	41 57                	push   %r15
   14f12:	41 56                	push   %r14
   14f14:	41 55                	push   %r13
   14f16:	41 54                	push   %r12
   14f18:	53                   	push   %rbx
   14f19:	48 83 ec 40          	sub    $0x40,%rsp
   14f1d:	48 89 fe             	mov    %rdi,%rsi
   14f20:	48 8d 7c 24 28       	lea    0x28(%rsp),%rdi
   14f25:	ff 15 f5 c4 03 00    	call   *0x3c4f5(%rip)        # 51420 <_DYNAMIC+0x2a0>
   14f2b:	48 8b 5c 24 28       	mov    0x28(%rsp),%rbx
   14f30:	4c 8b 74 24 30       	mov    0x30(%rsp),%r14
   14f35:	4c 8b 7c 24 38       	mov    0x38(%rsp),%r15
   14f3a:	4d 85 ff             	test   %r15,%r15
   14f3d:	74 4b                	je     14f8a <bench_make_numbers+0x7a>
   14f3f:	4e 8d 2c fd 00 00 00 	lea    0x0(,%r15,8),%r13
   14f46:	00 
   14f47:	ff 15 93 c4 03 00    	call   *0x3c493(%rip)        # 513e0 <_DYNAMIC+0x260>
   14f4d:	be 08 00 00 00       	mov    $0x8,%esi
   14f52:	4c 89 ef             	mov    %r13,%rdi
   14f55:	ff 15 8d c4 03 00    	call   *0x3c48d(%rip)        # 513e8 <_DYNAMIC+0x268>
   14f5b:	48 85 c0             	test   %rax,%rax
   14f5e:	0f 84 bd 00 00 00    	je     15021 <bench_make_numbers+0x111>
   14f64:	49 89 c4             	mov    %rax,%r12
   14f67:	49 8d 4d f8          	lea    -0x8(%r13),%rcx
   14f6b:	48 83 f9 28          	cmp    $0x28,%rcx
   14f6f:	0f 92 c0             	setb   %al
   14f72:	4c 89 e2             	mov    %r12,%rdx
   14f75:	4c 29 f2             	sub    %r14,%rdx
   14f78:	48 83 fa 20          	cmp    $0x20,%rdx
   14f7c:	0f 92 c2             	setb   %dl
   14f7f:	08 c2                	or     %al,%dl
   14f81:	74 0c                	je     14f8f <bench_make_numbers+0x7f>
   14f83:	31 d2                	xor    %edx,%edx
   14f85:	4c 89 f0             	mov    %r14,%rax
   14f88:	eb 4a                	jmp    14fd4 <bench_make_numbers+0xc4>
   14f8a:	45 31 e4             	xor    %r12d,%r12d
   14f8d:	eb 64                	jmp    14ff3 <bench_make_numbers+0xe3>
   14f8f:	48 c1 e9 03          	shr    $0x3,%rcx
   14f93:	48 ff c1             	inc    %rcx
   14f96:	48 89 ca             	mov    %rcx,%rdx
   14f99:	48 83 e2 fc          	and    $0xfffffffffffffffc,%rdx
   14f9d:	49 8d 04 d6          	lea    (%r14,%rdx,8),%rax
   14fa1:	31 f6                	xor    %esi,%esi
   14fa3:	66 66 66 66 2e 0f 1f 	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   14faa:	84 00 00 00 00 00 
   14fb0:	41 0f 10 04 f6       	movups (%r14,%rsi,8),%xmm0
   14fb5:	41 0f 10 4c f6 10    	movups 0x10(%r14,%rsi,8),%xmm1
   14fbb:	41 0f 11 04 f4       	movups %xmm0,(%r12,%rsi,8)
   14fc0:	41 0f 11 4c f4 10    	movups %xmm1,0x10(%r12,%rsi,8)
   14fc6:	48 83 c6 04          	add    $0x4,%rsi
   14fca:	48 39 f2             	cmp    %rsi,%rdx
   14fcd:	75 e1                	jne    14fb0 <bench_make_numbers+0xa0>
   14fcf:	48 39 d1             	cmp    %rdx,%rcx
   14fd2:	74 1f                	je     14ff3 <bench_make_numbers+0xe3>
   14fd4:	4d 01 f5             	add    %r14,%r13
   14fd7:	49 8d 0c d4          	lea    (%r12,%rdx,8),%rcx
   14fdb:	0f 1f 44 00 00       	nopl   0x0(%rax,%rax,1)
   14fe0:	48 8b 10             	mov    (%rax),%rdx
   14fe3:	48 83 c0 08          	add    $0x8,%rax
   14fe7:	48 89 11             	mov    %rdx,(%rcx)
   14fea:	48 83 c1 08          	add    $0x8,%rcx
   14fee:	4c 39 e8             	cmp    %r13,%rax
   14ff1:	75 ed                	jne    14fe0 <bench_make_numbers+0xd0>
   14ff3:	48 85 db             	test   %rbx,%rbx
   14ff6:	74 15                	je     1500d <bench_make_numbers+0xfd>
   14ff8:	48 c1 e3 03          	shl    $0x3,%rbx
   14ffc:	ba 08 00 00 00       	mov    $0x8,%edx
   15001:	4c 89 f7             	mov    %r14,%rdi
   15004:	48 89 de             	mov    %rbx,%rsi
   15007:	ff 15 9b c3 03 00    	call   *0x3c39b(%rip)        # 513a8 <_DYNAMIC+0x228>
   1500d:	4c 89 e0             	mov    %r12,%rax
   15010:	4c 89 fa             	mov    %r15,%rdx
   15013:	48 83 c4 40          	add    $0x40,%rsp
   15017:	5b                   	pop    %rbx
   15018:	41 5c                	pop    %r12
   1501a:	41 5d                	pop    %r13
   1501c:	41 5e                	pop    %r14
   1501e:	41 5f                	pop    %r15
   15020:	c3                   	ret
   15021:	bf 08 00 00 00       	mov    $0x8,%edi
   15026:	4c 89 ee             	mov    %r13,%rsi
   15029:	ff 15 c9 c3 03 00    	call   *0x3c3c9(%rip)        # 513f8 <_DYNAMIC+0x278>
   1502f:	e9 e2 00 00 00       	jmp    15116 <bench_make_numbers+0x206>
   15034:	49 89 c7             	mov    %rax,%r15
   15037:	48 85 db             	test   %rbx,%rbx
   1503a:	74 1a                	je     15056 <bench_make_numbers+0x146>
   1503c:	48 c1 e3 03          	shl    $0x3,%rbx
   15040:	ba 08 00 00 00       	mov    $0x8,%edx
   15045:	4c 89 f7             	mov    %r14,%rdi
   15048:	48 89 de             	mov    %rbx,%rsi
   1504b:	ff 15 57 c3 03 00    	call   *0x3c357(%rip)        # 513a8 <_DYNAMIC+0x228>
   15051:	eb 03                	jmp    15056 <bench_make_numbers+0x146>
   15053:	49 89 c7             	mov    %rax,%r15
   15056:	4c 89 ff             	mov    %r15,%rdi
   15059:	ff 15 51 c3 03 00    	call   *0x3c351(%rip)        # 513b0 <_DYNAMIC+0x230>
   1505f:	48 89 c3             	mov    %rax,%rbx
   15062:	49 89 d6             	mov    %rdx,%r14
   15065:	ff 15 75 c3 03 00    	call   *0x3c375(%rip)        # 513e0 <_DYNAMIC+0x260>
   1506b:	bf 17 00 00 00       	mov    $0x17,%edi
   15070:	be 01 00 00 00       	mov    $0x1,%esi
   15075:	ff 15 6d c3 03 00    	call   *0x3c36d(%rip)        # 513e8 <_DYNAMIC+0x268>
   1507b:	48 85 c0             	test   %rax,%rax
   1507e:	75 15                	jne    15095 <bench_make_numbers+0x185>
   15080:	bf 01 00 00 00       	mov    $0x1,%edi
   15085:	be 17 00 00 00       	mov    $0x17,%esi
   1508a:	ff 15 68 c3 03 00    	call   *0x3c368(%rip)        # 513f8 <_DYNAMIC+0x278>
   15090:	e9 81 00 00 00       	jmp    15116 <bench_make_numbers+0x206>
   15095:	49 89 c7             	mov    %rax,%r15
   15098:	0f 10 05 f6 2b ff ff 	movups -0xd40a(%rip),%xmm0        # 7c95 <GCC_except_table151+0x39>
   1509f:	0f 11 00             	movups %xmm0,(%rax)
   150a2:	48 b8 70 61 6e 69 63 	movabs $0x64656b63696e6170,%rax
   150a9:	6b 65 64 
   150ac:	49 89 47 0f          	mov    %rax,0xf(%r15)
   150b0:	49 8b 06             	mov    (%r14),%rax
   150b3:	48 85 c0             	test   %rax,%rax
   150b6:	74 05                	je     150bd <bench_make_numbers+0x1ad>
   150b8:	48 89 df             	mov    %rbx,%rdi
   150bb:	ff d0                	call   *%rax
   150bd:	49 8b 76 08          	mov    0x8(%r14),%rsi
   150c1:	48 85 f6             	test   %rsi,%rsi
   150c4:	74 0d                	je     150d3 <bench_make_numbers+0x1c3>
   150c6:	49 8b 56 10          	mov    0x10(%r14),%rdx
   150ca:	48 89 df             	mov    %rbx,%rdi
   150cd:	ff 15 d5 c2 03 00    	call   *0x3c2d5(%rip)        # 513a8 <_DYNAMIC+0x228>
   150d3:	48 c7 04 24 17 00 00 	movq   $0x17,(%rsp)
   150da:	00 
   150db:	4c 89 7c 24 08       	mov    %r15,0x8(%rsp)
   150e0:	48 c7 44 24 10 17 00 	movq   $0x17,0x10(%rsp)
   150e7:	00 00 
   150e9:	48 89 e0             	mov    %rsp,%rax
   150ec:	48 89 44 24 18       	mov    %rax,0x18(%rsp)
   150f1:	48 8d 05 d8 ed ff ff 	lea    -0x1228(%rip),%rax        # 13ed0 <_RNvXsq_NtCscHgRw1M2fX5_5alloc6stringNtB5_6StringNtNtCsc36rpYXAlPq_4core3fmt7Display3fmt>
   150f8:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
   150fd:	48 8d 3d f7 35 ff ff 	lea    -0xca09(%rip),%rdi        # 86fb <GCC_except_table151+0xa9f>
   15104:	48 8d 15 cd a9 03 00 	lea    0x3a9cd(%rip),%rdx        # 4fad8 <__frame_dummy_init_array_entry+0x68>
   1510b:	48 8d 74 24 18       	lea    0x18(%rsp),%rsi
   15110:	ff 15 ea c2 03 00    	call   *0x3c2ea(%rip)        # 51400 <_DYNAMIC+0x280>
   15116:	0f 0b                	ud2
   15118:	49 8b 76 08          	mov    0x8(%r14),%rsi
   1511c:	48 85 f6             	test   %rsi,%rsi
   1511f:	74 32                	je     15153 <bench_make_numbers+0x243>
   15121:	49 8b 56 10          	mov    0x10(%r14),%rdx
   15125:	48 89 df             	mov    %rbx,%rdi
   15128:	ff 15 7a c2 03 00    	call   *0x3c27a(%rip)        # 513a8 <_DYNAMIC+0x228>
   1512e:	ff 15 8c c2 03 00    	call   *0x3c28c(%rip)        # 513c0 <_DYNAMIC+0x240>
   15134:	48 8b 3c 24          	mov    (%rsp),%rdi
   15138:	48 8b 74 24 08       	mov    0x8(%rsp),%rsi
   1513d:	e8 0e ed ff ff       	call   13e50 <_RINvNtCsc36rpYXAlPq_4core3ptr9drop_glueNtNtCscHgRw1M2fX5_5alloc6string6StringECsjcgVWhsr6Vq_18c_collection_bench>
   15142:	ff 15 78 c2 03 00    	call   *0x3c278(%rip)        # 513c0 <_DYNAMIC+0x240>
   15148:	48 89 df             	mov    %rbx,%rdi
   1514b:	4c 89 f6             	mov    %r14,%rsi
   1514e:	e8 9d ec ff ff       	call   13df0 <_RINvNtCsc36rpYXAlPq_4core3ptr9drop_glueINtNtCscHgRw1M2fX5_5alloc5boxed3BoxDNtNtB4_3any3AnyNtNtB4_6marker4SendEL_EECsjcgVWhsr6Vq_18c_collection_bench>
   15153:	ff 15 67 c2 03 00    	call   *0x3c267(%rip)        # 513c0 <_DYNAMIC+0x240>
   15159:	ff 15 a9 c2 03 00    	call   *0x3c2a9(%rip)        # 51408 <_DYNAMIC+0x288>
   1515f:	ff 15 5b c2 03 00    	call   *0x3c25b(%rip)        # 513c0 <_DYNAMIC+0x240>

Disassembly of section .plt:
