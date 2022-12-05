MAKEFLAGS += --no-print-directory

# Build flags.
# ---------------------------------------------------------------------------

# Verbosity.
VERBOSE ?= 1
# Quiet.
Q := @
# Architecture
ARCH = x86_64

export Q ARCH VERBOSE

# Make constants.
# ---------------------------------------------------------------------------

# Source root.
srctree := $(abspath .)
# Makefile include directory.
makeinc := $(srctree)/scripts
# File name for relocatable binary blob.
blob := blob.a.o

export srctree makeinc echo blob

# Compiler configuration.
# ---------------------------------------------------------------------------

HOSTCC 		:= gcc
HOSTCCFLAGS	:= -Og -c  -ffreestanding
HOSTCCINC	:= -Iinclude
CC 			:= $(HOSTCC) $(HOSTCCFLAGS) $(HOSTCCINC) -o

HOSTAS		:= gcc
HOSTASFLAGS := -Og -c  -ffreestanding
HOSTASINC 	:= -Iinclude
AS 			:= $(HOSTAS) $(HOSTASFLAGS) $(HOSTASINC) -o
 
HOSTLD		:= ld
HOSTLDFLAGS	:= -nostdlib -static
LD			:= $(HOSTLD) $(HOSTLDFLAGS) -o
LDRELOCF	:= -r
 
# C-pre-processor
CPP 		:= gcc

export CC AS LD LDRELOCF CPP

# Makefile building variables.
# ---------------------------------------------------------------------------

ifeq ($(ARCH), x86_64)
ARCHSRC := arch/x86 
else
ARCHSRC := 
endif 

# Redundant for now. Always print.
ifeq ($(VERBOSE), 0)
ECHO := @echo
else 
ECHO := @echo
endif 

PHONY := 

export PHONY ARCHSRC ECHO

build := -f $(srctree)/scripts/makefile.build obj
clean := -f $(srctree)/scripts/makefile.clean path

# Commands.
# ---------------------------------------------------------------------------

all: 
	@:

clean: 
	$(Q)$(MAKE) $(clean)=.
	$(Q)rm -f $(vmimage) $(vmimage).bin

image: vmacorn
	@:

PHONY += all clean image

# Image creation.
# ---------------------------------------------------------------------------

# File name for final binary blob.
built-in 	:= built-in.a.o
# Image name.
vmimage		:= vmacorn

arch-y 		:= $(ARCHSRC)
drivers-y 	:=
core-y 		:= kernel/

vmacorn-deps 		:= $(patsubst %/,%,$(drivers-y) $(core-y) $(arch-y))
vmacorn-built-in	:= $(addsuffix /$(built-in),$(vmacorn-deps))

$(vmimage): $(vmimage).bin

$(vmimage).bin: prepare build blob
	$(echo) IMAGE $@
	$(Q)$(LD) $@ $(vmacorn-built-in)

prepare: 

build: $(vmacorn-deps)

$(vmacorn-deps): FORCE
	$(Q)$(MAKE) $(build)=$@

blob: $(vmacorn-built-in) 

%/$(built-in): FORCE
	$(echo) BUILT-IN $@ 
	$(Q)$(LD) $@ $(shell find $(dir $@) -wholename "*/$(blob)") $(LDRELOCF)

PHONY += prepare build blob

# Misc.
# ---------------------------------------------------------------------------

include $(makeinc)/makefile.tail 