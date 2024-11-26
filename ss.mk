# -*- mode: conf -*-

# Only ABSOLUTE paths are to be used!

# If your path to GCC executable is:
#   /home/user/x-tools/foo/bar/bin/sh2eb-elf-gcc
# then set the variables to:
#   YAUL_INSTALL_ROOT=/home/user/x-tools/foo/bar
#   YAUL_PROG_SH_PREFIX=sh2eb-elf
#   YAUL_ARCH_SH_PREFIX=sh-elf

# Path to tool-chain installation directory
export YAUL_INSTALL_ROOT=/home/seth/.local/x-tools/sh2eb-elf

# SH-2 tool-chain program prefix (leave empty if the same as
# YAUL_ARCH_SH_PREFIX)
export YAUL_PROG_SH_PREFIX=

# SH-2 tool-chain prefix
export YAUL_ARCH_SH_PREFIX=sh2eb-elf

# M68k tool-chain prefix
export YAUL_ARCH_M68K_PREFIX=m68keb-elf

# Path to where the build is to be located
export YAUL_BUILD_ROOT=${HOME}/libyaul

# Name of build directory
export YAUL_BUILD=build

# Option: Memory allocator used:
# Values:
#  tlsf: Use TLSF (Two-Level Segregated Fit)
#      : Do not use any memory allocator
export YAUL_OPTION_MALLOC_IMPL="tlsf"

# Compilation verbosity
# Values:
#   : Verbose
#  1: Display build step line only
export SILENT=1

# Display ANSI colors during build process
# Values:
#   : Display
#  1: Do not display
export NOCOLOR=

# Enable DEBUG on a release build
# Values:
#   : Disable DEBUG
#  1: Enable DEBUG
export DEBUG_RELEASE=1

# Absolute path for executable xorrisofs
export MAKE_ISO_XORRISOFS=/usr/bin/xorrisofs

