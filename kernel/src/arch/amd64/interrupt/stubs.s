.code64
.section .text

.set EXCEPTION_DUMMY_ERROR, 0xFFFFFFFFFFFFFFFF

.macro PUSH_REGS
	push rax
	push rbx
	push rcx
	push rdx
	push rsi
	push rdi
	push r8
	push r9
	push r10
	push r11
	push r12
	push r13
	push r14
	push r15
	push rbp

	mov rax, cr0
	push rax

	mov rax, cr3
	push rax

	mov rax, cr4 
	push rax 
.endm

.macro POP_REGS
	pop rax
	mov cr4, rax

	pop rax
	mov cr3, rax

	pop rax
	mov cr0, rax

	pop rbp
	pop r15
	pop r14
	pop r13
	pop r12
	pop r11
	pop r10
	pop r9
	pop r8
	pop rdi
	pop rsi
	pop rdx
	pop rcx
	pop rbx
	pop rax
.endm

.macro HANDLER n, handler 
	push \n
	PUSH_REGS
	mov rdi, rsp 
	.extern \handler
	call \handler
	POP_REGS
	add rsp, 16 
	iretq
.endm 

.macro EXCPT n, handler
	.align 8
	irq_handler_\n:
		HANDLER \n excpt_\handler
.endm 

.macro EXCPT_DUMMY n, handler
	.align 8
	irq_handler_\n:
        push EXCEPTION_DUMMY_ERROR 
		HANDLER \n excpt_\handler
.endm 

.macro IRQ n, handler
	.align 8
	irq_handler_\n:
        push EXCEPTION_DUMMY_ERROR
		HANDLER \n \handler
.endm 

EXCPT_DUMMY 0 division_error
EXCPT_DUMMY 1 debug
EXCPT_DUMMY 2 non_maskable_interrupt
EXCPT_DUMMY 3 breakpoint
EXCPT_DUMMY 4 overflow
EXCPT_DUMMY 5 bound_range_exceeded
EXCPT_DUMMY 6 invalid_opcode
EXCPT_DUMMY 7 device_not_available
EXCPT 8 double_fault
EXCPT_DUMMY 9 deprecated
EXCPT 10 invalid_tss
EXCPT 11 segment_not_present
EXCPT 12 stack_segment_fault
EXCPT 13 general_protection_fault
EXCPT 14 page_fault
EXCPT_DUMMY 15 reserved
EXCPT_DUMMY 16 x87_floating_point
EXCPT 17 alignment_check
EXCPT_DUMMY 18 machine_check
EXCPT_DUMMY 19 simd_floating_point
EXCPT_DUMMY 20 virtualization
EXCPT 21 control_protection
EXCPT_DUMMY 22 reserved
EXCPT_DUMMY 23 reserved 
EXCPT_DUMMY 24 reserved 
EXCPT_DUMMY 25 reserved 
EXCPT_DUMMY 26 reserved
EXCPT_DUMMY 27 reserved
EXCPT_DUMMY 28 hypervisor_injection
EXCPT 29 vmm_communication
EXCPT 30 security
EXCPT_DUMMY 31 reserved


IRQ 32 irq_timer
IRQ 33 unimp 
IRQ 34 unimp
IRQ 35 unimp
IRQ 36 unimp
IRQ 37 unimp
IRQ 38 unimp
IRQ 39 unimp
IRQ 40 unimp
IRQ 41 unimp
IRQ 42 unimp
IRQ 43 unimp
IRQ 44 unimp
IRQ 45 unimp 
IRQ 46 unimp
IRQ 47 unimp

