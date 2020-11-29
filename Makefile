SHADER_DIR := derky-d3d11/shaders
SHADERS := \
    geometry

# -----------------------------------------------------------------------------
SHADERS_QUALIFIED = $(addprefix $(SHADER_DIR)/,$(SHADERS))

all: $(addsuffix .vs.bin,$(SHADERS_QUALIFIED)) $(addsuffix .ps.bin,$(SHADERS_QUALIFIED))

%.vs.bin: %.hlsl FORCE
	fxc.exe	/T vs_5_0 /E vertex_main /Fo $@ $<

%.ps.bin: %.hlsl FORCE
	fxc.exe	/T ps_5_0 /E pixel_main /Fo $@ $<

FORCE:
.PHONY: all FORCE