THIS_ROOT:=$(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

ifeq ($(strip $(YAUL_INSTALL_ROOT)),)
  $(error Undefined YAUL_INSTALL_ROOT (install root directory))
endif

ifneq (1,$(words [$(strip $(YAUL_INSTALL_ROOT))]))
  $(error YAUL_INSTALL_ROOT (install root directory) contains spaces)
endif

ifeq ($(strip $(YAUL_ARCH_SH_PREFIX)),)
  $(error Undefined YAUL_ARCH_SH_PREFIX (tool-chain prefix))
endif

ifneq (1,$(words [$(strip $(YAUL_ARCH_SH_PREFIX))]))
  $(error YAUL_ARCH_SH_PREFIX (tool-chain prefix) contains spaces)
endif

ifneq (1,$(words [$(strip $(YAUL_PROG_SH_PREFIX))]))
  $(error YAUL_PROG_SH_PREFIX (tool-chain program prefix) contains spaces)
endif

ifeq ($(strip $(SILENT)),)
  ECHO=
else
  ECHO=@
endif
export ECHO

ifeq ($(strip $(NOCOLOR)),)
V_BEGIN_BLACK= [1;30m
V_BEGIN_RED= [1;31m
V_BEGIN_GREEN= [1;32m
V_BEGIN_YELLOW= [1;33m
V_BEGIN_BLUE= [1;34m
V_BEGIN_MAGENTA= [1;35m
V_BEGIN_CYAN= [1;36m
V_BEGIN_WHITE= [1;37m
V_END= [m
else
V_BEGIN_BLACK=
V_BEGIN_RED=
V_BEGIN_GREEN=
V_BEGIN_YELLOW=
V_BEGIN_BLUE=
V_BEGIN_MAGENTA=
V_BEGIN_CYAN=
V_BEGIN_WHITE=
V_END=
endif
export V_BEGIN_YELLOW

# Get the Makefile that calls to include this Makefile
BUILD_ROOT:= $(patsubst %/,%,$(dir $(abspath $(firstword $(MAKEFILE_LIST)))))

ifeq '$(OS)' "Windows_NT"
EXE_EXT:= .exe

# In order to avoid the following error under MSYS2 and MinGW-w32:
#
# /opt/tool-chains/sh2eb-elf/bin/sh2eb-elf-gcc.exe: error while loading shared
# libraries: libwinpthread-1.dll: cannot open shared object file: No such file
# or directory
#
# We need to have /mingw/bin in our path. It's unclear exactly why, but
# libwinpthread-1.dll resides in /mingw64/bin. Copying libwinpthread-1.dll to
# /opt/tool-chains/sh2eb-elf/bin does not resolve the issue
PATH:= /mingw64/bin:$(PATH)
endif

# $1 -> Relative/absolute path to be converted using '@' instead of '/'
define macro-convert-build-path
$(SH_BUILD_PATH)/$(subst /,@,$(abspath $1))
endef

# $1 -> Variable
# $2 -> Index
define macro-word-split
$(word $2,$(subst ;, ,$1))
endef

YAUL:= yaul

YAUL_CFLAGS_shared:= -I$(YAUL_INSTALL_ROOT)/$(YAUL_ARCH_SH_PREFIX)/include/yaul

YAUL_CFLAGS:= $(YAUL_CFLAGS_shared)
YAUL_CXXFLAGS:= $(YAUL_CFLAGS_shared)

YAUL_LDFLAGS:=

# Customizable (must be overwritten in user's Makefile)
SH_PROGRAM?= unknown-program
SH_DEFSYMS?=
SH_SRCS?=
SH_SRCS_NO_LINK?=
SH_BUILD_DIR?= build
SH_OUTPUT_DIR?= .

# Customizable variables (must be overwritten in user's Makefile)
#   IMAGE_DIRECTORY        ISO/CUE
#   AUDIO_TRACKS_DIRECTORY ISO/CUE
#   IMAGE_1ST_READ_BIN     ISO/CUE
#   IP_VERSION             ISO/CUE, SS
#   IP_RELEASE_DATE        ISO/CUE, SS
#   IP_AREAS               ISO/CUE, SS
#   IP_PERIPHERALS         ISO/CUE, SS
#   IP_TITLE               ISO/CUE, SS
#   IP_MASTER_STACK_ADDR   ISO/CUE, SS
#   IP_SLAVE_STACK_ADDR    ISO/CUE, SS
#   IP_1ST_READ_ADDR       ISO/CUE, SS
#   IP_1ST_READ_SIZE       ISO/CUE, SS

YAUL_PROG_SH_PREFIX?= $(YAUL_ARCH_SH_PREFIX)
ifeq ($(strip $(YAUL_PROG_SH_PREFIX)),)
YAUL_PROG_SH_PREFIX:= $(YAUL_ARCH_SH_PREFIX)
endif

SH_BUILD_PATH= $(abspath $(SH_BUILD_DIR))
SH_OUTPUT_PATH= $(abspath $(SH_OUTPUT_DIR))

SH_AS:=      $(YAUL_INSTALL_ROOT)/bin/$(YAUL_PROG_SH_PREFIX)-as$(EXE_EXT)
SH_AR:=      $(YAUL_INSTALL_ROOT)/bin/$(YAUL_PROG_SH_PREFIX)-gcc-ar$(EXE_EXT)
SH_CC:=      $(YAUL_INSTALL_ROOT)/bin/$(YAUL_PROG_SH_PREFIX)-gcc$(EXE_EXT)
SH_CXX:=     $(YAUL_INSTALL_ROOT)/bin/$(YAUL_PROG_SH_PREFIX)-g++$(EXE_EXT)
SH_LD:=      $(YAUL_INSTALL_ROOT)/bin/$(YAUL_PROG_SH_PREFIX)-gcc$(EXE_EXT)
SH_NM:=      $(YAUL_INSTALL_ROOT)/bin/$(YAUL_PROG_SH_PREFIX)-gcc-nm$(EXE_EXT)
SH_OBJCOPY:= $(YAUL_INSTALL_ROOT)/bin/$(YAUL_PROG_SH_PREFIX)-objcopy$(EXE_EXT)
SH_OBJDUMP:= $(YAUL_INSTALL_ROOT)/bin/$(YAUL_PROG_SH_PREFIX)-objdump$(EXE_EXT)

M68K_AS:=      $(YAUL_INSTALL_ROOT)/bin/$(YAUL_ARCH_M68K_PREFIX)-as$(EXE_EXT)
M68K_AR:=      $(YAUL_INSTALL_ROOT)/bin/$(YAUL_ARCH_M68K_PREFIX)-gcc-ar$(EXE_EXT)
M68K_CC:=      $(YAUL_INSTALL_ROOT)/bin/$(YAUL_ARCH_M68K_PREFIX)-gcc$(EXE_EXT)
M68K_CXX:=     $(YAUL_INSTALL_ROOT)/bin/$(YAUL_ARCH_M68K_PREFIX)-g++$(EXE_EXT)
M68K_LD:=      $(YAUL_INSTALL_ROOT)/bin/$(YAUL_ARCH_M68K_PREFIX)-gcc$(EXE_EXT)
M68K_NM:=      $(YAUL_INSTALL_ROOT)/bin/$(YAUL_ARCH_M68K_PREFIX)-gcc-nm$(EXE_EXT)
M68K_OBJCOPY:= $(YAUL_INSTALL_ROOT)/bin/$(YAUL_ARCH_M68K_PREFIX)-objcopy$(EXE_EXT)
M68K_OBJDUMP:= $(YAUL_INSTALL_ROOT)/bin/$(YAUL_ARCH_M68K_PREFIX)-objdump$(EXE_EXT)

SH_AFLAGS= --fatal-warnings
SH_CFLAGS_shared:= \
	-W \
	-Wall \
	-Wduplicated-branches \
	-Wduplicated-cond \
	-Wextra \
	-Winit-self \
	-Wmissing-include-dirs \
	-Wno-format \
	-Wno-main \
	-Wnull-dereference \
	-Wshadow \
	-Wstrict-aliasing \
	-Wunused \
	-Wunused-parameter \
	-save-temps=obj

SH_LDFLAGS:= \
	-static \
	-Wl,--gc-sections \
	-Wl,-Map,$(SH_BUILD_PATH)/$(SH_PROGRAM).map \
	$(YAUL_LDFLAGS)

SH_CFLAGS:= \
	-std=c11 \
	-Wbad-function-cast \
	$(SH_CFLAGS_shared) \
	$(YAUL_CFLAGS)

SH_CXXFLAGS:= \
	-std=c++17 \
	-fno-exceptions \
	-fno-rtti \
	-fno-unwind-tables \
	-fno-asynchronous-unwind-tables \
	-fno-threadsafe-statics \
	-fno-use-cxa-atexit \
	$(SH_CFLAGS_shared) \
	$(YAUL_CXXFLAGS)

.PHONY: all generate-cdb clean .build

.SUFFIXES:
.SUFFIXES: .c .cc .C .cpp .cxx .sx .o .bin .elf

.PRECIOUS: %.elf %.c %.o

.SECONDARY: .build

all: .build

# Each asset follows the format: <path>;<symbol>. Duplicates are removed
BUILTIN_ASSETS=

SH_PROGRAM:= template
SH_SRCS:= \
	main.c

SH_CFLAGS+= -O2 -I. -DDEBUG -g
SH_LDFLAGS+=

IP_VERSION:= V1.000
IP_RELEASE_DATE:= 19940101
IP_AREAS:= JTUBKAEL
IP_PERIPHERALS:= JAMKST
IP_TITLE:= Template
IP_MASTER_STACK_ADDR:= 0x06004000
IP_SLAVE_STACK_ADDR:= 0x06001E00
IP_1ST_READ_ADDR:= 0x06004000
IP_1ST_READ_SIZE:= 0

# The targets which .SECONDARY depends on are treated as intermediate files,
# except that they are never automatically deleted
.SECONDARY: pre-build-iso post-build-iso

pre-build-iso:

post-build-iso:

ifeq ($(strip $(IP_VERSION)),)
  $(error Undefined IP_VERSION)
endif

ifeq ($(strip $(IP_RELEASE_DATE)),)
  $(error Undefined IP_RELEASE_DATE)
endif

ifeq ($(strip $(IP_AREAS)),)
  $(error Undefined IP_AREAS)
endif

ifeq ($(strip $(IP_PERIPHERALS)),)
  $(error Undefined IP_PERIPHERALS)
endif

ifeq ($(strip $(IP_TITLE)),)
  $(error Undefined IP_TITLE)
endif

ifeq ($(strip $(IP_MASTER_STACK_ADDR)),)
  $(error Undefined IP_MASTER_STACK_ADDR)
endif

ifeq ($(strip $(IP_SLAVE_STACK_ADDR)),)
  $(error Undefined IP_SLAVE_STACK_ADDR)
endif

ifeq ($(strip $(IP_1ST_READ_ADDR)),)
  $(error Undefined IP_1ST_READ_ADDR)
endif

ifeq ($(strip $(IP_1ST_READ_SIZE)),)
  $(error Undefined IP_1ST_READ_SIZE)
endif

SH_DEFSYMS+= \
	-Wl,--defsym=___master_stack=$(IP_MASTER_STACK_ADDR) \
	-Wl,--defsym=___slave_stack=$(IP_SLAVE_STACK_ADDR)

IMAGE_DIRECTORY?= cd
AUDIO_TRACKS_DIRECTORY?= audio-tracks
IMAGE_1ST_READ_BIN?= A.BIN

OUTPUT_FILES= $(SH_OUTPUT_PATH)/$(SH_PROGRAM).iso $(SH_OUTPUT_PATH)/$(SH_PROGRAM).cue
CLEAN_OUTPUT_FILES= $(OUTPUT_FILES) $(SH_BUILD_PATH)/IP.BIN $(SH_BUILD_PATH)/IP.BIN.map

THIS_FILE:=$(firstword $(MAKEFILE_LIST))

ifeq ($(strip $(SH_BUILD_DIR)),)
  $(error Empty SH_BUILD_DIR (SH build directory))
endif

ifeq ($(strip $(SH_OUTPUT_DIR)),)
  $(error Empty SH_OUTPUT_DIR (SH output directory))
endif

ifeq ($(strip $(SH_PROGRAM)),)
  $(error Empty SH_PROGRAM (SH program name))
endif

$(shell mkdir -p $(SH_BUILD_DIR))
$(shell mkdir -p $(SH_OUTPUT_DIR))

# $1 -> Path to asset file
# $2 -> Valid C symbol asset name
define macro-builtin-asset-rule
$(call macro-convert-build-path,$(addsuffix .o,$1)): $1
	@printf -- "$(V_BEGIN_CYAN)$(strip $1)$(V_END)\n"
	$(ECHO)$(YAUL_INSTALL_ROOT)/bin/bin2o $1 $2 $$@

SH_SRCS+= $(addsuffix .o,$1)
endef

$(foreach BUILTIN_ASSET,$(BUILTIN_ASSETS), \
	$(eval $(call macro-builtin-asset-rule,\
		$(call macro-word-split,$(BUILTIN_ASSET),1), \
		$(call macro-word-split,$(BUILTIN_ASSET),2))))

# Check that SH_SRCS don't include duplicates. Be mindful that sort remove
# duplicates
SH_SRCS_UNIQ= $(sort $(SH_SRCS))

SH_SRCS_C:= $(filter %.c,$(SH_SRCS_UNIQ))
SH_SRCS_CXX:= $(filter %.cxx,$(SH_SRCS_UNIQ)) \
	$(filter %.cpp,$(SH_SRCS_UNIQ)) \
	$(filter %.cc,$(SH_SRCS_UNIQ)) \
	$(filter %.C,$(SH_SRCS_UNIQ))
SH_SRCS_S:= $(filter %.sx,$(SH_SRCS_UNIQ))
SH_SRCS_OTHER:= $(filter-out %.c %.cxx %.cpp %.cc %.C %.sx,$(SH_SRCS_UNIQ))

SH_OBJS_UNIQ:= $(addsuffix .o,$(foreach SRC,$(SH_SRCS_C) $(SH_SRCS_CXX) $(SH_SRCS_S) $(SH_SRCS_OTHER),$(basename $(SRC))))
SH_OBJS_UNIQ:= $(foreach OBJ,$(SH_OBJS_UNIQ),$(call macro-convert-build-path,$(OBJ)))

SH_LDFLAGS+= $(SH_DEFSYMS)

ifeq ($(strip $(SH_SPECS)),)
  SH_SPECS:= yaul.specs yaul-main.specs
endif

# If there are any C++ files, add the specific C++ specs file. This is done to
# avoid adding (small) bloat to any C-only projects
ifneq ($(strip $(SH_SRCS_CXX)),)
  SH_CXX_SPECS:= yaul-main-c++.specs
endif

SH_DEPS:= $(SH_OBJS_UNIQ:.o=.d)
SH_TEMPS:= $(SH_OBJS_UNIQ:.o=.i) $(SH_OBJS_UNIQ:.o=.ii) $(SH_OBJS_UNIQ:.o=.s)

# Parse out included paths from GCC when the specs files are used. This is used
# to explictly populate each command database entry with include paths
SH_SYSTEM_INCLUDE_DIRS:=$(shell echo | $(SH_CC) -E -Wp,-v - 2>&1 | \
	awk '/^\s/ { sub(/^\s+/,""); gsub(/\\/,"/"); print }')

# $1 -> $<
define macro-sh-generate-cdb-rule
generate-cdb::
	$(ECHO)printf -- "C\n" >&2
	$(ECHO)printf -- "/usr/bin/gcc$(EXE_EXT)\n" >&2
	$(ECHO)printf -- "$(abspath $(1))\n" >&2
	$(ECHO)printf -- "-D__INTELLISENSE__ -m32 -nostdlibinc -Wno-gnu-statement-expression $(SH_CFLAGS) $(foreach dir,$(SH_SYSTEM_INCLUDE_DIRS),-isystem $(abspath $(dir))) $(foreach dir,$(SHARED_INCLUDE_DIRS),-isystem $(abspath $(dir))) --include="$(YAUL_INSTALL_ROOT)/$(YAUL_PROG_SH_PREFIX)/include/intellisense.h" -c $(abspath $(1))\n" >&2
endef

# $2 -> Build type (release, debug)
define macro-sh-c++-generate-cdb-rule
generate-cdb::
	$(ECHO)printf -- "C++\n" >&2
	$(ECHO)printf -- "/usr/bin/g++$(EXE_EXT)\n" >&2
	$(ECHO)printf -- "$(abspath $(1))\n" >&2
	$(ECHO)printf -- "-D__INTELLISENSE__ -m32 -nostdinc++ -nostdlibinc -Wno-gnu-statement-expression $(SH_CXXFLAGS) $(foreach dir,$(SH_SYSTEM_INCLUDE_DIRS),-isystem $(abspath $(dir))) $(foreach dir,$(SHARED_INCLUDE_DIRS),-isystem $(abspath $(dir))) --include="$(YAUL_INSTALL_ROOT)/$(YAUL_PROG_SH_PREFIX)/include/intellisense.h" -c $(abspath $(1))\n" >&2
endef

# $1 -> $<
# $2 -> $@
define macro-generate-sh-build-object
$2: $1
	@printf -- "$(V_BEGIN_YELLOW)$1$(V_END)\n"
	$(ECHO)$(SH_CC) -MT $2 -MF $(addsuffix .d,$(basename $2)) -MD $(SH_CFLAGS) $(foreach specs,$(SH_SPECS),-specs=$(specs)) -c -o $2 $1
endef

# $1 -> $<
# $2 -> $@
define macro-generate-sh-build-asm-object
$2: $1
	@printf -- "$(V_BEGIN_YELLOW)$1$(V_END)\n"
	$(ECHO)$(SH_CC) $(SH_CFLAGS) -c -o $2 $1
endef

# $1 -> $<
# $2 -> $@
define macro-generate-sh-build-c++-object
$2: $1
	@printf -- "$(V_BEGIN_YELLOW)$1$(V_END)\n"
	$(ECHO)$(SH_CXX) -MT $2 -MF $(addsuffix .d,$(basename $2)) -MD $(SH_CXXFLAGS) $(foreach specs,$(SH_SPECS),-specs=$(specs)) $(foreach specs,$(SH_CXX_SPECS),-specs=$(specs)) -c -o $2 $1
endef

OUTPUT_FILES?=

ifeq ($(strip $(OUTPUT_FILES)),)
OUTPUT_FILES= $(SH_OUTPUT_PATH)/$(SH_PROGRAM).bin
CLEAN_OUTPUT_FILES?= $(OUTPUT_FILES)

$(SH_OUTPUT_PATH)/$(SH_PROGRAM).bin: $(SH_BUILD_PATH)/$(SH_PROGRAM).bin
	$(ECHO)cp $< $@
endif

.build: $(OUTPUT_FILES)

$(SH_BUILD_PATH)/$(SH_PROGRAM).bin: $(SH_BUILD_PATH)/$(SH_PROGRAM).elf
	@printf -- "$(V_BEGIN_YELLOW)$(@F)$(V_END)\n"
	$(ECHO)$(SH_OBJCOPY) -O binary $< $@
	@[ -z "${SILENT}" ] && du -hs $@ | awk '{ print $$1; }' || true

$(SH_BUILD_PATH)/$(SH_PROGRAM).elf: $(SH_OBJS_UNIQ)
	@printf -- "$(V_BEGIN_YELLOW)$(@F)$(V_END)\n"
	$(ECHO)$(SH_LD) $(foreach specs,$(SH_SPECS),-specs=$(specs)) $(foreach specs,$(SH_CXX_SPECS),-specs=$(specs)) $(SH_OBJS_UNIQ) $(SH_LDFLAGS) -o $@
	$(ECHO)$(SH_NM) $(SH_BUILD_PATH)/$(SH_PROGRAM).elf > $(SH_BUILD_PATH)/$(SH_PROGRAM).sym
	$(ECHO)$(SH_OBJDUMP) -S $(SH_BUILD_PATH)/$(SH_PROGRAM).elf > $(SH_BUILD_PATH)/$(SH_PROGRAM).asm

$(foreach SRC,$(SH_SRCS_C), \
	$(eval $(call macro-generate-sh-build-object,$(SRC),\
		$(call macro-convert-build-path,$(addsuffix .o,$(basename $(SRC)))))))

$(foreach SRC,$(SH_SRCS_CXX), \
	$(eval $(call macro-generate-sh-build-c++-object,$(SRC),\
		$(call macro-convert-build-path,$(addsuffix .o,$(basename $(SRC)))))))

$(foreach SRC,$(SH_SRCS_S), \
	$(eval $(call macro-generate-sh-build-asm-object,$(SRC),\
		$(call macro-convert-build-path,$(addsuffix .o,$(basename $(SRC)))))))

$(foreach FILE,$(SH_SRCS_C),$(eval $(call macro-sh-generate-cdb-rule,$(FILE))))
$(foreach FILE,$(SH_SRCS_CXX),$(eval $(call macro-sh-c++-generate-cdb-rule,$(FILE))))

clean:
	$(ECHO)printf -- "$(V_BEGIN_CYAN)$(SH_PROGRAM)$(V_END) $(V_BEGIN_GREEN)clean$(V_END)\n"
	$(ECHO)-rm -f \
	    $(SH_OBJS_UNIQ) \
	    $(SH_DEPS) \
	    $(SH_TEMPS) \
	    $(SH_BUILD_PATH)/$(SH_PROGRAM).asm \
	    $(SH_BUILD_PATH)/$(SH_PROGRAM).bin \
	    $(SH_BUILD_PATH)/$(SH_PROGRAM).elf \
	    $(SH_BUILD_PATH)/$(SH_PROGRAM).map \
	    $(SH_BUILD_PATH)/$(SH_PROGRAM).sym \
	    $(CLEAN_OUTPUT_FILES)

-include $(SH_DEPS)

undefine macro-builtin-asset-rule
undefine macro-generate-sh-build-object
undefine macro-generate-sh-build-asm-object
undefine macro-generate-sh-build-c++-object

$(SH_OUTPUT_PATH)/$(SH_PROGRAM).iso: $(SH_BUILD_PATH)/$(SH_PROGRAM).bin $(SH_BUILD_PATH)/IP.BIN
	@printf -- "$(V_BEGIN_YELLOW)$(@F)$(V_END)\n"
    # This is a rather nasty hack to suppress any output from running the
    # pre/post-build-iso targets
	$(ECHO)$(MAKE) --no-print-directory $$([ -z "$(SILENT)" ] || printf -- "-s") -f $(THIS_FILE) pre-build-iso
	$(ECHO)mkdir -p $(IMAGE_DIRECTORY)
	$(ECHO)cp $(SH_BUILD_PATH)/$(SH_PROGRAM).bin $(IMAGE_DIRECTORY)/$(IMAGE_1ST_READ_BIN)
	$(ECHO)for txt in "ABS.TXT" "BIB.TXT" "CPY.TXT"; do \
	    if ! [ -s $(IMAGE_DIRECTORY)/$$txt ]; then \
		printf -- "empty\n" > $(IMAGE_DIRECTORY)/$$txt; \
	    fi \
	done
	$(ECHO)$(YAUL_INSTALL_ROOT)/share/wrap-error $(YAUL_INSTALL_ROOT)/bin/make-iso $(IMAGE_DIRECTORY) $(SH_BUILD_PATH)/IP.BIN $(SH_OUTPUT_PATH) $(SH_PROGRAM)
	$(ECHO)$(MAKE) --no-print-directory $$([ -z "$(SILENT)" ] || printf -- "-s") -f $(THIS_FILE) post-build-iso

$(SH_OUTPUT_PATH)/$(SH_PROGRAM).cue: | $(SH_OUTPUT_PATH)/$(SH_PROGRAM).iso
	@printf -- "$(V_BEGIN_YELLOW)$(@F)$(V_END)\n"
	$(ECHO)mkdir -p $(AUDIO_TRACKS_DIRECTORY)
	$(ECHO)$(YAUL_INSTALL_ROOT)/share/wrap-error $(YAUL_INSTALL_ROOT)/bin/make-cue $(AUDIO_TRACKS_DIRECTORY) $(SH_OUTPUT_PATH)/$(SH_PROGRAM).iso

$(SH_BUILD_PATH)/IP.BIN: $(YAUL_INSTALL_ROOT)/share/yaul/ip/ip.sx $(SH_BUILD_PATH)/$(SH_PROGRAM).bin
	$(ECHO)$(YAUL_INSTALL_ROOT)/share/wrap-error $(YAUL_INSTALL_ROOT)/bin/make-ip \
	    "$(SH_BUILD_PATH)/$(SH_PROGRAM).bin" \
		"$(IP_VERSION)" \
		$(IP_RELEASE_DATE) \
		"$(IP_AREAS)" \
		"$(IP_PERIPHERALS)" \
		'"$(IP_TITLE)"' \
		$(IP_MASTER_STACK_ADDR) \
		$(IP_SLAVE_STACK_ADDR) \
		$(IP_1ST_READ_ADDR) \
	    $(IP_1ST_READ_SIZE)