IRQ 48 unimp
IRQ 49 unimp
IRQ 50 unimp
IRQ 51 unimp
IRQ 52 unimp
IRQ 53 unimp
IRQ 54 unimp
IRQ 55 unimp
IRQ 56 unimp
IRQ 57 unimp
IRQ 58 unimp
IRQ 59 unimp
IRQ 60 unimp
IRQ 61 unimp
IRQ 62 unimp
IRQ 63 unimp
IRQ 64 unimp
IRQ 65 unimp
IRQ 66 unimp
IRQ 67 unimp
IRQ 68 unimp
IRQ 69 unimp
IRQ 70 unimp
IRQ 71 unimp
IRQ 72 unimp
IRQ 73 unimp
IRQ 74 unimp
IRQ 75 unimp
IRQ 76 unimp
IRQ 77 unimp
IRQ 78 unimp
IRQ 79 unimp
IRQ 80 unimp
IRQ 81 unimp
IRQ 82 unimp
IRQ 83 unimp
IRQ 84 unimp
IRQ 85 unimp
IRQ 86 unimp
IRQ 87 unimp
IRQ 88 unimp
IRQ 89 unimp
IRQ 90 unimp
IRQ 91 unimp
IRQ 92 unimp
IRQ 93 unimp
IRQ 94 unimp
IRQ 95 unimp
IRQ 96 unimp
IRQ 97 unimp
IRQ 98 unimp
IRQ 99 unimp
IRQ 100 unimp
IRQ 101 unimp
IRQ 102 unimp
IRQ 103 unimp
IRQ 104 unimp
IRQ 105 unimp
IRQ 106 unimp
IRQ 107 unimp
IRQ 108 unimp
IRQ 109 unimp
IRQ 110 unimp
IRQ 111 unimp
IRQ 112 unimp
IRQ 113 unimp
IRQ 114 unimp
IRQ 115 unimp
IRQ 116 unimp
IRQ 117 unimp
IRQ 118 unimp
IRQ 119 unimp
IRQ 120 unimp
IRQ 121 unimp
IRQ 122 unimp
IRQ 123 unimp
IRQ 124 unimp
IRQ 125 unimp
IRQ 126 unimp
IRQ 127 unimp
IRQ 128 unimp
IRQ 129 unimp
IRQ 130 unimp
IRQ 131 unimp
IRQ 132 unimp
IRQ 133 unimp
IRQ 134 unimp
IRQ 135 unimp
IRQ 136 unimp
IRQ 137 unimp
IRQ 138 unimp
IRQ 139 unimp
IRQ 140 unimp
IRQ 141 unimp
IRQ 142 unimp
IRQ 143 unimp
IRQ 144 unimp
IRQ 145 unimp
IRQ 146 unimp
IRQ 147 unimp
IRQ 148 unimp
IRQ 149 unimp
IRQ 150 unimp
IRQ 151 unimp
IRQ 152 unimp
IRQ 153 unimp
IRQ 154 unimp
IRQ 155 unimp
IRQ 156 unimp
IRQ 157 unimp
IRQ 158 unimp
IRQ 159 unimp
IRQ 160 unimp
IRQ 161 unimp
IRQ 162 unimp
IRQ 163 unimp
IRQ 164 unimp
IRQ 165 unimp
IRQ 166 unimp
IRQ 167 unimp
IRQ 168 unimp
IRQ 169 unimp
IRQ 170 unimp
IRQ 171 unimp
IRQ 172 unimp
IRQ 173 unimp
IRQ 174 unimp
IRQ 175 unimp
IRQ 176 unimp
IRQ 177 unimp
IRQ 178 unimp
IRQ 179 unimp
IRQ 180 unimp
IRQ 181 unimp
IRQ 182 unimp
IRQ 183 unimp
IRQ 184 unimp
IRQ 185 unimp
IRQ 186 unimp
IRQ 187 unimp
IRQ 188 unimp
IRQ 189 unimp
IRQ 190 unimp
IRQ 191 unimp
IRQ 192 unimp
IRQ 193 unimp
IRQ 194 unimp
IRQ 195 unimp
IRQ 196 unimp
IRQ 197 unimp
IRQ 198 unimp
IRQ 199 unimp
IRQ 200 unimp
IRQ 201 unimp
IRQ 202 unimp
IRQ 203 unimp
IRQ 204 unimp
IRQ 205 unimp
IRQ 206 unimp
IRQ 207 unimp
IRQ 208 unimp
IRQ 209 unimp
IRQ 210 unimp
IRQ 211 unimp
IRQ 212 unimp
IRQ 213 unimp
IRQ 214 unimp
IRQ 215 unimp
IRQ 216 unimp
IRQ 217 unimp
IRQ 218 unimp
IRQ 219 unimp
IRQ 220 unimp
IRQ 221 unimp
IRQ 222 unimp
IRQ 223 unimp
IRQ 224 unimp
IRQ 225 unimp
IRQ 226 unimp
IRQ 227 unimp
IRQ 228 unimp
IRQ 229 unimp
IRQ 230 unimp
IRQ 231 unimp
IRQ 232 unimp
IRQ 233 unimp
IRQ 234 unimp
IRQ 235 unimp
IRQ 236 unimp
IRQ 237 unimp
IRQ 238 unimp
IRQ 239 unimp
IRQ 240 unimp
IRQ 241 unimp
IRQ 242 unimp
IRQ 243 unimp
IRQ 244 unimp
IRQ 245 unimp
IRQ 246 unimp
IRQ 247 unimp
IRQ 248 unimp
IRQ 249 unimp
IRQ 250 unimp
IRQ 251 unimp
IRQ 252 unimp
IRQ 253 unimp
IRQ 254 unimp
IRQ 255 unimp

.section .rodata

.global irq_routines
irq_routines:
.quad irq_handler_0
.quad irq_handler_1
.quad irq_handler_2
.quad irq_handler_3
.quad irq_handler_4
.quad irq_handler_5
.quad irq_handler_6
.quad irq_handler_7
.quad irq_handler_8
.quad irq_handler_9
.quad irq_handler_10
.quad irq_handler_11
.quad irq_handler_12
.quad irq_handler_13
.quad irq_handler_14
.quad irq_handler_15
.quad irq_handler_16
.quad irq_handler_17
.quad irq_handler_18
.quad irq_handler_19
.quad irq_handler_20
.quad irq_handler_21
.quad irq_handler_22
.quad irq_handler_23
.quad irq_handler_24
.quad irq_handler_25
.quad irq_handler_26
.quad irq_handler_27
.quad irq_handler_28
.quad irq_handler_29
.quad irq_handler_30
.quad irq_handler_31
.quad irq_handler_32
.quad irq_handler_33
.quad irq_handler_34
.quad irq_handler_35
.quad irq_handler_36
.quad irq_handler_37
.quad irq_handler_38
.quad irq_handler_39
.quad irq_handler_40
.quad irq_handler_41
.quad irq_handler_42
.quad irq_handler_43
.quad irq_handler_44
.quad irq_handler_45
.quad irq_handler_46
.quad irq_handler_47
.quad irq_handler_48
.quad irq_handler_49
.quad irq_handler_50
.quad irq_handler_51
.quad irq_handler_52
.quad irq_handler_53
.quad irq_handler_54
.quad irq_handler_55
.quad irq_handler_56
.quad irq_handler_57
.quad irq_handler_58
.quad irq_handler_59
.quad irq_handler_60
.quad irq_handler_61
.quad irq_handler_62
.quad irq_handler_63
.quad irq_handler_64
.quad irq_handler_65
.quad irq_handler_66
.quad irq_handler_67
.quad irq_handler_68
.quad irq_handler_69
.quad irq_handler_70
.quad irq_handler_71
.quad irq_handler_72
.quad irq_handler_73
.quad irq_handler_74
.quad irq_handler_75
.quad irq_handler_76
.quad irq_handler_77
.quad irq_handler_78
.quad irq_handler_79
.quad irq_handler_80
.quad irq_handler_81
.quad irq_handler_82
.quad irq_handler_83
.quad irq_handler_84
.quad irq_handler_85
.quad irq_handler_86
.quad irq_handler_87
.quad irq_handler_88
.quad irq_handler_89
.quad irq_handler_90
.quad irq_handler_91
.quad irq_handler_92
.quad irq_handler_93
.quad irq_handler_94
.quad irq_handler_95
.quad irq_handler_96
.quad irq_handler_97
.quad irq_handler_98
.quad irq_handler_99
.quad irq_handler_100
.quad irq_handler_101
.quad irq_handler_102
.quad irq_handler_103
.quad irq_handler_104
.quad irq_handler_105
.quad irq_handler_106
.quad irq_handler_107
.quad irq_handler_108
.quad irq_handler_109
.quad irq_handler_110
.quad irq_handler_111
.quad irq_handler_112
.quad irq_handler_113
.quad irq_handler_114
.quad irq_handler_115
.quad irq_handler_116
.quad irq_handler_117
.quad irq_handler_118
.quad irq_handler_119
.quad irq_handler_120
.quad irq_handler_121
.quad irq_handler_122
.quad irq_handler_123
.quad irq_handler_124
.quad irq_handler_125
.quad irq_handler_126
.quad irq_handler_127
.quad irq_handler_128
.quad irq_handler_129
.quad irq_handler_130
.quad irq_handler_131
.quad irq_handler_132
.quad irq_handler_133
.quad irq_handler_134
.quad irq_handler_135
.quad irq_handler_136
.quad irq_handler_137
.quad irq_handler_138
.quad irq_handler_139
.quad irq_handler_140
.quad irq_handler_141
.quad irq_handler_142
.quad irq_handler_143
.quad irq_handler_144
.quad irq_handler_145
.quad irq_handler_146
.quad irq_handler_147
.quad irq_handler_148
.quad irq_handler_149
.quad irq_handler_150
.quad irq_handler_151
.quad irq_handler_152
.quad irq_handler_153
.quad irq_handler_154
.quad irq_handler_155
.quad irq_handler_156
.quad irq_handler_157
.quad irq_handler_158
.quad irq_handler_159
.quad irq_handler_160
.quad irq_handler_161
.quad irq_handler_162
.quad irq_handler_163
.quad irq_handler_164
.quad irq_handler_165
.quad irq_handler_166
.quad irq_handler_167
.quad irq_handler_168
.quad irq_handler_169
.quad irq_handler_170
.quad irq_handler_171
.quad irq_handler_172
.quad irq_handler_173
.quad irq_handler_174
.quad irq_handler_175
.quad irq_handler_176
.quad irq_handler_177
.quad irq_handler_178
.quad irq_handler_179
.quad irq_handler_180
.quad irq_handler_181
.quad irq_handler_182
.quad irq_handler_183
.quad irq_handler_184
.quad irq_handler_185
.quad irq_handler_186
.quad irq_handler_187
.quad irq_handler_188
.quad irq_handler_189
.quad irq_handler_190
.quad irq_handler_191
.quad irq_handler_192
.quad irq_handler_193
.quad irq_handler_194
.quad irq_handler_195
.quad irq_handler_196
.quad irq_handler_197
.quad irq_handler_198
.quad irq_handler_199
.quad irq_handler_200
.quad irq_handler_201
.quad irq_handler_202
.quad irq_handler_203
.quad irq_handler_204
.quad irq_handler_205
.quad irq_handler_206
.quad irq_handler_207
.quad irq_handler_208
.quad irq_handler_209
.quad irq_handler_210
.quad irq_handler_211
.quad irq_handler_212
.quad irq_handler_213
.quad irq_handler_214
.quad irq_handler_215
.quad irq_handler_216
.quad irq_handler_217
.quad irq_handler_218
.quad irq_handler_219
.quad irq_handler_220
.quad irq_handler_221
.quad irq_handler_222
.quad irq_handler_223
.quad irq_handler_224
.quad irq_handler_225
.quad irq_handler_226
.quad irq_handler_227
.quad irq_handler_228
.quad irq_handler_229
.quad irq_handler_230
.quad irq_handler_231
.quad irq_handler_232
.quad irq_handler_233
.quad irq_handler_234
.quad irq_handler_235
.quad irq_handler_236
.quad irq_handler_237
.quad irq_handler_238
.quad irq_handler_239
.quad irq_handler_240
.quad irq_handler_241
.quad irq_handler_242
.quad irq_handler_243
.quad irq_handler_244
.quad irq_handler_245
.quad irq_handler_246
.quad irq_handler_247
.quad irq_handler_248
.quad irq_handler_249
.quad irq_handler_250
.quad irq_handler_251
.quad irq_handler_252
.quad irq_handler_253
.quad irq_handler_254
.quad irq_handler_255